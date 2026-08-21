import { useEffect, useState } from 'react';
import { useAgents, useTasks } from '../hooks/useHelios';
import './StatusBar.css';

interface StatusBarProps {
  connectionStatus: 'connected' | 'disconnected' | 'connecting';
  messageCount: number;
}

function StatusBar({ connectionStatus, messageCount }: StatusBarProps) {
  const { agents } = useAgents();
  const { tasks } = useTasks();
  const [lastRefresh, setLastRefresh] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setLastRefresh(new Date()), 30000);
    return () => clearInterval(timer);
  }, []);

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

  const runningAgents = agents.filter((a) => a.status === 'running').length;
  const pendingTasks = tasks.filter((t) => t.status === 'pending').length;

  const formatTime = (d: Date) => d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className={`status-indicator ${getStatusColor()}`}></span>
        <span className="status-text">{getStatusText()}</span>
        <span className="status-divider">|</span>
        <span className="status-agents">
          <span className="status-dot running-dot"></span>
          {runningAgents} agent{runningAgents !== 1 ? 's' : ''}
        </span>
        <span className="status-divider">|</span>
        <span className="status-tasks">
          <span className="status-dot pending-dot"></span>
          {pendingTasks} pending task{pendingTasks !== 1 ? 's' : ''}
        </span>
      </div>
      <div className="status-center">
        <span className="message-count">
          {messageCount} message{messageCount !== 1 ? 's' : ''}
        </span>
      </div>
      <div className="status-right">
        <span className="last-refresh">Refreshed {formatTime(lastRefresh)}</span>
        <span className="version-info">v0.2.0</span>
      </div>
    </footer>
  );
}

export default StatusBar;