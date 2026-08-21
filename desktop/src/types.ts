// ---------------------------------------------------------------------------
// Agent types
// ---------------------------------------------------------------------------

export type AgentStatus = 'running' | 'idle' | 'error' | 'stopped';

export interface AgentInfo {
  id: string;
  name: string;
  status: AgentStatus;
  pid: number | null;
  repo: string | null;
  started_at: string | null;
  last_heartbeat: string | null;
  log_path: string | null;
}

export interface AgentLogEntry {
  timestamp: string;
  level: string;
  message: string;
}

// ---------------------------------------------------------------------------
// Task types
// ---------------------------------------------------------------------------

export type TaskStatus = 'pending' | 'running' | 'completed' | 'failed' | 'rolled_back';

export interface TaskResult {
  summary: string;
  artifacts: string[];
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  assignee_agent: string | null;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  result: TaskResult | null;
  error: string | null;
}

// ---------------------------------------------------------------------------
// Agent configuration (frontend-only for now)
// ---------------------------------------------------------------------------

export interface AgentConfig {
  model: string;
  tools: string[];
  fileAccessScope: string;
  maxTokens: number;
  temperature: number;
}

// ---------------------------------------------------------------------------
// Integration types (Tracera / AgilePlus)
// ---------------------------------------------------------------------------

export interface TraceraIssue {
  id: string;
  title: string;
  status: string;
  priority: string;
  assignee: string | null;
  linked_task_id: string | null;
}

export interface AgilePlusSpec {
  id: string;
  name: string;
  pillar: string;
  score: number;
  quality_gate: 'pass' | 'fail' | 'pending';
  linked_task_id: string | null;
}

// ---------------------------------------------------------------------------
// Milestone 3: Unified Search types
// ---------------------------------------------------------------------------

export type ToolSource = 'tracera' | 'agileplus' | 'github' | 'helios';

export type ResultType =
  | 'issue'
  | 'task'
  | 'spec'
  | 'pull_request'
  | 'workflow'
  | 'notification'
  | 'agent';

export interface SearchQuery {
  text: string;
  source_filter: ToolSource | null;
  type_filter: ResultType | null;
}

export interface SearchResult {
  source: ToolSource;
  type: ResultType;
  title: string;
  snippet: string;
  url: string | null;
  timestamp: string | null;
  score: number;
}

// ---------------------------------------------------------------------------
// Milestone 3: Notification types
// ---------------------------------------------------------------------------

export type NotificationSource = 'tracera' | 'agileplus' | 'github' | 'helios';

export type NotificationType =
  | 'tracera_issue'
  | 'agile_plus_gate_failure'
  | 'agent_error'
  | 'ci_status'
  | 'task_complete';

export interface Notification {
  id: string;
  source: NotificationSource;
  type: NotificationType;
  title: string;
  body: string;
  link: string | null;
  is_read: boolean;
  created_at: string;
}

export interface SourceCount {
  source: NotificationSource;
  total: number;
  unread: number;
}

export interface NotificationCounts {
  total: number;
  unread: number;
  by_source: SourceCount[];
}

// ---------------------------------------------------------------------------
// Navigation view types
// ---------------------------------------------------------------------------

export type AppView =
  | 'chat'
  | 'agents'
  | 'tasks'
  | 'tracera'
  | 'agileplus'
  | 'tracera-board'
  | 'agileplus-board'
  | 'notifications'
  | 'search';
