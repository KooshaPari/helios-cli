import { useEffect, useState } from 'react';
import { appWindow } from '@tauri-apps/api/window';
import './Header.css';

interface HeaderProps {
  title: string;
  isProcessing: boolean;
  onOpenCommandPalette: () => void;
  onOpenNotifications?: () => void;
  notificationCount?: number;
}

function Header({ title, isProcessing, onOpenCommandPalette, onOpenNotifications, notificationCount }: HeaderProps) {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    const checkMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };
    checkMaximized();

    const unlisten = appWindow.onResizedEvent(() => {
      checkMaximized();
    });

    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, []);

  const handleMinimize = async () => {
    await appWindow.minimize();
  };

  const handleMaximize = async () => {
    await appWindow.toggleMaximize();
  };

  const handleClose = async () => {
    await appWindow.close();
  };

  return (
    <header className="header" data-tauri-drag-region>
      <div className="header-left" data-tauri-drag-region>
        {isProcessing && (
          <div className="processing-indicator">
            <div className="spinner"></div>
          </div>
        )}
        <h2 className="header-title" data-tauri-drag-region>
          {title}
        </h2>
      </div>

      <div className="header-center">
        <button
          className="command-palette-trigger"
          onClick={onOpenCommandPalette}
          title="Unified Search (Ctrl+K)"
        >
          <span className="search-icon">{'\u2318'}</span>
          <span className="command-hint">Search across tools...</span>
          <kbd className="shortcut">{'\u2318'}K</kbd>
        </button>
      </div>

      <div className="header-right">
        <div className="header-actions">
          <button
            className="notification-bell-btn"
            onClick={onOpenNotifications}
            title="Notifications (Ctrl+J)"
          >
            <span className="bell-icon">{'\u{1f514}'}</span>
            {(notificationCount || 0) > 0 && (
              <span className="notification-count-badge">
                {notificationCount! > 99 ? '99+' : notificationCount}
              </span>
            )}
          </button>
        </div>
        <div className="window-controls">
          <button
            className="control-btn minimize"
            onClick={handleMinimize}
            aria-label="Minimize"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M2.5 6H9.5"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinecap="round"
              />
            </svg>
          </button>
          <button
            className="control-btn maximize"
            onClick={handleMaximize}
            aria-label={isMaximized ? 'Restore' : 'Maximize'}
          >
            {isMaximized ? (
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path
                  d="M3 4.5V2.5C3 2.22 3.22 2 3.5 2H6.5M6.5 10H9.5C9.78 10 10 9.78 10 9.5V6.5M3.5 10C3.22 10 3 9.78 3 9.5V6.5M9.5 2H6.5V4.5"
                  stroke="currentColor"
                  strokeWidth="1.2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            ) : (
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path
                  d="M3 2.5H9.5V9M2.5 3V9C2.5 9.28 2.72 9.5 3 9.5H9"
                  stroke="currentColor"
                  strokeWidth="1.2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            )}
          </button>
          <button
            className="control-btn close"
            onClick={handleClose}
            aria-label="Close"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M3 3L9 9M9 3L3 9"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>
      </div>
    </header>
  );
}

export default Header;
