<script lang="ts">
  export let tasks: any[] = [];
  export let selectedTaskId: string | null = null;
  export let onSelect: (id: string | null) => void;
</script>

<div class="task-list">
  <div class="task-list-header">
    <h3>Mevcut Operasyonlar</h3>
    <button class="new-task-btn" on:click={() => onSelect(null)}>+ YENİ İŞLEM BAŞLAT</button>
  </div>
  <ul>
    {#each tasks as t}
      <li class:active={t.id === selectedTaskId}>
        <button
          type="button"
          class="task-select-btn"
          on:click={() => onSelect(t.id)}
          aria-current={t.id === selectedTaskId ? "true" : undefined}
        >
        <div class="task-info">
          <span class="title">{t.title}</span>
          <span class="status {t.status}">{t.status.replace('_', ' ')}</span>
        </div>
        </button>
      </li>
    {/each}
    {#if tasks.length === 0}
      <li class="empty-msg">Henüz görev yok.</li>
    {/if}
  </ul>
</div>

<style>
  .task-list {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .task-list-header {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 15px;
    border-bottom: 1px solid #1f1f21;
  }
  .task-list-header h3 {
    margin: 0;
    color: #f4f4f5;
    font-size: 14px;
    text-transform: uppercase;
  }
  .new-task-btn {
    background: #0b74de;
    color: white;
    border: none;
    padding: 10px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    transition: background 0.2s;
  }
  .new-task-btn:hover { background: #005bb5; }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    overflow-y: auto;
    flex: 1;
  }
  li {
    border-bottom: 1px solid #1f1f21;
    transition: background 0.2s;
  }
  li:hover { background: #1a1a1c; }
  li.active {
    background: #18181a;
    border-left: 3px solid #0b74de;
  }
  .task-info { display: flex; flex-direction: column; gap: 5px; }
  .task-select-btn {
    width: 100%;
    padding: 15px;
    background: transparent;
    border: 0;
    text-align: left;
    cursor: pointer;
    font: inherit;
  }
  .task-select-btn:focus-visible {
    outline: 2px solid #0b74de;
    outline-offset: -2px;
  }
  .title { color: #f4f4f5; font-weight: 500; font-size: 14px; }
  .status { font-size: 11px; text-transform: uppercase; font-weight: bold; padding: 2px 6px; border-radius: 4px; display: inline-block; width: fit-content; }
  .status.pending { background: #3b3b40; color: #a1a1aa; }
  .status.in_progress { background: rgba(11,116,222,0.1); color: #0b74de; }
  .status.completed { background: rgba(71,209,140,0.1); color: #47d18c; }
  .empty-msg { color: #8d8d95; text-align: center; padding: 20px; font-style: italic; cursor: default; }
  .empty-msg:hover { background: transparent; }
</style>
