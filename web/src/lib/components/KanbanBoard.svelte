<script lang="ts">
  import { untrack } from 'svelte';
  import { dndzone } from 'svelte-dnd-action';
  import type { Task, TaskStatus } from '../api/client';
  import { api } from '../api/client';
  import { loadTeam, tasks, teamId } from '../stores/orchestrator';

  const columnDefs: { id: TaskStatus; label: string }[] = [
    { id: 'backlog', label: 'Backlog' },
    { id: 'in_progress', label: 'In progress' },
    { id: 'review', label: 'Review' },
    { id: 'done', label: 'Done' },
  ];

  let newTitle = $state('');
  let columnItems = $state<Record<TaskStatus, Task[]>>({
    backlog: [],
    in_progress: [],
    review: [],
    done: [],
  });
  let dragging = $state(false);
  /** Bumps when task list changes so dnd zones re-bind items from the store. */
  let boardKey = $state(0);

  function columnsFromTasks(list: Task[]): Record<TaskStatus, Task[]> {
    return {
      backlog: list.filter((t) => t.status === 'backlog'),
      in_progress: list.filter((t) => t.status === 'in_progress'),
      review: list.filter((t) => t.status === 'review'),
      done: list.filter((t) => t.status === 'done'),
    };
  }

  function columnsSignature(cols: Record<TaskStatus, Task[]>): string {
    return (['backlog', 'in_progress', 'review', 'done'] as const)
      .map((s) => cols[s].map((t) => `${t.id}:${t.status}`).join(','))
      .join('|');
  }

  $effect(() => {
    const list = $tasks;
    if (dragging) return;
    const next = columnsFromTasks(list);
    untrack(() => {
      if (columnsSignature(columnItems) === columnsSignature(next)) return;
      columnItems = next;
      boardKey += 1;
    });
  });

  function onConsider(status: TaskStatus) {
    return (e: CustomEvent<{ items: Task[] }>) => {
      dragging = true;
      columnItems[status] = e.detail.items;
    };
  }

  async function onFinalize(status: TaskStatus, e: CustomEvent<{ items: Task[] }>) {
    const tid = $teamId;
    dragging = false;
    if (!tid) return;
    columnItems[status] = e.detail.items;
    for (const task of e.detail.items) {
      if (task.status !== status) {
        await api.patchTask(tid, task.id, { status });
      }
    }
    await loadTeam(tid);
  }

  async function addTask() {
    const tid = $teamId;
    if (!tid || !newTitle.trim()) return;
    await api.createTask(tid, { title: newTitle.trim(), status: 'backlog' });
    newTitle = '';
    await loadTeam(tid);
  }
</script>

<section class="kanban">
  <div class="kanban-toolbar">
    <input bind:value={newTitle} placeholder="New task title" onkeydown={(e) => e.key === 'Enter' && addTask()} />
    <button type="button" onclick={addTask}>Add task</button>
    {#if $tasks.length > 0}
      <span class="task-count">{$tasks.length} task(s)</span>
    {/if}
  </div>

  <div class="columns">
    {#each columnDefs as col (col.id)}
      <div class="column">
        <h3>{col.label} <span class="count">({columnItems[col.id].length})</span></h3>
        {#key `${col.id}-${boardKey}`}
          <div
            class="drop-zone"
            use:dndzone={{
              items: columnItems[col.id],
              flipDurationMs: 150,
              type: col.id,
            }}
            onconsider={onConsider(col.id)}
            onfinalize={(e) => onFinalize(col.id, e)}
          >
            {#each columnItems[col.id] as task (task.id)}
              <article class="card" aria-label={task.title}>
                <strong>{task.title}</strong>
                {#if task.description}
                  <p>{task.description}</p>
                {/if}
                <small>{task.created_by}</small>
              </article>
            {:else}
              <p class="empty-col">No tasks</p>
            {/each}
          </div>
        {/key}
      </div>
    {/each}
  </div>
</section>

<style>
  .kanban-toolbar {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-bottom: 1rem;
  }
  .kanban-toolbar input {
    flex: 1;
    padding: 0.5rem;
  }
  .task-count {
    font-size: 0.8rem;
    color: #8b95a8;
    white-space: nowrap;
  }
  .columns {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.75rem;
  }
  .column {
    background: #1a1d24;
    border-radius: 8px;
    padding: 0.75rem;
    min-height: 280px;
  }
  .column h3 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #9aa3b2;
  }
  .count {
    font-weight: normal;
    color: #6b7689;
  }
  .drop-zone {
    min-height: 200px;
  }
  .empty-col {
    margin: 0;
    font-size: 0.8rem;
    color: #6b7689;
  }
  .card {
    background: #2a3140;
    border-radius: 6px;
    padding: 0.6rem;
    margin-bottom: 0.5rem;
    cursor: grab;
  }
  .card p {
    margin: 0.25rem 0;
    font-size: 0.85rem;
    color: #b8c0cc;
  }
  .card small {
    color: #6b7689;
  }
</style>
