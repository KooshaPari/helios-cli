import { useState, useRef, useEffect } from 'react';
import type { AgentLogEntry } from '../types';
import './AgentLogs.css';

interface AgentLogsProps {
  logs: AgentLogEntry[];
}

type LogLevel = 'all' | 'info' | 'warn' | 'error' | 'debug';

function AgentLogs({ logs }: AgentLogsProps) {
  const [filter, setFilter] = useState<LogLevel>('all');
  const [searchTerm, setSearchTerm] = useState('');
  const [isPaused, setIsPaused] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const filteredLogs = logs.filter((log) => {
    if (filter !== 'all' && log.level.toLowerCase() !== filter) {
      return false;
    }
    if (
      searchTerm &&
      !log.message.toLowerCase().includes(searchTerm.toLowerCase())
    ) {
      return false;
    }
    return true;
  });

  // Auto-scroll to bottom when new logs arrive (unless paused).
  useEffect(() => {
    if (!isPaused && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [filteredLogs.length, isPaused]);

  const levelClass = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'log-error';
      case 'warn':
        return 'log-warn';
      case 'debug':
        return 'log-debug';
      default:
        return 'log-info';
    }
  };

  return (
    <div className="agent-logs">
      <div className="logs-toolbar">
        <div className="log-filters">
          {(['all', 'info', 'warn', 'error', 'debug'] as LogLevel[]).map(
            (lvl) => (
              <button
                key={lvl}
                className={`filter-btn ${filter === lvl ? 'active' : ''}`}
                onClick={() => setFilter(lvl)}
              >
                {lvl}
              </button>
            ),
          )}
        </div>
        <input
          type="text"
          className="log-search"
          placeholder="Search logs..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
        <button
          className={`pause-btn ${isPaused ? 'paused' : ''}`}
          onClick={() => setIsPaused(!isPaused)}
          title={isPaused ? 'Resume auto-scroll' : 'Pause auto-scroll'}
        >
          {isPaused ? '|>' : '||'}
        </button>
      </div>

      <div className="logs-container" ref={scrollRef}>
        {filteredLogs.length === 0 && (
          <div className="logs-empty">
            {logs.length === 0 ? 'No logs yet' : 'No logs match filter'}
          </div>
        )}
        {filteredLogs.map((log, idx) => (
          <div key={idx} className={`log-line ${levelClass(log.level)}`}>
            {log.timestamp && (
              <span className="log-timestamp">{log.timestamp}</span>
            )}
            <span className={`log-level ${levelClass(log.level)}`}>
              [{log.level}]
            </span>
            <span className="log-message">{log.message}</span>
          </div>
        ))}
        {isPaused && (
          <div className="pause-indicator">
            Auto-scroll paused ({filteredLogs.length} entries)
          </div>
        )}
      </div>
    </div>
  );
}

export default AgentLogs;
