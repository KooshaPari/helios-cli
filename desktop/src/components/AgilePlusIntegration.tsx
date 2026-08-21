import { useState } from 'react';
import type { AgilePlusSpec } from '../types';
import './AgilePlusIntegration.css';

// Simulated data - in production, this would come from AgilePlus via IPC.
const DEMO_SPECS: AgilePlusSpec[] = [
  {
    id: 'SP-201',
    name: 'API Rate Limiter Spec',
    pillar: 'Performance',
    score: 92,
    quality_gate: 'pass',
    linked_task_id: null,
  },
  {
    id: 'SP-202',
    name: 'WebSocket Handler Spec',
    pillar: 'Reliability',
    score: 78,
    quality_gate: 'pending',
    linked_task_id: 'task-1723456789000',
  },
  {
    id: 'SP-203',
    name: 'Auth Module Integration Tests',
    pillar: 'Testing',
    score: 45,
    quality_gate: 'fail',
    linked_task_id: null,
  },
  {
    id: 'SP-204',
    name: 'CI Pipeline Migration',
    pillar: 'DevOps',
    score: 100,
    quality_gate: 'pass',
    linked_task_id: null,
  },
  {
    id: 'SP-205',
    name: 'Dashboard Performance',
    pillar: 'Performance',
    score: 67,
    quality_gate: 'pending',
    linked_task_id: null,
  },
];

function AgilePlusIntegration() {
  const [specs] = useState<AgilePlusSpec[]>(DEMO_SPECS);
  const [isRunning] = useState(true);

  const passCount = specs.filter((s) => s.quality_gate === 'pass').length;
  const failCount = specs.filter((s) => s.quality_gate === 'fail').length;
  const pendingCount = specs.filter((s) => s.quality_gate === 'pending').length;

  const avgScore = Math.round(
    specs.reduce((sum, s) => sum + s.score, 0) / specs.length,
  );

  const scoreColor = (score: number) => {
    if (score >= 80) return 'score-good';
    if (score >= 60) return 'score-warn';
    return 'score-bad';
  };

  const gateIcon = (gate: string) => {
    switch (gate) {
      case 'pass':
        return 'v';
      case 'fail':
        return 'x';
      default:
        return '~';
    }
  };

  return (
    <div className="agileplus-panel">
      <div className="integration-header">
        <div className="integration-title">
          <h2>AgilePlus Specs</h2>
          <span
            className={`connection-dot ${isRunning ? 'connected' : 'disconnected'}`}
          />
          <span className="connection-label">
            {isRunning ? 'Connected' : 'Disconnected'}
          </span>
        </div>
        <div className="integration-stats">
          <span className="stat">Avg: {avgScore}</span>
          <span className="stat gate-pass">{passCount} passing</span>
          <span className="stat gate-fail">{failCount} failing</span>
          <span className="stat gate-pending">{pendingCount} pending</span>
        </div>
      </div>

      {/* Pillar scores overview */}
      <div className="pillar-overview">
        {getPillarScores(specs).map((p) => (
          <div key={p.name} className="pillar-card">
            <div className="pillar-name">{p.name}</div>
            <div className={`pillar-score ${scoreColor(p.avgScore)}`}>
              {p.avgScore}
            </div>
            <div className="pillar-count">
              {p.count} spec{p.count !== 1 ? 's' : ''}
            </div>
          </div>
        ))}
      </div>

      {/* Spec list */}
      <div className="specs-list">
        {specs.map((spec) => (
          <div key={spec.id} className="spec-card">
            <div className="spec-header">
              <span className="spec-id">{spec.id}</span>
              <span className="spec-pillar">{spec.pillar}</span>
              <span
                className={`spec-gate gate-${spec.quality_gate}`}
              >
                {gateIcon(spec.quality_gate)} {spec.quality_gate}
              </span>
              <span
                className={`spec-score ${scoreColor(spec.score)}`}
              >
                {spec.score}
              </span>
            </div>
            <div className="spec-name">{spec.name}</div>
            <div className="spec-bar-track">
              <div
                className={`spec-bar-fill ${scoreColor(spec.score)}`}
                style={{ width: `${spec.score}%` }}
              />
            </div>
            {spec.linked_task_id && (
              <div className="spec-link">Linked to: {spec.linked_task_id}</div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getPillarScores(specs: AgilePlusSpec[]) {
  const map = new Map<string, { sum: number; count: number }>();
  for (const s of specs) {
    const entry = map.get(s.pillar) || { sum: 0, count: 0 };
    entry.sum += s.score;
    entry.count += 1;
    map.set(s.pillar, entry);
  }
  return Array.from(map.entries())
    .map(([name, { sum, count }]) => ({
      name,
      avgScore: Math.round(sum / count),
      count,
    }))
    .sort((a, b) => b.avgScore - a.avgScore);
}

export default AgilePlusIntegration;
