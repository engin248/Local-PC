<script lang="ts">
  let { tasks = [], selectedTaskId = null, onSelect, onCreate } = $props<{
    tasks: any[];
    selectedTaskId: string | null;
    onSelect: (id: string) => void;
    onCreate: (title: string, req: string) => void;
  }>();

  let newTitle = $state("");
  let newRequest = $state("");

  function handleCreate(e: Event) {
    e.preventDefault();
    if (!newTitle || !newRequest) return;
    onCreate(newTitle, newRequest);
    newTitle = "";
    newRequest = "";
  }
</script>

<div class="task-tabs-container">
  <h3>İŞLEM SEKMELERİ</h3>
  <div class="tabs-list">
    {#each tasks as task}
      <button 
        class="tab-btn" 
        class:active={selectedTaskId === task.id} 
        onclick={() => onSelect(task.id)}
      >
        <span class="id">[{task.id.substring(5, 11)}]</span> {task.title}
        <span class="status-badge {task.status}">{task.status}</span>
      </button>
    {/each}
  </div>

  <div class="new-task-form">
    <h4>YENİ İŞLEM BAŞLAT</h4>
    <form onsubmit={handleCreate}>
      <input placeholder="Görev Başlığı" bind:value={newTitle} />
      <textarea placeholder="Kullanıcı Talebi..." bind:value={newRequest}></textarea>
      <button type="submit">Görev Kaydet (Intake)</button>
    </form>
  </div>
</div>

<style>
  .task-tabs-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 10px;
    background: #1e1e1e;
    border-right: 1px solid #333;
    color: #ccc;
  }
  h3, h4 {
    margin: 5px 0 10px 0;
    font-size: 0.85rem;
    letter-spacing: 1px;
    border-bottom: 1px solid #444;
    padding-bottom: 5px;
    color: #e0e0e0;
  }
  .tabs-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .tab-btn {
    text-align: left;
    background: #252526;
    border: 1px solid #3c3c3c;
    color: #aaa;
    padding: 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .tab-btn:hover {
    background: #2d2d2d;
    color: #fff;
  }
  .tab-btn.active {
    background: #007acc;
    color: white;
    border-color: #0098ff;
  }
  .id {
    font-family: monospace;
    font-weight: bold;
  }
  .status-badge {
    font-size: 0.65rem;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: uppercase;
    font-weight: bold;
    background: #444;
    color: #eee;
  }
  .status-badge.completed { background: #4ec9b0; color: #1e1e1e; }
  .status-badge.failed { background: #f44747; color: #fff; }
  .status-badge.executing { background: #dcdcaa; color: #1e1e1e; }
  .status-badge.planning_incomplete { background: #ce9178; color: white; }
  
  .new-task-form {
    margin-top: 15px;
    border-top: 1px solid #444;
    padding-top: 15px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  input, textarea {
    width: 100%;
    padding: 8px;
    background: #252526;
    border: 1px solid #3c3c3c;
    color: white;
    font-size: 0.8rem;
    border-radius: 4px;
    box-sizing: border-box;
  }
  textarea { height: 60px; resize: none; }
  form button {
    width: 100%;
    padding: 8px;
    background: #3c3c3c;
    border: 1px solid #555;
    color: white;
    cursor: pointer;
    font-size: 0.8rem;
    border-radius: 4px;
  }
  form button:hover { background: #4c4c4c; }
</style>
