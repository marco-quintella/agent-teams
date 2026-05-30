<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './lib/api/client';
  import AgentStatusList from './lib/components/AgentStatusList.svelte';
  import KanbanBoard from './lib/components/KanbanBoard.svelte';
  import TeamLauncher from './lib/components/TeamLauncher.svelte';
  import {
    connectionStatus,
    lastError,
    launched,
    members,
    resumeTeamIfStored,
    teamId,
  } from './lib/stores/orchestrator';

  let health = $state<{ claude_on_path: boolean; profile: string } | null>(null);

  const showGitWarning = $derived($launched && $members.length > 1);

  onMount(async () => {
    try {
      health = await api.health();
    } catch {
      health = null;
    }
    await resumeTeamIfStored();
  });
</script>

<div class="app">
  <header>
    <h1>Claude Orchestrator</h1>
    {#if health}
      <span class="badge">profile: {health.profile}</span>
      <span class="badge" class:warn={!health.claude_on_path}>
        claude: {health.claude_on_path ? 'on PATH' : 'missing'}
      </span>
    {/if}
    <span class="badge ws-{$connectionStatus}">ws: {$connectionStatus}</span>
  </header>

  <aside class="banner">
    V1 has no authentication. Do not expose this UI on a public VPS without a reverse proxy and auth.
  </aside>

  {#if showGitWarning}
    <aside class="banner git-warn">
      Multiple agents share one project checkout. Enable git worktree isolation before parallel edits (V1.1 does not enforce worktrees).
    </aside>
  {/if}

  {#if $lastError}
    <aside class="error">{$lastError}</aside>
  {/if}

  <main>
    <div class="sidebar">
      <TeamLauncher />
      <AgentStatusList />
    </div>
    <div class="board">
      {#if $teamId}
        <KanbanBoard />
      {:else}
        <p class="hint">Create and launch a team to open the kanban board.</p>
      {/if}
    </div>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: #0d0f12;
    color: #e8ecf1;
  }
  .app {
    max-width: 1400px;
    margin: 0 auto;
    padding: 1rem;
  }
  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }
  header h1 {
    margin: 0;
    font-size: 1.35rem;
    flex: 1;
  }
  .badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    background: #252a35;
    color: #b8c0cc;
  }
  .badge.warn {
    background: #4d3a1f;
    color: #f5d0a0;
  }
  .banner {
    background: #3a2a10;
    color: #f0d090;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 0.75rem;
  }
  .banner.git-warn {
    background: #3a3010;
    color: #e8d080;
  }
  .error {
    background: #4d1f1f;
    color: #f5c0c0;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    margin-bottom: 0.75rem;
  }
  main {
    display: grid;
    grid-template-columns: 320px 1fr;
    gap: 1rem;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .hint {
    color: #8b95a8;
    padding: 2rem;
    text-align: center;
  }
  @media (max-width: 900px) {
    main {
      grid-template-columns: 1fr;
    }
  }
</style>
