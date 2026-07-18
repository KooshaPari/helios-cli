// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Phenotype org (heliosCLI)

//! Queue module - High-performance queues for heliosHarness

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{debug, instrument};

/// Error types for queues
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Channel is closed")]
    Closed,

    #[error("Channel is full")]
    Full,

    #[error("Channel is empty")]
    Empty,

    #[error("Send error: {0}")]
    Send(String),

    #[error("Receive error: {0}")]
    Receive(String),

    /// Underlying I/O failure (e.g. poisoned mutex serialised to I/O).
    ///
    /// Traces to: FR-HELIOS-IO-009
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// MPSC (Multiple Producer Single Consumer) channel
pub struct Channel<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    size: Arc<AtomicUsize>,
    closed: Arc<Mutex<bool>>,
}

impl<T> Channel<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            capacity,
            size: Arc::new(AtomicUsize::new(0)),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Traces to: FR-HELIOS-Q-001
    #[instrument(skip(self, item), fields(capacity = self.capacity))]
    pub fn send(&self, item: T) -> Result<(), QueueError> {
        // `PoisonError<MutexGuard>` is not `Sync`, so we can't pass it
        // directly to `io::Error::other` (which requires `Into<Box<dyn
        // Error + Send + Sync>>`). Serialising via `to_string()` is the
        // canonical workaround.
        let is_closed = {
            let closed = self.closed.lock().map_err(|e| std::io::Error::other(e.to_string()))?;
            *closed
        };
        if is_closed {
            debug!("rejecting send: channel closed");
            return Err(QueueError::Closed);
        }

        let mut buffer = self.buffer.lock().map_err(|e| std::io::Error::other(e.to_string()))?;
        if buffer.len() >= self.capacity {
            debug!(len = buffer.len(), "rejecting send: channel full");
            return Err(QueueError::Full);
        }

        buffer.push_back(item);
        self.size.fetch_add(1, Ordering::Relaxed);
        debug!(size = self.size.load(Ordering::Relaxed), "sent item");
        Ok(())
    }

    pub fn recv(&self) -> Option<T> {
        let mut buffer = self.buffer.lock().ok()?;
        if buffer.is_empty() {
            return None;
        }
        self.size.fetch_sub(1, Ordering::Relaxed);
        buffer.pop_front()
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    pub fn close(&self) {
        if let Ok(mut closed) = self.closed.lock() {
            *closed = true;
        }
    }
}

/// Ring buffer for single producer/consumer
#[allow(dead_code)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    read: usize,
    write: usize,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self { data: Vec::with_capacity(capacity), read: 0, write: 0, capacity }
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.data.len() >= self.capacity {
            return false;
        }
        self.data.push(item);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.read >= self.data.len() {
            return None;
        }
        let item = self.data.remove(self.read);
        self.read += 1;
        Some(item)
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.read)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Work-stealing queue for parallel processing
pub struct WorkQueue<T> {
    local: Mutex<VecDeque<T>>,
    global: Arc<Mutex<VecDeque<T>>>,
}

impl<T> WorkQueue<T> {
    pub fn new() -> Self {
        Self { local: Mutex::new(VecDeque::new()), global: Arc::new(Mutex::new(VecDeque::new())) }
    }

    pub fn push(&self, item: T) {
        if let Ok(mut q) = self.local.lock() {
            q.push_back(item);
        }
    }

    pub fn pop(&self) -> Option<T> {
        if let Ok(mut q) = self.local.lock() {
            if let Some(item) = q.pop_front() {
                return Some(item);
            }
        }
        if let Ok(mut g) = self.global.lock() {
            return g.pop_back();
        }
        None
    }

    pub fn steal(&self) -> Option<T> {
        if let Ok(mut g) = self.global.lock() {
            return g.pop_back();
        }
        None
    }
}

impl<T> Default for WorkQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Traces to: FR-HELIOS-IO-009
    /// `From<io::Error>` must produce the `Io` variant.
    #[test]
    fn from_io_error_maps_to_io_variant() {
        let io_err = std::io::Error::other("queue lock poisoned");
        let err: QueueError = io_err.into();
        assert!(matches!(err, QueueError::Io(_)));
    }

    /// Traces to: FR-HELIOS-IO-009
    /// Display must surface the underlying I/O message.
    #[test]
    fn io_error_display_includes_message() {
        let io_err = std::io::Error::other("queue lock poisoned");
        let err: QueueError = io_err.into();
        assert!(err.to_string().contains("queue lock poisoned"));
    }

    /// Traces to: FR-HELIOS-Q-001
    /// Sending after close returns the Closed variant.
    #[test]
    fn send_after_close_returns_closed() {
        let ch: Channel<i32> = Channel::new(4);
        ch.close();
        let result = ch.send(1);
        assert!(matches!(result, Err(QueueError::Closed)));
    }

    /// Traces to: FR-HELIOS-Q-001
    /// Sending past capacity returns the Full variant.
    #[test]
    fn send_at_capacity_returns_full() {
        let ch: Channel<i32> = Channel::new(2);
        ch.send(1).unwrap();
        ch.send(2).unwrap();
        let result = ch.send(3);
        assert!(matches!(result, Err(QueueError::Full)));
    }

    #[test]
    fn channel_send_recv_and_capacity_helpers() {
        let ch: Channel<i32> = Channel::new(4);
        ch.send(42).unwrap();
        assert_eq!(ch.len(), 1);
        assert!(!ch.is_empty());
        assert!(!ch.is_full());
        assert_eq!(ch.recv(), Some(42));
        assert!(ch.is_empty());
        assert!(ch.recv().is_none());
    }

    #[test]
    fn ring_buffer_push_pop_and_capacity() {
        let mut ring: RingBuffer<i32> = RingBuffer::new(2);
        assert!(ring.push(1));
        assert!(ring.push(2));
        assert!(!ring.push(3));
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.pop(), Some(1));
        assert_eq!(ring.len(), 0);
    }

    #[test]
    fn work_queue_local_push_pop() {
        let queue: WorkQueue<i32> = WorkQueue::default();
        queue.push(7);
        assert_eq!(queue.pop(), Some(7));
        assert!(queue.pop().is_none());
        assert!(queue.steal().is_none());
    }
}
