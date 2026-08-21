import { useState } from 'react';
import { useTasks } from '../hooks/useHelios';
import type { Task, TaskStatus } from '../types';
import './TaskQueue.css';

const COLUMNS: { key: TaskStatus; label: string; color: string }[] = [
  { key: 'pending', label: 'Pending', color: '#f59e0b' },
  { key: 'running', label: 'Running', color: '#6366f1' },
  { key: 'completed', label: 'Completed', color: '#10b981' },
  { key: 'failed', label: 'Failed', color: '#ef4444' },
];

function TaskQueue() {
  const { tasks, loading, error, create, rollback } = useTasks();
  const [showForm, setShowForm] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newAssignee, setNewAssignee] = useState('');

  const tasksByStatus = (status: TaskStatus) =>
    tasks.filter((t) => t.status === status);

  const handleCreate = async () => {
    if (!newTitle.trim()) return;
    await create(
      newTitle.trim(),
      newAssignee.trim() || undefined,
    );
    setNewTitle('');
    setNewAssignee('');
    setShowForm(false);
  };

  const formatTime = (iso: string | null) => {
    if (!iso) return '-';
    return new Date(iso).toLocaleTimeString();
  };

  const timeSince = (iso: string) => {
    const ms = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(ms / 60000);
    if (mins < 1) return '<1m';
    if (mins < 60) return `${mins}m`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ${mins % 60}m`;
  };

  return (
    <div className="task-queue">
      <div className="task-queue-header">
        <h2>Task Queue</h2>
        <div className="task-summary">
          <span className="summary-item">{tasks.length} total</span>
          <span className="summary-item pending">
            {tasksByStatus('pending').length} pending
          </span>
          <span className="summary-item running">
            {tasksByStatus('running').length} running
          </span>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => setShowForm(!showForm)}
        >
          + New Task
        </button>
      </div>

      {showForm && (
        <div className="task-form">
          <input
            type="text"
            placeholder="Task title..."
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
            autoFocus
          />
          <input
            type="text"
            placeholder="Assign to agent (optional)"
            value={newAssignee}
            onChange={(e) => setNewAssignee(e.target.value)}
          />
          <button className="btn btn-primary" onClick={handleCreate}>
            Create
          </button>
          <button
            className="btn btn-ghost"
            onClick={() => setShowForm(false)}
          >
            Cancel
          </button>
        </div>
      )}

      {error && <div className="error-banner">{error}</div>}

      <div className="kanban-board">
        {COLUMNS.map((col) => {
          const colTasks = tasksByStatus(col.key);
          return (
            <div key={col.key} className="kanban-column">
              <div className="column-header">
                <span
                  className="column-dot"
                  style={{ background: col.color }}
                />
                <span className="column-title">{col.label}</span>
                <span className="column-count">{colTasks.length}</span>
              </div>
              <div className="column-body">
                {colTasks.length === 0 && (
                  <div className="column-empty">No tasks</div>
                )}
                {colTasks.map((task) => (
                  <TaskCard
                    key={task.id}
                    task={task}
                    onRollback={() => rollback(task.id)}
                    formatTime={formatTime}
                    timeSince={timeSince}
                  />
                ))}
              </div>
            </div>
          );
        })}
      </div>

      {loading && <div className="loading-indicator">Refreshing...</div>}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Task card sub-component
// ---------------------------------------------------------------------------

interface TaskCardProps {
  task: Task;
  onRollback: () => void;
  formatTime: (iso: string | null) => string;
  timeSince: (iso: string) => string;
}

function TaskCard({ task, onRollback, formatTime, timeSince }: TaskCardProps) {
  return (
    <div className={`task-card status-${task.status}`}>
      <div className="task-card-title">{task.title}</div>
      {task.assignee_agent && (
        <div className="task-card-assignee">
          Agent: {task.assignee_agent}
        </div>
      )}
      <div className="task-card-times">
        <span>Created: {timeSince(task.created_at)}</span>
        {task.started_at && (
          <span>Started: {formatTime(task.started_at)}</span>
        )}
        {task.completed_at && (
          <span>Done: {formatTime(task.completed_at)}</span>
        )}
      </div>
      {task.error && (
        <div className="task-card-error">{task.error}</div>
      )}
      {task.result && (
        <div className="task-card-result">{task.result.summary}</div>
      )}
      {task.status === 'completed' && (
        <div className="task-card-actions">
          <button className="btn btn-ghost btn-sm" onClick={onRollback}>
            Rollback
          </button>
        </div>
      )}
    </div>
  );
}

export default TaskQueue;
