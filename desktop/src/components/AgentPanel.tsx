import { useState } from 'react';
import { useAgents, useAgentLogs } from '../hooks/useHelios';
import AgentLogs from './AgentLogs';
import AgentConfig from './AgentConfig';
import type { AgentInfo } from '../types';
import './AgentPanel.css';

function AgentPanel() {
  const { agents, loading, error, spawn, stop } = useAgents();
  const [selectedAgent, setSelectedAgent] = useState<AgentInfo | null>(null);
  const [showConfig, setShowConfig] = useState(false);
  const [spawnName, setSpawnName] = useState('');
  const [showSpawnForm, setShowSpawnForm] = useState(false);

  const { logs } = useAgentLogs(selectedAgent?.id ?? null);

  const runningCount = agents.filter((a) => a.status === 'running').length;
  const idleCount = agents.filter((a) => a.status === 'idle').length;
  const errorCount = agents.filter((a) => a.status === 'error').length;

  const handleSpawn = async () => {
    if (!spawnName.trim()) return;
    try {
      const agent = await spawn(spawnName.trim());
      setSelectedAgent(agent);
      setSpawnName('');
      setShowSpawnForm(false);
    } catch {
      // Error is surfaced via hook.
    }
  };

  const handleStop = async (id: string) => {
    await stop(id);
    if (selectedAgent?.id === id) {
      setSelectedAgent(null);
    }
  };

  const statusBadgeClass = (status: string) => {
    switch (status) {
      case 'running':
        return 'badge badge-running';
      case 'idle':
        return 'badge badge-idle';
      case 'error':
        return 'badge badge-error';
      default:
        return 'badge badge-stopped';
    }
  };

  return (
    <div className="agent-panel">
      <div className="agent-panel-header">
        <h2>Agent Management</h2>
        <div className="agent-stats">
          <span className="stat running">{runningCount} running</span>
          <span className="stat idle">{idleCount} idle</span>
          <span className="stat error">{errorCount} error</span>
        </div>
        <div className="agent-actions">
          <button
            className="btn btn-primary"
            onClick={() => setShowSpawnForm(!showSpawnForm)}
          >
            + New Agent
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => setShowConfig(!showConfig)}
          >
            Config
          </button>
        </div>
      </div>

      {showSpawnForm && (
        <div className="spawn-form">
          <input
            type="text"
            placeholder="Agent name..."
            value={spawnName}
            onChange={(e) => setSpawnName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSpawn()}
            autoFocus
          />
          <button className="btn btn-primary" onClick={handleSpawn}>
            Spawn
          </button>
          <button
            className="btn btn-ghost"
            onClick={() => setShowSpawnForm(false)}
          >
            Cancel
          </button>
        </div>
      )}

      {error && <div className="error-banner">{error}</div>}

      <div className="agent-panel-body">
        <div className="agent-list">
          {loading && agents.length === 0 && (
            <div className="empty-state">Loading agents...</div>
          )}
          {!loading && agents.length === 0 && (
            <div className="empty-state">
              <p>No agents running</p>
              <p className="empty-hint">Click "+ New Agent" to get started</p>
            </div>
          )}
          {agents.map((agent) => (
            <div
              key={agent.id}
              className={`agent-card ${
                selectedAgent?.id === agent.id ? 'selected' : ''
              }`}
              onClick={() => setSelectedAgent(agent)}
            >
              <div className="agent-card-header">
                <span className="agent-name">{agent.name}</span>
                <span className={statusBadgeClass(agent.status)}>
                  {agent.status}
                </span>
              </div>
              <div className="agent-card-meta">
                {agent.pid && <span className="meta-item">PID {agent.pid}</span>}
                {agent.repo && (
                  <span className="meta-item">{agent.repo}</span>
                )}
              </div>
              {agent.started_at && (
                <div className="agent-card-time">
                  Started: {new Date(agent.started_at).toLocaleTimeString()}
                </div>
              )}
              <div className="agent-card-actions">
                {agent.status === 'running' && (
                  <button
                    className="btn btn-danger btn-sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleStop(agent.id);
                    }}
                  >
                    Stop
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="agent-detail">
          {showConfig && <AgentConfig onClose={() => setShowConfig(false)} />}
          {selectedAgent && !showConfig && (
            <>
              <div className="detail-header">
                <h3>{selectedAgent.name}</h3>
                <span className={statusBadgeClass(selectedAgent.status)}>
                  {selectedAgent.status}
                </span>
              </div>
              <div className="detail-info">
                <div className="info-row">
                  <span className="label">ID:</span>
                  <span className="value">{selectedAgent.id}</span>
                </div>
                {selectedAgent.pid && (
                  <div className="info-row">
                    <span className="label">PID:</span>
                    <span className="value">{selectedAgent.pid}</span>
                  </div>
                )}
                {selectedAgent.repo && (
                  <div className="info-row">
                    <span className="label">Repo:</span>
                    <span className="value">{selectedAgent.repo}</span>
                  </div>
                )}
                {selectedAgent.started_at && (
                  <div className="info-row">
                    <span className="label">Started:</span>
                    <span className="value">
                      {new Date(selectedAgent.started_at).toLocaleString()}
                    </span>
                  </div>
                )}
                {selectedAgent.last_heartbeat && (
                  <div className="info-row">
                    <span className="label">Heartbeat:</span>
                    <span className="value">
                      {new Date(selectedAgent.last_heartbeat).toLocaleTimeString()}
                    </span>
                  </div>
                )}
              </div>
              <AgentLogs logs={logs} />
            </>
          )}
          {!selectedAgent && !showConfig && (
            <div className="empty-state">
              <p>Select an agent to view details and logs</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default AgentPanel;
