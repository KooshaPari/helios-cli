import { Conversation } from '../App';
import type { AppView } from '../types';
import './Sidebar.css';

interface SidebarProps {
  conversations: Conversation[];
  activeConversationId: string | null;
  onSelectConversation: (id: string) => void;
  onNewConversation: () => void;
  onDeleteConversation: (id: string) => void;
  activeView: AppView;
  onNavigate: (view: AppView) => void;
  agentCount: number;
  taskCount: number;
}

function Sidebar({
  conversations,
  activeConversationId,
  onSelectConversation,
  onNewConversation,
  onDeleteConversation,
  activeView,
  onNavigate,
  agentCount,
  taskCount,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1 className="app-title">Helios CLI</h1>
        <button
          className="new-conversation-btn"
          onClick={onNewConversation}
          title="New Conversation (Ctrl+N)"
        >
          <span className="btn-icon">+</span>
        </button>
      </div>

      <nav className="sidebar-nav">
        <button
          className={`nav-item ${activeView === 'chat' ? 'active' : ''}`}
          onClick={() => onNavigate('chat')}
        >
          <span className="nav-icon">#</span>
          <span className="nav-label">Chat</span>
        </button>
        <button
          className={`nav-item ${activeView === 'agents' ? 'active' : ''}`}
          onClick={() => onNavigate('agents')}
        >
          <span className="nav-icon">{'>*'}</span>
          <span className="nav-label">Agents</span>
          {agentCount > 0 && (
            <span className="nav-badge agents-badge">{agentCount}</span>
          )}
        </button>
        <button
          className={`nav-item ${activeView === 'tasks' ? 'active' : ''}`}
          onClick={() => onNavigate('tasks')}
        >
          <span className="nav-icon">[]</span>
          <span className="nav-label">Tasks</span>
          {taskCount > 0 && (
            <span className="nav-badge tasks-badge">{taskCount}</span>
          )}
        </button>
        <button
          className={`nav-item ${activeView === 'tracera' ? 'active' : ''}`}
          onClick={() => onNavigate('tracera')}
        >
          <span className="nav-icon">T</span>
          <span className="nav-label">Tracera</span>
        </button>
        <button
          className={`nav-item ${activeView === 'agileplus' ? 'active' : ''}`}
          onClick={() => onNavigate('agileplus')}
        >
          <span className="nav-icon">A+</span>
          <span className="nav-label">AgilePlus</span>
        </button>
      </nav>

      <div className="sidebar-section-header">Conversations</div>

      <div className="conversations-list">
        {conversations.length === 0 ? (
          <div className="empty-state">
            <p>No conversations yet</p>
            <p className="empty-hint">Click the + button to start a new conversation</p>
          </div>
        ) : (
          conversations.map((conversation) => (
            <div
              key={conversation.id}
              className={`conversation-item ${
                conversation.id === activeConversationId ? 'active' : ''
              }`}
              onClick={() => onSelectConversation(conversation.id)}
            >
              <div className="conversation-info">
                <div className="conversation-title">{conversation.title}</div>
                <div className="conversation-meta">
                  {conversation.messages.length} message
                  {conversation.messages.length !== 1 ? 's' : ''} •{' '}
                  {formatRelativeTime(conversation.createdAt)}
                </div>
              </div>
              <button
                className="delete-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteConversation(conversation.id);
                }}
                title="Delete conversation"
              >
                ×
              </button>
            </div>
          ))
        )}
      </div>

      <div className="sidebar-footer">
        <div className="version-info">Helios CLI v0.1.0</div>
        <div className="shortcut-hint">Ctrl+K for commands</div>
      </div>
    </aside>
  );
}

function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${diffDays}d ago`;
}

export default Sidebar;