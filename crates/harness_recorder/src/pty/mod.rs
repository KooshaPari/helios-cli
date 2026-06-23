use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::script::TerminalSettings;

pub mod controller;
pub mod capture;

pub use controller::TerminalController;

pub struct Terminal {
    pty_pair: portable_pty::PtyPair,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    reader: Arc<std::sync::Mutex<Box<dyn Read + Send>>>,
    buffer: Arc<std::sync::Mutex<String>>,
}

impl Terminal {
    pub fn new(settings: &TerminalSettings) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: settings.height,
                cols: settings.width,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;
        
        let mut cmd = CommandBuilder::new(&settings.shell);
        
        if let Some(working_dir) = &settings.working_dir {
            cmd.cwd(working_dir);
        }
        
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;
        
        let writer = pty_pair.master.take_writer()
            .context("Failed to get PTY writer")?;
        
        let reader = Arc::new(std::sync::Mutex::new(
            pty_pair.master.try_clone_reader()
                .context("Failed to get PTY reader")?
        ));
        
        let buffer = Arc::new(std::sync::Mutex::new(String::new()));
        
        // Start background thread to read output
        let reader_clone = reader.clone();
        let buffer_clone = buffer.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok(mut reader) = reader_clone.lock() {
                    match reader.read(&mut buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let text = String::from_utf8_lossy(&buf[..n]);
                            if let Ok(mut buffer) = buffer_clone.lock() {
                                buffer.push_str(&text);
                            }
                        }
                        Err(_) => break,
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        
        Ok(Terminal {
            pty_pair,
            child,
            writer,
            reader,
            buffer,
        })
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        self.send_input(&format!("{}\n", command)).await
    }
    
    pub async fn send_input(&mut self, input: &str) -> Result<()> {
        self.writer.write_all(input.as_bytes())
            .context("Failed to write to PTY")?;
        self.writer.flush()
            .context("Failed to flush PTY writer")?;
        Ok(())
    }
    
    pub async fn type_text(&mut self, text: &str, delay_per_char: Duration) -> Result<()> {
        for ch in text.chars() {
            self.send_input(&ch.to_string()).await?;
            tokio::time::sleep(delay_per_char).await;
        }
        Ok(())
    }
    
    pub fn get_output(&self) -> String {
        self.buffer.lock()
            .map(|buffer| buffer.clone())
            .unwrap_or_default()
    }
    
    pub fn get_size(&self) -> (u16, u16) {
        let size = self.pty_pair.master.get_size()
            .unwrap_or(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            });
        (size.cols, size.rows)
    }
    
    pub async fn wait_for_output(&self, pattern: &str, timeout_duration: Duration) -> Result<bool> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout_duration {
            let output = self.get_output();
            if output.contains(pattern) {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(false)
    }
    
    pub fn clear_buffer(&self) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}