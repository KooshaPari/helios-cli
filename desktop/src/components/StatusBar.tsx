import './StatusBar.css';

interface StatusBarProps {
  connectionStatus: 'connected' | 'disconnected' | 'connecting';
  messageCount: number;
}

function StatusBar({ connectionStatus, messageCount }: StatusBarProps) {
  const getStatusColor = () => {
    switch (connectionStatus) {
      case 'connected':
        return 'status-connected';
      case 'disconnected':
        return 'status-disconnected';
      case 'connecting':
        return 'status-connecting';
      default:
        return 'status-disconnected';
    }
  };

  const getStatusText = () => {
    switch (connectionStatus) {
      case 'connected':
        return 'Connected';
      case 'disconnected':
        return 'Disconnected';
      case 'connecting':
        return 'Connecting...';
      default:
        return 'Unknown';
    }
  };

  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className={`status-indicator ${getStatusColor()}`}></span>
        <span className="status-text">{getStatusText()}</span>
      </div>
      <div className="status-center">
        <span className="message-count">
          {messageCount} message{messageCount !== 1 ? 's' : ''}
        </span>
      </div>
      <div className="status-right">
        <span className="version-info">v0.1.0</span>
      </div>
    </footer>
  );
}

export default StatusBar;