import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { SearchResult, SearchQuery } from '../types';
import './EmbeddedPanel.css';

interface PillarScore {
  name: string;
  score: number;
  status: 'pass' | 'fail' | 'pending';
}

function EmbeddedAgilePlus() {
  const [specs, setSpecs] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [pillarScores] = useState<PillarScore[]>([
    { name: 'Architecture', score: 82, status: 'pass' },
    { name: 'Testing', score: 65, status: 'pending' },
    { name: 'Security', score: 90, status: 'pass' },
    { name: 'Performance', score: 55, status: 'fail' },
    { name: 'Documentation', score: 70, status: 'pass' },
  ]);

  const fetchSpecs = useCallback(async () => {
    setLoading(true);
    try {
      const query: SearchQuery = {
        text: '',
        source_filter: 'agileplus',
        type_filter: 'spec',
      };
      const results = await invoke<SearchResult[]>('unified_search_cmd', { query });
      setSpecs(results);
    } catch (err) {
      console.error('Failed to fetch AgilePlus specs:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSpecs();
  }, [fetchSpecs]);

  const handleRefresh = () => {
    fetchSpecs();
  };

  const handleOpenInAgilePlus = () => {
    window.open('http://localhost:3001', '_blank');
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pass': return '#10b981';
      case 'fail': return '#ef4444';
      case 'pending': return '#f59e0b';
      default: return '#6b7280';
    }
  };

  return (
    <div className="embedded-panel embedded-agileplus">
      <div className="embedded-toolbar">
        <div className="toolbar-left">
          <h3 className="toolbar-title">AgilePlus Scorecard</h3>
        </div>
        <div className="toolbar-actions">
          <button className="toolbar-btn" onClick={handleRefresh} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
          </button>
          <button className="toolbar-btn primary" onClick={handleOpenInAgilePlus}>
            Open in AgilePlus
          </button>
        </div>
      </div>

      <div className="embedded-content">
        {/* Pillar scorecard summary */}
        <div className="pillar-scorecard">
          <h4 className="scorecard-title">Pillar Scores</h4>
          <div className="pillar-grid">
            {pillarScores.map((pillar) => (
              <div key={pillar.name} className="pillar-card">
                <div className="pillar-header">
                  <span className="pillar-name">{pillar.name}</span>
                  <span
                    className="pillar-status"
                    style={{ color: getStatusColor(pillar.status) }}
                  >
                    {pillar.status.toUpperCase()}
                  </span>
                </div>
                <div className="pillar-score-bar">
                  <div
                    className="pillar-score-fill"
                    style={{
                      width: `${pillar.score}%`,
                      background: getStatusColor(pillar.status),
                    }}
                  />
                </div>
                <div className="pillar-score-value">{pillar.score}/100</div>
              </div>
            ))}
          </div>
        </div>

        {/* Spec list */}
        <div className="specs-section">
          <h4 className="specs-title">Specs</h4>
          {loading && <div className="embedded-loading">Loading specs...</div>}
          {!loading && specs.length === 0 && (
            <div className="embedded-empty">
              <div className="empty-icon">A+</div>
              <h4>No AgilePlus Specs Found</h4>
              <p>
                Spec files will appear here when an AgilePlus project directory
                is configured in settings.
              </p>
            </div>
          )}
          <div className="specs-list">
            {specs.map((spec, index) => (
              <div key={index} className="spec-card">
                <div className="spec-header">
                  <span className="spec-name">{spec.title}</span>
                  <span className="spec-score">Score: {spec.score}</span>
                </div>
                <div className="spec-snippet">{spec.snippet}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default EmbeddedAgilePlus;
