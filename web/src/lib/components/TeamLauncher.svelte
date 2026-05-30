<script lang="ts">
  import { api } from '../api/client';
  import {
    connectWs,
    launched,
    lastError,
    loadTeam,
    teamId,
  } from '../stores/orchestrator';

  let projectPath = $state('');
  let teamName = $state('Alpha team');
  let provisioningPrompt = $state('Ship the orchestrator V1');
  let leadName = $state('Lead');
  let workerName = $state('Worker');
  let messageText = $state('');
  let launchBusy = $state(false);
  let messageBusy = $state(false);

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
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
      if (createdTeamId) {
        try {
          await loadTeam(createdTeamId);
        } catch {
          // keep partial state visible when launch succeeded but UI sync failed
        }
      }
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
    lastError.set(null);
    try {
      await api.sendMessage(tid, messageText.trim());
      messageText = '';
    } catch (e) {
      lastError.set(e instanceof Error ? e.message : String(e));
    } finally {
      messageBusy = false;
    }
  }
</script>

<section class="launcher">
  <h2>Team launcher</h2>
  <label>
    Project path
    <input bind:value={projectPath} placeholder="C:\path\to\repo" />
  </label>
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
    <button type="button" disabled={launchBusy || !projectPath} onclick={createAndLaunch}>
      {launchBusy ? 'Launching…' : 'Launch team'}
    </button>
    <button type="button" disabled={launchBusy || !$teamId} onclick={stopTeam}>Stop team</button>
  </div>

  {#if $teamId}
    <p class="meta">Team id: <code>{$teamId}</code> {#if $launched}(running){/if}</p>
    <div class="message-row">
      <input bind:value={messageText} placeholder="Message to lead" />
      <button type="button" disabled={!$launched || messageBusy} onclick={sendMessage}>
        {messageBusy ? 'Sending…' : 'Send'}
      </button>
    </div>
  {/if}
</section>

<style>
  .launcher {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .launcher h2 {
    margin: 0;
    font-size: 1.1rem;
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
</style>
