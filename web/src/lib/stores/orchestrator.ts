import { writable, get } from 'svelte/store';
import { api, type AgentRun, type Task, type TeamMember, wsUrl } from '../api/client';

export type WsEvent =
  | { type: 'task_updated'; task: Task }
  | { type: 'agent_run_updated'; run: AgentRun }
  | { type: 'team_updated'; team_id: string };

export const teamId = writable<string | null>(null);
export const tasks = writable<Task[]>([]);
export const members = writable<TeamMember[]>([]);
export const agentRuns = writable<AgentRun[]>([]);
export const connectionStatus = writable<'idle' | 'connecting' | 'open' | 'closed'>('idle');
export const lastError = writable<string | null>(null);
export const launched = writable(false);

let socket: WebSocket | null = null;

export async function loadTeam(id: string) {
  teamId.set(id);
  const [taskList, memberList, runs] = await Promise.all([
    api.listTasks(id),
    api.listMembers(id),
    api.listAgentRuns(id),
  ]);
  tasks.set(taskList);
  members.set(memberList);
  agentRuns.set(runs);
  launched.set(runs.some((r) => r.status === 'running' || r.status === 'starting'));
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
  if (event.type === 'task_updated') {
    tasks.update((list) => {
      const idx = list.findIndex((t) => t.id === event.task.id);
      if (idx === -1) return [...list, event.task];
      const next = [...list];
      next[idx] = event.task;
      return next;
    });
  } else if (event.type === 'agent_run_updated') {
    agentRuns.update((list) => {
      const idx = list.findIndex((r) => r.id === event.run.id);
      if (idx === -1) return [...list, event.run];
      const next = [...list];
      next[idx] = event.run;
      return next;
    });
    launched.set(true);
  } else if (event.type === 'team_updated') {
    const id = get(teamId);
    if (id) void loadTeam(id);
  }
}

export function disconnectWs() {
  socket?.close();
  socket = null;
  connectionStatus.set('idle');
}
