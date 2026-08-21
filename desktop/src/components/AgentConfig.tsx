import { useState } from 'react';
import type { AgentConfig as AgentConfigType } from '../types';
import './AgentConfig.css';

interface AgentConfigProps {
  onClose: () => void;
}

const AVAILABLE_MODELS = [
  'gpt-4o',
  'gpt-4o-mini',
  'claude-sonnet-4-20250514',
  'claude-haiku-3-20240307',
  'gemini-2.0-flash',
  'gemini-2.5-pro',
  'deepseek-chat',
];

const AVAILABLE_TOOLS = [
  'file_read',
  'file_write',
  'shell_exec',
  'git_ops',
  'web_search',
  'api_call',
  'test_run',
  'lint_check',
];

function AgentConfig({ onClose }: AgentConfigProps) {
  const [config, setConfig] = useState<AgentConfigType>({
    model: 'gpt-4o',
    tools: ['file_read', 'git_ops'],
    fileAccessScope: 'repo',
    maxTokens: 4096,
    temperature: 0.7,
  });

  const [saved, setSaved] = useState(false);

  const toggleTool = (tool: string) => {
    setConfig((prev) => ({
      ...prev,
      tools: prev.tools.includes(tool)
        ? prev.tools.filter((t) => t !== tool)
        : [...prev.tools, tool],
    }));
  };

  const handleSave = () => {
    // In a full implementation, this would persist to the backend.
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleReset = () => {
    setConfig({
      model: 'gpt-4o',
      tools: ['file_read', 'git_ops'],
      fileAccessScope: 'repo',
      maxTokens: 4096,
      temperature: 0.7,
    });
  };

  return (
    <div className="agent-config">
      <div className="config-header">
        <h3>Agent Configuration</h3>
        <button className="btn-close" onClick={onClose}>
          x
        </button>
      </div>

      <div className="config-section">
        <label className="config-label">Model</label>
        <select
          className="config-select"
          value={config.model}
          onChange={(e) =>
            setConfig((prev) => ({ ...prev, model: e.target.value }))
          }
        >
          {AVAILABLE_MODELS.map((model) => (
            <option key={model} value={model}>
              {model}
            </option>
          ))}
        </select>
      </div>

      <div className="config-section">
        <label className="config-label">Tool Permissions</label>
        <div className="tool-grid">
          {AVAILABLE_TOOLS.map((tool) => (
            <label key={tool} className="tool-checkbox">
              <input
                type="checkbox"
                checked={config.tools.includes(tool)}
                onChange={() => toggleTool(tool)}
              />
              <span className="tool-name">{tool}</span>
            </label>
          ))}
        </div>
      </div>

      <div className="config-section">
        <label className="config-label">File Access Scope</label>
        <select
          className="config-select"
          value={config.fileAccessScope}
          onChange={(e) =>
            setConfig((prev) => ({
              ...prev,
              fileAccessScope: e.target.value,
            }))
          }
        >
          <option value="repo">Current Repository</option>
          <option value="workspace">Workspace</option>
          <option value="global">Global (Full Access)</option>
          <option value="read-only">Read Only</option>
        </select>
      </div>

      <div className="config-section">
        <label className="config-label">
          Max Tokens: {config.maxTokens.toLocaleString()}
        </label>
        <input
          type="range"
          className="config-slider"
          min={256}
          max={16384}
          step={256}
          value={config.maxTokens}
          onChange={(e) =>
            setConfig((prev) => ({
              ...prev,
              maxTokens: parseInt(e.target.value),
            }))
          }
        />
        <div className="slider-labels">
          <span>256</span>
          <span>16K</span>
        </div>
      </div>

      <div className="config-section">
        <label className="config-label">
          Temperature: {config.temperature.toFixed(1)}
        </label>
        <input
          type="range"
          className="config-slider"
          min={0}
          max={2}
          step={0.1}
          value={config.temperature}
          onChange={(e) =>
            setConfig((prev) => ({
              ...prev,
              temperature: parseFloat(e.target.value),
            }))
          }
        />
        <div className="slider-labels">
          <span>0.0 (Precise)</span>
          <span>2.0 (Creative)</span>
        </div>
      </div>

      <div className="config-actions">
        <button className="btn btn-primary" onClick={handleSave}>
          {saved ? 'Saved!' : 'Save Config'}
        </button>
        <button className="btn btn-secondary" onClick={handleReset}>
          Reset
        </button>
      </div>
    </div>
  );
}

export default AgentConfig;
