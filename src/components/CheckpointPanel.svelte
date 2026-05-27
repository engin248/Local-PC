<script lang="ts">
  let { checkpoints = [] } = $props<{
    checkpoints: any[];
  }>();
</script>

<div class="checkpoint-panel-container">
  <h3>KONTROL NOKTALARI & KAPILARI (CHECKPOINTS)</h3>

  {#if checkpoints.length > 0}
    <div class="checkpoints-list">
      {#each checkpoints as cp}
        <div class="checkpoint-item" class:passed={cp.status === 'passed'}>
          <div class="status-indicator {cp.status}"></div>
          <div class="cp-info">
            <span class="type">{cp.checkpoint_type.toUpperCase()}</span>
            <p class="result">{cp.result}</p>
          </div>
          <span class="badge {cp.status}">{cp.status.toUpperCase()}</span>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Kontrol noktası doğrulamaları henüz çalıştırılmadı.</p>
  {/if}
</div>

<style>
  .checkpoint-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .checkpoints-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .checkpoint-item {
    display: flex;
    align-items: center;
    background: #252526;
    border: 1px solid #2d2d2d;
    padding: 8px 12px;
    border-radius: 4px;
    gap: 15px;
  }
  .checkpoint-item.passed {
    border-color: #2e4d3a;
  }
  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .status-indicator.passed { background: #4ec9b0; }
  .status-indicator.failed { background: #f44747; }
  .status-indicator.pending { background: #dcdcaa; }

  .cp-info { flex: 1; }
  .cp-info .type { font-size: 0.65rem; color: #888; font-weight: bold; }
  .cp-info .result { margin: 3px 0 0 0; font-size: 0.75rem; color: #eee; }

  .badge {
    font-size: 0.65rem;
    font-weight: bold;
    padding: 2px 6px;
    border-radius: 3px;
  }
  .badge.passed { background: #4ec9b0; color: #1e1e1e; }
  .badge.failed { background: #f44747; color: white; }
  .badge.pending { background: #dcdcaa; color: #1e1e1e; }

  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
