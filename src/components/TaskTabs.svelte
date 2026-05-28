<script lang="ts">
  let { tasks = [], selectedTaskId = null, onSelect, onCreate } = $props<{
    tasks: any[];
    selectedTaskId: string | null;
    onSelect: (id: string) => void;
    onCreate: (title: string, req: string) => void;
  }>();

  let newTitle = $state("");
  let newTaskType = $state("Analiz");
  let selectedAgents = $state({ codex: true, oam: true, antigravity: false, cursor: false });
  let newRequest = $state("");

    function handleCreate(e: Event) {
    e.preventDefault();
    if (!newTitle || !newRequest) return;
    let agentTags = Object.entries(selectedAgents).filter(([_, v]) => v).map(([k, _]) => k.toUpperCase()).join(",");
    let finalRequest = `[${newTaskType}] [Ajanlar: ${agentTags}] ${newRequest}`;
    onCreate(newTitle, finalRequest);
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
      <select bind:value={newTaskType} class="task-type-select">
        <option value="Analiz">Sadece Analiz</option>
        <option value="Kod Yazma">Kod Değişikliği / Yazma</option>
        <option value="Araştırma">Dış İnternet Araştırması</option>
        <option value="Sistem">Sistem Taraması</option>
      </select>
      
      
      <div class="agent-selectors">
        <span>Ajan Atamaları:</span>
        <label><input type="checkbox" bind:checked={selectedAgents.codex} /> Codex</label>
        <label><input type="checkbox" bind:checked={selectedAgents.oam} /> OAM</label>
        <label><input type="checkbox" bind:checked={selectedAgents.antigravity} /> AntiGrav</label>
        <label><input type="checkbox" bind:checked={selectedAgents.cursor} /> Cursor</label>
      </div>

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
  .task-type-select { width: 100%; background: #252526; border: 1px solid #3c3c3c; color: white; padding: 8px; border-radius: 4px; font-size: 0.8rem; }
  .agent-selectors { display: flex; flex-wrap: wrap; gap: 8px; font-size: 0.75rem; background: #18181a; padding: 8px; border: 1px solid #333; border-radius: 4px; align-items: center; color: #ccc;}
  .agent-selectors span { width: 100%; font-weight: bold; color: #8d8d95; }
  .agent-selectors label { display: flex; align-items: center; gap: 4px; cursor: pointer; }
</style>
