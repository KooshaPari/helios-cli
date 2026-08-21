import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { SearchResult, SearchQuery } from '../types';
import './EmbeddedPanel.css';

function EmbeddedTracera() {
  const [issues, setIssues] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [viewMode, setViewMode] = useState<'webview' | 'fallback'>('fallback');

  const fetchTraceraIssues = useCallback(async () => {
    setLoading(true);
    try {
      const query: SearchQuery = {
        text: '',
        source_filter: 'tracera',
        type_filter: 'issue',
      };
      const results = await invoke<SearchResult[]>('unified_search_cmd', { query });
      setIssues(results);
    } catch (err) {
      console.error('Failed to fetch Tracera issues:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTraceraIssues();
  }, [fetchTraceraIssues]);

  const handleRefresh = () => {
    fetchTraceraIssues();
  };

  const handleOpenInTracera = () => {
    // Would open Tracera in external browser or webview.
    window.open('http://localhost:3000', '_blank');
  };

  return (
    <div className="embedded-panel embedded-tracera">
      <div className="embedded-toolbar">
        <div className="toolbar-left">
          <h3 className="toolbar-title">Tracera Board</h3>
          <div className="toolbar-view-toggle">
            <button
              className={`toggle-btn ${viewMode === 'webview' ? 'active' : ''}`}
              onClick={() => setViewMode('webview')}
            >
              Live View
            </button>
            <button
              className={`toggle-btn ${viewMode === 'fallback' ? 'active' : ''}`}
              onClick={() => setViewMode('fallback')}
            >
              Issue List
            </button>
          </div>
        </div>
        <div className="toolbar-actions">
          <button className="toolbar-btn" onClick={handleRefresh} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
          </button>
          <button className="toolbar-btn primary" onClick={handleOpenInTracera}>
            Open in Tracera
          </button>
        </div>
      </div>

      <div className="embedded-content">
        {viewMode === 'webview' ? (
          <div className="webview-fallback">
            <div className="webview-placeholder">
              <div className="webview-icon">T</div>
              <h4>Tracera Live Board</h4>
              <p>
                To enable the live view, ensure Tracera is running locally at{' '}
                <code>http://localhost:3000</code>
              </p>
              <button className="toolbar-btn primary" onClick={() => setViewMode('fallback')}>
                View Issue List Instead
              </button>
            </div>
          </div>
        ) : (
          <div className="tracera-issues-list">
            {loading && <div className="embedded-loading">Loading issues...</div>}
            {!loading && issues.length === 0 && (
              <div className="embedded-empty">
                <div className="empty-icon">T</div>
                <h4>No Tracera Issues Found</h4>
                <p>
                  Tracera issues will appear here when the Tracera database is
                  configured. Add the Tracera SQLite database path in settings.
                </p>
              </div>
            )}
            {issues.map((issue, index) => (
              <div key={index} className="tracera-issue-card">
                <div className="issue-header">
                  <span className="issue-id">{issue.title}</span>
                  <span
                    className={`issue-priority priority-${issue.snippet.includes('[open]') ? 'high' : 'normal'}`}
                  >
                    {issue.snippet.match(/^\[([^\]]+)\]/)?.[1] || 'open'}
                  </span>
                </div>
                <div className="issue-snippet">{issue.snippet}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default EmbeddedTracera;
