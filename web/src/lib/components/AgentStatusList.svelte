<script lang="ts">
  import { agentRuns, members } from '../stores/orchestrator';

  function memberName(memberId: string): string {
    return $members.find((m) => m.id === memberId)?.name ?? memberId.slice(0, 8);
  }
</script>

<section class="agents">
  <h2>Agents</h2>
  {#if $agentRuns.length === 0}
    <p class="empty">No agent runs yet. Launch a team to start sessions.</p>
  {:else}
    <ul>
      {#each $agentRuns as run (run.id)}
        <li>
          <div class="row">
            <span class="name">{memberName(run.team_member_id)}</span>
            <span class="status status-{run.status}">{run.status}</span>
          </div>
          {#if run.last_output_snippet}
            <pre>{run.last_output_snippet.slice(-400)}</pre>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .agents h2 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
  }
  .empty {
    color: #8b95a8;
    font-size: 0.9rem;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  li {
    background: #1a1d24;
    border-radius: 6px;
    padding: 0.6rem;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.35rem;
  }
  .name {
    font-weight: 600;
  }
  .status {
    font-size: 0.75rem;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    text-transform: uppercase;
  }
  .status-running,
  .status-starting {
    background: #1f4d3a;
    color: #6ee7a0;
  }
  .status-stopped {
    background: #3a3f4a;
    color: #b8c0cc;
  }
  .status-error {
    background: #4d1f1f;
    color: #f5a0a0;
  }
  pre {
    margin: 0;
    font-size: 0.7rem;
    white-space: pre-wrap;
    word-break: break-word;
    color: #9aa3b2;
    max-height: 6rem;
    overflow: hidden;
  }
</style>
