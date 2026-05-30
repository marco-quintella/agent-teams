import { writable, get } from 'svelte/store';
import { api, type AgentRun, type Task, type TeamMember, wsUrl } from '../api/client';

export type WsEvent =
  | { type: 'task_updated'; task: Task }
  | { type: 'agent_run_updated'; run: AgentRun }
  | { type: 'team_updated'; team_id: string };

const TEAM_ID_STORAGE_KEY = 'orchestrator.activeTeamId';

export const teamId = writable<string | null>(null);
export const tasks = writable<Task[]>([]);
export const members = writable<TeamMember[]>([]);
export const agentRuns = writable<AgentRun[]>([]);
export const connectionStatus = writable<'idle' | 'connecting' | 'open' | 'closed'>('idle');
export const lastError = writable<string | null>(null);
export const launched = writable(false);

let socket: WebSocket | null = null;

function memberIdsForCurrentTeam(): Set<string> {
  return new Set(get(members).map((m) => m.id));
}

export async function loadTeam(id: string) {
  const [taskList, memberList, runs] = await Promise.all([
    api.listTasks(id),
    api.listMembers(id),
    api.listAgentRuns(id),
  ]);
  teamId.set(id);
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(TEAM_ID_STORAGE_KEY, id);
  }
  tasks.set(taskList);
  members.set(memberList);
  agentRuns.set(runs);
  launched.set(runs.some((r) => r.status === 'running' || r.status === 'starting'));
}

/** Restore last team from localStorage after page reload (best-effort). */
export async function resumeTeamIfStored() {
  if (typeof localStorage === 'undefined') return;
  const stored = localStorage.getItem(TEAM_ID_STORAGE_KEY);
  if (!stored) return;
  try {
    connectWs();
    await loadTeam(stored);
  } catch {
    localStorage.removeItem(TEAM_ID_STORAGE_KEY);
    teamId.set(null);
    tasks.set([]);
    members.set([]);
    agentRuns.set([]);
    launched.set(false);
  }
}

export function connectWs() {
  if (socket?.readyState === WebSocket.OPEN) return;
  connectionStatus.set('connecting');
  socket = new WebSocket(wsUrl());

  socket.onopen = () => connectionStatus.set('open');
  socket.onclose = () => connectionStatus.set('closed');
  socket.onerror = () => lastError.set('WebSocket error');

  socket.onmessage = (msg) => {
    try {
      const event = JSON.parse(msg.data as string) as WsEvent;
      handleEvent(event);
    } catch {
      // ignore malformed frames
    }
  };
}

function handleEvent(event: WsEvent) {
  const activeTeamId = get(teamId);

  if (event.type === 'task_updated') {
    if (activeTeamId && event.task.team_id !== activeTeamId) return;
    tasks.update((list) => {
      const idx = list.findIndex((t) => t.id === event.task.id);
      if (idx === -1) return [...list, event.task];
      const next = [...list];
      next[idx] = event.task;
      return next;
    });
  } else if (event.type === 'agent_run_updated') {
    const allowed = memberIdsForCurrentTeam();
    if (allowed.size > 0 && !allowed.has(event.run.team_member_id)) return;
    agentRuns.update((list) => {
      const idx = list.findIndex((r) => r.id === event.run.id);
      if (idx === -1) return [...list, event.run];
      const next = [...list];
      next[idx] = event.run;
      return next;
    });
    launched.set(true);
  } else if (event.type === 'team_updated') {
    if (activeTeamId && event.team_id !== activeTeamId) return;
    if (activeTeamId) void loadTeam(activeTeamId);
  }
}

export function disconnectWs() {
  socket?.close();
  socket = null;
  connectionStatus.set('idle');
}
