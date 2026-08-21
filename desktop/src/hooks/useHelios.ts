import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type {
  AgentInfo,
  AgentLogEntry,
  Task,
  TaskStatus,
  SearchResult,
  SearchQuery,
  Notification,
  NotificationCounts,
} from '../types';

// ---------------------------------------------------------------------------
// Agent hooks
// ---------------------------------------------------------------------------

export function useAgents() {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchAgents = useCallback(async () => {
    try {
      const result = await invoke<AgentInfo[]>('list_agents');
      setAgents(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    await fetchAgents();
    setLoading(false);
  }, [fetchAgents]);

  const spawn = useCallback(
    async (name: string, repo?: string, command?: string) => {
      try {
        const agent = await invoke<AgentInfo>('spawn_agent', {
          name,
          repo: repo || null,
          command: command || null,
          args: null,
        });
        await fetchAgents();
        return agent;
      } catch (err) {
        setError(String(err));
        throw err;
      }
    },
    [fetchAgents],
  );

  const stop = useCallback(
    async (id: string) => {
      try {
        await invoke<AgentInfo>('stop_agent', { id });
        await fetchAgents();
      } catch (err) {
        setError(String(err));
        throw err;
      }
    },
    [fetchAgents],
  );

  // Auto-refresh every 3 seconds.
  useEffect(() => {
    fetchAgents();
    intervalRef.current = setInterval(fetchAgents, 3000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchAgents]);

  return { agents, loading, error, refresh, spawn, stop };
}

// ---------------------------------------------------------------------------
// Agent Logs hook
// ---------------------------------------------------------------------------

export function useAgentLogs(agentId: string | null, tail: number = 200) {
  const [logs, setLogs] = useState<AgentLogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchLogs = useCallback(async () => {
    if (!agentId) {
      setLogs([]);
      return;
    }
    try {
      const result = await invoke<AgentLogEntry[]>('get_agent_logs', {
        id: agentId,
        tail,
      });
      setLogs(result);
    } catch {
      // Agent may have been removed; keep existing logs.
    }
  }, [agentId, tail]);

  useEffect(() => {
    fetchLogs();
    if (agentId) {
      intervalRef.current = setInterval(fetchLogs, 2000);
    }
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchLogs, agentId]);

  return { logs, loading, refresh: fetchLogs };
}

// ---------------------------------------------------------------------------
// Task hooks
// ---------------------------------------------------------------------------

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchTasks = useCallback(async () => {
    try {
      const result = await invoke<Task[]>('list_tasks');
      setTasks(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    await fetchTasks();
    setLoading(false);
  }, [fetchTasks]);

  const create = useCallback(
    async (title: string, assigneeAgent?: string) => {
      try {
        const task = await invoke<Task>('create_task', {
          title,
          assigneeAgent: assigneeAgent || null,
        });
        await fetchTasks();
        return task;
      } catch (err) {
        setError(String(err));
        throw err;
      }
    },
    [fetchTasks],
  );

  const rollback = useCallback(
    async (taskId: string) => {
      try {
        await invoke<Task>('rollback_task', { taskId });
        await fetchTasks();
      } catch (err) {
        setError(String(err));
        throw err;
      }
    },
    [fetchTasks],
  );

  useEffect(() => {
    fetchTasks();
    intervalRef.current = setInterval(fetchTasks, 5000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchTasks]);

  return { tasks, loading, error, refresh, create, rollback };
}

// ---------------------------------------------------------------------------
// Milestone 3: Unified Search hook
// ---------------------------------------------------------------------------

export function useUnifiedSearch() {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const search = useCallback(async (query: SearchQuery) => {
    if (!query.text.trim()) {
      setResults([]);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const searchResults = await invoke<SearchResult[]>('unified_search_cmd', {
        query,
      });
      setResults(searchResults);
    } catch (err) {
      setError(String(err));
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const clear = useCallback(() => {
    setResults([]);
    setError(null);
  }, []);

  return { results, loading, error, search, clear };
}

// ---------------------------------------------------------------------------
// Milestone 3: Notifications hook
// ---------------------------------------------------------------------------

export function useNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [counts, setCounts] = useState<NotificationCounts | null>(null);
  const [loading, setLoading] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchNotifications = useCallback(async () => {
    try {
      const [notifs, cts] = await Promise.all([
        invoke<Notification[]>('list_notifications', { limit: 100 }),
        invoke<NotificationCounts>('get_notification_counts'),
      ]);
      setNotifications(notifs);
      setCounts(cts);
    } catch {
      // Silently fail on auto-refresh.
    }
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    await fetchNotifications();
    setLoading(false);
  }, [fetchNotifications]);

  const markRead = useCallback(async (notificationId: string) => {
    try {
      await invoke('mark_notification_read', { notificationId });
      setNotifications((prev) =>
        prev.map((n) => (n.id === notificationId ? { ...n, is_read: true } : n)),
      );
      // Refresh counts.
      const cts = await invoke<NotificationCounts>('get_notification_counts');
      setCounts(cts);
    } catch (err) {
      throw err;
    }
  }, []);

  const markAllRead = useCallback(async () => {
    try {
      await invoke('mark_all_notifications_read');
      setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
      const cts = await invoke<NotificationCounts>('get_notification_counts');
      setCounts(cts);
    } catch (err) {
      throw err;
    }
  }, []);

  // Auto-refresh every 10 seconds.
  useEffect(() => {
    fetchNotifications();
    intervalRef.current = setInterval(fetchNotifications, 10000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchNotifications]);

  return {
    notifications,
    counts,
    loading,
    refresh,
    markRead,
    markAllRead,
  };
}
