<script lang="ts">
  let { logs = [] } = $props<{
    logs: any[];
  }>();

  let filterLevel = $state("ALL");
</script>

<div class="live-log-container">
  <div class="header">
    <h3>CANLI TAKİP & AUDIT LOGLARI</h3>
    <div class="filters">
      <label for="log-filter-select">Filtre:</label>
      <select id="log-filter-select" bind:value={filterLevel}>
        <option value="ALL">Hepsi (ALL)</option>
        <option value="INFO">Bilgi (INFO)</option>
        <option value="WARNING">Uyarı (WARNING)</option>
        <option value="ERROR">Hata (ERROR)</option>
      </select>
    </div>
  </div>

  <div class="console">
    {#each logs.filter((l: any) => filterLevel === 'ALL' || l.level === filterLevel) as log}
      <div class="log-line level-{log.level.toLowerCase()}">
        <span class="time">[{log.timestamp.substring(11, 19)}]</span>
        <span class="badge">{log.level}</span>
        {#if log.gate_name}
          <span class="gate">[{log.gate_name}]</span>
        {/if}
        <span class="msg">{log.message}</span>
      </div>
    {/each}

    {#if logs.length === 0}
      <p class="empty-msg">Henüz canlı kayıt üretilmedi.</p>
    {/if}
  </div>
</div>

<style>
  .live-log-container {
    padding: 15px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 4px;
    color: #ccc;
    font-family: monospace;
    font-size: 0.75rem;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #333;
    padding-bottom: 8px;
    margin-bottom: 10px;
  }
  .header h3 { margin: 0; font-size: 0.8rem; color: #eee; font-family: sans-serif; }
  .filters { display: flex; align-items: center; gap: 5px; font-family: sans-serif; }
  .filters select {
    background: #252526;
    border: 1px solid #3c3c3c;
    color: white;
    font-size: 0.7rem;
    padding: 2px 5px;
    border-radius: 3px;
  }
  .console {
    flex: 1;
    overflow-y: auto;
    background: #181818;
    border: 1px solid #2d2d2d;
    padding: 10px;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .log-line {
    line-height: 1.4;
    word-break: break-all;
  }
  .time { color: #888; margin-right: 5px; }
  .badge {
    font-weight: bold;
    margin-right: 5px;
  }
  .level-info .badge { color: #4ec9b0; }
  .level-info .msg { color: #dcdcaa; }
  
  .level-warning .badge { color: #ce9178; }
  .level-warning .msg { color: #ce9178; }

  .level-error .badge { color: #f44747; }
  .level-error .msg { color: #f44747; }

  .gate { color: #569cd6; font-weight: bold; margin-right: 5px; }
  .empty-msg { text-align: center; color: #555; font-size: 0.75rem; padding: 20px 0; }
</style>
