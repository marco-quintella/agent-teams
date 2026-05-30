const apiBase = (import.meta.env.VITE_API_BASE ?? '').replace(/\/$/, '');

export type TaskStatus = 'backlog' | 'in_progress' | 'review' | 'done';

export interface Task {
  id: string;
  team_id: string;
  title: string;
  description: string;
  status: TaskStatus;
  assignee_member_id: string | null;
  created_by: 'human' | 'agent';
  created_at: string;
  updated_at: string;
}

export interface TeamMember {
  id: string;
  team_id: string;
  name: string;
  role: 'lead' | 'worker';
  role_prompt: string;
  created_at: string;
}

export interface AgentRun {
  id: string;
  team_member_id: string;
  status: string;
  last_output_snippet: string;
  pid: number | null;
  started_at: string | null;
  stopped_at: string | null;
  updated_at: string;
}

export interface Project {
  id: string;
  root_path: string;
  created_at: string;
}

export interface Team {
  id: string;
  project_id: string;
  name: string;
  provisioning_prompt: string;
  created_at: string;
}

export interface DoctorResponse {
  orchestrator_version: string;
  cli: { found: boolean; version: string | null };
  credentials: { mode: string; ready: boolean; hint: string };
  model: { default_model: string | null; hint: string };
}

export interface ClaudeSettingsView {
  credential_mode: string;
  api_key_masked: string | null;
  api_base_url: string | null;
  default_model: string | null;
  updated_at: string;
}

export interface TeamSummary {
  id: string;
  name: string;
  project_root_path: string;
  created_at: string;
  status: 'running' | 'stopped';
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${apiBase}${path}`, {
    headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || res.statusText);
  }
  if (res.status === 204) {
    return undefined as T;
  }
  return res.json() as Promise<T>;
}

export const api = {
  health: () => request<{ version: string; profile: string; claude_on_path: boolean }>('/api/health'),

  createProject: (root_path: string) =>
    request<Project>('/api/projects', { method: 'POST', body: JSON.stringify({ root_path }) }),

  listTeams: () => request<TeamSummary[]>('/api/teams'),

  createTeam: (body: { project_id: string; name: string; provisioning_prompt: string }) =>
    request<Team>('/api/teams', { method: 'POST', body: JSON.stringify(body) }),

  addMember: (
    teamId: string,
    body: { name: string; role: string; role_prompt: string },
  ) =>
    request<TeamMember>(`/api/teams/${teamId}/members`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  listMembers: (teamId: string) => request<TeamMember[]>(`/api/teams/${teamId}/members`),

  launchTeam: (teamId: string) =>
    request<void>(`/api/teams/${teamId}/launch`, { method: 'POST' }),

  stopTeam: (teamId: string) => request<void>(`/api/teams/${teamId}/stop`, { method: 'POST' }),

  sendMessage: (teamId: string, text: string) =>
    request<void>(`/api/teams/${teamId}/message`, {
      method: 'POST',
      body: JSON.stringify({ text }),
    }),

  listTasks: (teamId: string) => request<Task[]>(`/api/teams/${teamId}/tasks`),

  createTask: (
    teamId: string,
    body: { title: string; description?: string; status?: TaskStatus },
  ) =>
    request<Task>(`/api/teams/${teamId}/tasks`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  patchTask: (
    teamId: string,
    taskId: string,
    body: { status?: TaskStatus; assignee_member_id?: string | null },
  ) =>
    request<Task>(`/api/teams/${teamId}/tasks/${taskId}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    }),

  listAgentRuns: (teamId: string) =>
    request<AgentRun[]>(`/api/teams/${teamId}/agent-runs`),

  doctor: () => request<DoctorResponse>('/api/setup/doctor'),

  getClaudeSettings: () => request<ClaudeSettingsView>('/api/setup/claude-settings'),

  patchClaudeSettings: (body: {
    credential_mode: string;
    api_key?: string;
    api_base_url?: string;
    default_model?: string | null;
  }) =>
    request<ClaudeSettingsView>('/api/setup/claude-settings', {
      method: 'PATCH',
      body: JSON.stringify(body),
    }),

  claudeLogin: () =>
    request<{ ok: boolean; message: string }>('/api/setup/claude-login', { method: 'POST' }),

  installClaude: (confirm: boolean) =>
    request<{ ok: boolean; command: string; output: string }>('/api/setup/install-claude', {
      method: 'POST',
      body: JSON.stringify({ confirm }),
    }),

  browseDirectory: (body?: { initial_path?: string }) =>
    request<{ path: string }>('/api/setup/browse-directory', {
      method: 'POST',
      body: JSON.stringify(body ?? {}),
    }),
};

export function wsUrl(): string {
  const base = apiBase || window.location.origin;
  const url = new URL('/ws', base);
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  return url.toString();
}
