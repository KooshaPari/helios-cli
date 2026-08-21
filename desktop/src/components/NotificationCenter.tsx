import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { Notification, NotificationCounts, NotificationSource } from '../types';
import './NotificationCenter.css';

interface NotificationCenterProps {
  isOpen: boolean;
  onClose: () => void;
}

const SOURCE_LABELS: Record<NotificationSource, string> = {
  tracera: 'Tracera',
  agileplus: 'AgilePlus',
  github: 'GitHub',
  helios: 'Helios',
};

const SOURCE_COLORS: Record<NotificationSource, string> = {
  tracera: '#3b82f6',
  agileplus: '#a855f7',
  github: '#e6edf3',
  helios: '#10b981',
};

const TYPE_LABELS: Record<string, string> = {
  tracera_issue: 'Tracera Issue',
  agile_plus_gate_failure: 'Gate Failure',
  agent_error: 'Agent Error',
  ci_status: 'CI Status',
  task_complete: 'Task Complete',
};

function NotificationCenter({ isOpen, onClose }: NotificationCenterProps) {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [counts, setCounts] = useState<NotificationCounts | null>(null);
  const [loading, setLoading] = useState(false);
  const [activeFilter, setActiveFilter] = useState<NotificationSource | 'all'>('all');
  const panelRef = useRef<HTMLDivElement>(null);

  const fetchNotifications = useCallback(async () => {
    setLoading(true);
    try {
      const [notifs, cts] = await Promise.all([
        invoke<Notification[]>('list_notifications', { limit: 100 }),
        invoke<NotificationCounts>('get_notification_counts'),
      ]);
      setNotifications(notifs);
      setCounts(cts);
    } catch (err) {
      console.error('Failed to load notifications:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      fetchNotifications();
    }
  }, [isOpen, fetchNotifications]);

  // Close on outside click.
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen, onClose]);

  const handleMarkRead = async (notificationId: string) => {
    try {
      await invoke('mark_notification_read', { notificationId });
      setNotifications((prev) =>
        prev.map((n) =>
          n.id === notificationId ? { ...n, is_read: true } : n
        )
      );
      // Refresh counts.
      const cts = await invoke<NotificationCounts>('get_notification_counts');
      setCounts(cts);
    } catch (err) {
      console.error('Failed to mark notification as read:', err);
    }
  };

  const handleMarkAllRead = async () => {
    try {
      await invoke('mark_all_notifications_read');
      setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
      const cts = await invoke<NotificationCounts>('get_notification_counts');
      setCounts(cts);
    } catch (err) {
      console.error('Failed to mark all as read:', err);
    }
  };

  const filteredNotifications =
    activeFilter === 'all'
      ? notifications
      : notifications.filter((n) => n.source === activeFilter);

  if (!isOpen) return null;

  return (
    <div className="notification-center" ref={panelRef}>
      <div className="nc-header">
        <h3 className="nc-title">Notifications</h3>
        <div className="nc-actions">
          {counts && counts.unread > 0 && (
            <button className="nc-mark-all" onClick={handleMarkAllRead}>
              Mark all read
            </button>
          )}
          <button className="nc-close" onClick={onClose}>
            &times;
          </button>
        </div>
      </div>

      {/* Source filter tabs */}
      <div className="nc-filters">
        <button
          className={`nc-filter ${activeFilter === 'all' ? 'active' : ''}`}
          onClick={() => setActiveFilter('all')}
        >
          All {counts ? `(${counts.total})` : ''}
        </button>
        <button
          className={`nc-filter ${activeFilter === 'tracera' ? 'active' : ''}`}
          onClick={() => setActiveFilter('tracera')}
        >
          <span className="filter-dot" style={{ background: SOURCE_COLORS.tracera }} />
          Tracera
        </button>
        <button
          className={`nc-filter ${activeFilter === 'agileplus' ? 'active' : ''}`}
          onClick={() => setActiveFilter('agileplus')}
        >
          <span className="filter-dot" style={{ background: SOURCE_COLORS.agileplus }} />
          AgilePlus
        </button>
        <button
          className={`nc-filter ${activeFilter === 'github' ? 'active' : ''}`}
          onClick={() => setActiveFilter('github')}
        >
          <span className="filter-dot" style={{ background: SOURCE_COLORS.github }} />
          GitHub
        </button>
        <button
          className={`nc-filter ${activeFilter === 'helios' ? 'active' : ''}`}
          onClick={() => setActiveFilter('helios')}
        >
          <span className="filter-dot" style={{ background: SOURCE_COLORS.helios }} />
          Helios
        </button>
      </div>

      <div className="nc-divider" />

      {/* Notification list */}
      <div className="nc-list">
        {loading && <div className="nc-loading">Loading...</div>}
        {!loading && filteredNotifications.length === 0 && (
          <div className="nc-empty">No notifications</div>
        )}
        {filteredNotifications.map((notification) => (
          <div
            key={notification.id}
            className={`nc-item ${notification.is_read ? 'read' : 'unread'}`}
            onClick={() => handleMarkRead(notification.id)}
          >
            <div className="nc-item-header">
              <span
                className="nc-item-badge"
                style={{ background: SOURCE_COLORS[notification.source] }}
              >
                {SOURCE_LABELS[notification.source]}
              </span>
              <span className="nc-item-type">
                {TYPE_LABELS[notification.type] || notification.type}
              </span>
              {!notification.is_read && <span className="nc-unread-dot" />}
            </div>
            <div className="nc-item-title">{notification.title}</div>
            <div className="nc-item-body">{notification.body}</div>
            <div className="nc-item-time">
              {new Date(notification.created_at).toLocaleString()}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default NotificationCenter;
