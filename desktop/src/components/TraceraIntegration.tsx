import { useState } from 'react';
import type { TraceraIssue } from '../types';
import './TraceraIntegration.css';

// Simulated data - in production, this would come from Tracera via IPC.
const DEMO_ISSUES: TraceraIssue[] = [
  {
    id: 'TR-101',
    title: 'Implement rate limiter for API gateway',
    status: 'open',
    priority: 'high',
    assignee: null,
    linked_task_id: null,
  },
  {
    id: 'TR-102',
    title: 'Fix memory leak in WebSocket handler',
    status: 'in_progress',
    priority: 'critical',
    assignee: 'kooshapari',
    linked_task_id: null,
  },
  {
    id: 'TR-103',
    title: 'Add integration tests for auth module',
    status: 'open',
    priority: 'medium',
    assignee: null,
    linked_task_id: 'task-1723456789000',
  },
  {
    id: 'TR-104',
    title: 'Update CI pipeline for Rust 1.80',
    status: 'resolved',
    priority: 'low',
    assignee: 'kooshapari',
    linked_task_id: null,
  },
];

function TraceraIntegration() {
  const [issues, setIssues] = useState<TraceraIssue[]>(DEMO_ISSUES);
  const [filter, setFilter] = useState<'all' | 'open' | 'in_progress' | 'resolved'>('all');
  const [isRunning] = useState(true);

  const filteredIssues = issues.filter(
    (issue) => filter === 'all' || issue.status === filter,
  );

  const openCount = issues.filter((i) => i.status === 'open').length;
  const progressCount = issues.filter((i) => i.status === 'in_progress').length;
  const resolvedCount = issues.filter((i) => i.status === 'resolved').length;

  const priorityColor = (priority: string) => {
    switch (priority) {
      case 'critical':
        return 'priority-critical';
      case 'high':
        return 'priority-high';
      case 'medium':
        return 'priority-medium';
      default:
        return 'priority-low';
    }
  };

  const statusIcon = (status: string) => {
    switch (status) {
      case 'open':
        return 'o';
      case 'in_progress':
        return '~';
      case 'resolved':
        return 'v';
      default:
        return '?';
    }
  };

  const handleCreateIssue = () => {
    // Placeholder: in production, would call Tracera API.
    const newIssue: TraceraIssue = {
      id: `TR-${100 + issues.length + 1}`,
      title: 'New issue (click to edit)',
      status: 'open',
      priority: 'medium',
      assignee: null,
      linked_task_id: null,
    };
    setIssues((prev) => [...prev, newIssue]);
  };

  return (
    <div className="tracera-panel">
      <div className="integration-header">
        <div className="integration-title">
          <h2>Tracera Issues</h2>
          <span
            className={`connection-dot ${isRunning ? 'connected' : 'disconnected'}`}
          />
          <span className="connection-label">
            {isRunning ? 'Connected' : 'Disconnected'}
          </span>
        </div>
        <div className="integration-stats">
          <span className="stat">{openCount} open</span>
          <span className="stat progress">{progressCount} in progress</span>
          <span className="stat resolved">{resolvedCount} resolved</span>
        </div>
        <button className="btn btn-primary" onClick={handleCreateIssue}>
          + New Issue
        </button>
      </div>

      <div className="filter-bar">
        {(['all', 'open', 'in_progress', 'resolved'] as const).map((f) => (
          <button
            key={f}
            className={`filter-btn ${filter === f ? 'active' : ''}`}
            onClick={() => setFilter(f)}
          >
            {f === 'in_progress' ? 'In Progress' : f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        ))}
      </div>

      <div className="issues-list">
        {filteredIssues.length === 0 && (
          <div className="empty-state">No issues match filter</div>
        )}
        {filteredIssues.map((issue) => (
          <div key={issue.id} className="issue-card">
            <div className="issue-header">
              <span className="issue-id">{issue.id}</span>
              <span className={`issue-status status-${issue.status}`}>
                {statusIcon(issue.status)} {issue.status.replace('_', ' ')}
              </span>
              <span className={`issue-priority ${priorityColor(issue.priority)}`}>
                {issue.priority}
              </span>
            </div>
            <div className="issue-title">{issue.title}</div>
            <div className="issue-meta">
              {issue.assignee && (
                <span className="meta-tag">Assigned: {issue.assignee}</span>
              )}
              {issue.linked_task_id && (
                <span className="meta-tag linked">
                  Linked to: {issue.linked_task_id}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default TraceraIntegration;
