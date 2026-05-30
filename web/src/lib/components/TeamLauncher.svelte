<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { api, type TeamSummary } from '../api/client';
  import {
    connectWs,
    launched,
    lastError,
    loadTeam,
    teamId,
  } from '../stores/orchestrator';

  let credentialsReady = $state(false);
  let doctorHint = $state('');

  let teams = $state<TeamSummary[]>([]);
  let teamsLoading = $state(true);
  let mode = $state<'selected' | 'create-new'>('create-new');

  let projectPath = $state('');
  let teamName = $state('Alpha team');
  let provisioningPrompt = $state('Ship the orchestrator V1');
  let leadName = $state('Lead');
  let workerName = $state('Worker');
  let messageText = $state('');
  let launchBusy = $state(false);
  let messageBusy = $state(false);
  let browseBusy = $state(false);
  let browseError = $state('');
  let messageHint = $state<{ kind: 'ok' | 'err'; text: string } | null>(null);

  async function refreshTeams() {
    teamsLoading = true;
    try {
      teams = await api.listTeams();
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
      teams = [];
    } finally {
      teamsLoading = false;
    }
  }

  onMount(async () => {
    if (get(teamId)) {
      mode = 'selected';
    }
    try {
      const d = await api.doctor();
      credentialsReady = d.credentials.ready && d.cli.found;
      doctorHint = d.credentials.hint;
    } catch {
      credentialsReady = false;
      doctorHint = 'Could not reach doctor API.';
    }
    await refreshTeams();
  });

  async function selectTeam(id: string) {
    lastError.set(null);
    mode = 'selected';
    connectWs();
    try {
      await loadTeam(id);
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
    }
  }

  function startCreateNew() {
    mode = 'create-new';
    teamId.set(null);
    projectPath = '';
    browseError = '';
  }

  async function browseDirectory() {
    browseBusy = true;
    browseError = '';
    try {
      const res = await api.browseDirectory(
        projectPath.trim() ? { initial_path: projectPath.trim() } : undefined,
      );
      projectPath = res.path;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (!msg.includes('cancelled')) {
        browseError = msg.includes('503') || msg.includes('Service')
          ? 'Folder browse unavailable; type the path manually.'
          : msg;
      }
    } finally {
      browseBusy = false;
    }
  }

  async function createAndLaunch() {
    launchBusy = true;
    lastError.set(null);
    let createdTeamId: string | null = null;
    try {
      const project = await api.createProject(projectPath);
      const team = await api.createTeam({
        project_id: project.id,
        name: teamName,
        provisioning_prompt: provisioningPrompt,
      });
      createdTeamId = team.id;
      await api.addMember(team.id, {
        name: leadName,
        role: 'lead',
        role_prompt: 'Coordinate the team and update the board via ATOP.',
      });
      await api.addMember(team.id, {
        name: workerName,
        role: 'worker',
        role_prompt: 'Implement assigned tasks and report status via ATOP.',
      });
      connectWs();
      await api.launchTeam(team.id);
      await loadTeam(team.id);
      mode = 'selected';
      await refreshTeams();
      browseError = '';
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
      if (createdTeamId) {
        try {
          await loadTeam(createdTeamId);
          mode = 'selected';
        } catch {
          // keep partial state visible when launch succeeded but UI sync failed
        }
      }
    } finally {
      launchBusy = false;
    }
  }

  async function launchSelected() {
    const tid = $teamId;
    if (!tid) return;
    launchBusy = true;
    lastError.set(null);
    try {
      connectWs();
      await api.launchTeam(tid);
      await loadTeam(tid);
      await refreshTeams();
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
    } finally {
      launchBusy = false;
    }
  }

  async function stopTeam() {
    const tid = $teamId;
    if (!tid) return;
    launchBusy = true;
    try {
      await api.stopTeam(tid);
      await loadTeam(tid);
      await refreshTeams();
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
    } finally {
      launchBusy = false;
    }
  }

  async function sendMessage() {
    const tid = $teamId;
    if (!tid || !messageText.trim()) return;
    messageBusy = true;
    messageHint = null;
    lastError.set(null);
    try {
      await api.sendMessage(tid, messageText.trim());
      messageText = '';
      messageHint = {
        kind: 'ok',
        text: 'Message sent to the lead session.',
      };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      const offline =
        msg.includes('not running') || msg.includes('409') || msg.includes('CONFLICT');
      messageHint = {
        kind: 'err',
        text: offline
          ? 'Lead session is offline. Click Launch / Relaunch team, then send again.'
          : msg,
      };
      lastError.set(messageHint.text);
      if (offline) {
        await loadTeam(tid);
        await refreshTeams();
      }
    } finally {
      messageBusy = false;
    }
  }
</script>

<section class="launcher">
  <h2>Team launcher</h2>
  {#if !credentialsReady}
    <p class="warn">
      Claude not ready: {doctorHint} Open <strong>Settings</strong> to configure credentials.
    </p>
  {/if}

  <div class="team-history">
    <div class="history-header">
      <h3>Saved teams</h3>
      <button type="button" class="secondary" disabled={teamsLoading} onclick={refreshTeams}>
        Refresh
      </button>
    </div>
    {#if teamsLoading}
      <p class="meta">Loading teams…</p>
    {:else if teams.length === 0}
      <p class="meta">No teams yet. Create one below.</p>
    {:else}
      <ul class="team-list">
        {#each teams as team (team.id)}
          <li>
            <button
              type="button"
              class="team-row"
              class:active={$teamId === team.id && mode === 'selected'}
              onclick={() => selectTeam(team.id)}
            >
              <span class="team-name">{team.name}</span>
              <span class="team-path">{team.project_root_path}</span>
              <span class="badge" class:running={team.status === 'running'}>
                {team.status}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <button type="button" class="secondary" onclick={startCreateNew}>Create new team</button>
  </div>

  {#if $teamId && mode !== 'create-new'}
    <div class="selected-panel">
      {#each teams as team (team.id)}
        {#if team.id === $teamId}
          <p class="meta">
            <strong>{team.name}</strong>
            <span class="path-readonly">{team.project_root_path}</span>
          </p>
        {/if}
      {/each}
      <div class="actions">
        <button
          type="button"
          disabled={launchBusy || !credentialsReady}
          onclick={launchSelected}
        >
          {launchBusy ? 'Working…' : $launched ? 'Relaunch team' : 'Launch team'}
        </button>
        <button type="button" disabled={launchBusy || !$teamId} onclick={stopTeam}>
          Stop team
        </button>
      </div>
      <p class="meta">Team id: <code>{$teamId}</code> {#if $launched}(running){/if}</p>
      <div class="message-row">
        <input
          bind:value={messageText}
          placeholder="Message to lead"
          onkeydown={(e) => e.key === 'Enter' && sendMessage()}
        />
        <button type="button" disabled={!$launched || messageBusy} onclick={sendMessage}>
          {messageBusy ? 'Sending…' : 'Send'}
        </button>
      </div>
      {#if messageHint}
        <p class="message-hint" class:ok={messageHint.kind === 'ok'} class:err={messageHint.kind === 'err'}>
          {messageHint.text}
        </p>
      {/if}
      {#if !$launched}
        <p class="meta">Launch the team before sending a message to the lead.</p>
      {/if}
    </div>
  {:else}
    <div class="create-panel">
      <h3>Create new team</h3>
      <label>
        Project path
        <div class="path-row">
          <input bind:value={projectPath} placeholder="/path/to/repo" />
          <button
            type="button"
            class="secondary"
            disabled={browseBusy || launchBusy}
            onclick={browseDirectory}
          >
            {browseBusy ? '…' : 'Browse…'}
          </button>
        </div>
      </label>
      {#if browseError}
        <p class="browse-err">{browseError}</p>
      {/if}
      <label>
        Team name
        <input bind:value={teamName} />
      </label>
      <label>
        Provisioning prompt
        <textarea bind:value={provisioningPrompt} rows="3"></textarea>
      </label>
      <div class="members">
        <label>Lead <input bind:value={leadName} /></label>
        <label>Worker <input bind:value={workerName} /></label>
      </div>
      <div class="actions">
        <button
          type="button"
          disabled={launchBusy || !projectPath || !credentialsReady}
          onclick={createAndLaunch}
        >
          {launchBusy ? 'Launching…' : 'Create & launch'}
        </button>
      </div>
    </div>
  {/if}
</section>

<style>
  .launcher {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .launcher h2,
  .launcher h3 {
    margin: 0;
    font-size: 1rem;
  }
  .team-history {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #2a3140;
  }
  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .team-list {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .team-row {
    width: 100%;
    text-align: left;
    padding: 0.45rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #3a4254;
    background: #12151a;
    color: #e8ecf1;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .team-row.active {
    border-color: #3d6cf5;
    background: #1a2238;
  }
  .team-name {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .team-path {
    font-size: 0.72rem;
    color: #8b95a8;
    word-break: break-all;
  }
  .badge {
    align-self: flex-start;
    font-size: 0.65rem;
    text-transform: uppercase;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    background: #2a3140;
    color: #9aa3b2;
  }
  .badge.running {
    background: #1a3a28;
    color: #90c090;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #9aa3b2;
  }
  input,
  textarea {
    padding: 0.45rem;
    border-radius: 4px;
    border: 1px solid #3a4254;
    background: #12151a;
    color: #e8ecf1;
  }
  .path-row {
    display: flex;
    gap: 0.35rem;
  }
  .path-row input {
    flex: 1;
  }
  .path-readonly {
    display: block;
    font-size: 0.75rem;
    color: #8b95a8;
    word-break: break-all;
    margin-top: 0.25rem;
  }
  .members {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }
  .actions,
  .message-row {
    display: flex;
    gap: 0.5rem;
  }
  .message-row input {
    flex: 1;
  }
  button {
    padding: 0.45rem 0.9rem;
    border-radius: 4px;
    border: none;
    background: #3d6cf5;
    color: white;
    cursor: pointer;
  }
  button.secondary {
    background: #3a4458;
    color: #e8ecf1;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .meta {
    font-size: 0.8rem;
    color: #8b95a8;
  }
  code {
    font-size: 0.75rem;
  }
  .warn {
    font-size: 0.8rem;
    color: #f0d090;
    background: #3a2a10;
    padding: 0.5rem;
    border-radius: 4px;
    margin: 0;
  }
  .browse-err {
    font-size: 0.8rem;
    color: #f5a0a0;
    margin: 0;
  }
  .message-hint {
    font-size: 0.8rem;
    margin: 0.25rem 0 0;
  }
  .message-hint.ok {
    color: #90c090;
  }
  .message-hint.err {
    color: #f5a0a0;
  }
</style>
