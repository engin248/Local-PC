<script lang="ts">
  let { decisions = [] } = $props<{
    decisions: any[];
  }>();
</script>

<div class="decision-map-container">
  <h3>DAĞITIK KARAR AĞACI & YETKİ MAPİ</h3>

  {#if decisions.length > 0}
    <div class="tree">
      {#each decisions as node}
        <div class="node-level level-{node.level}">
          <div class="node-card">
            <div class="node-header">
              <span class="node-id">Düğüm: {node.id.substring(3, 9)}</span>
              <span class="node-level-tag">Düzey {node.level}</span>
            </div>
            <div class="node-body">
              <p><strong>Yetkili Karar Verici:</strong> <span class="decider">{node.authorized_decider_id}</span></p>
              <p><strong>Durum:</strong> <span class="status {node.status}">{node.status}</span></p>
              {#if node.selected_option}
                <p><strong>Seçilen Yol:</strong> <span class="selected-option">{node.selected_option}</span></p>
              {/if}
              {#if node.reason}
                <p class="reason"><em>Gerekçe: {node.reason}</em></p>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Henüz karar ağacı oluşturulmadı veya bu görev için yetki yönlendirmesi yapılmadı.</p>
  {/if}
</div>

<style>
  .decision-map-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .tree {
    display: flex;
    flex-direction: column;
    gap: 15px;
    padding-left: 10px;
    border-left: 2px dashed #444;
  }
  .node-level {
    position: relative;
  }
  .node-level::before {
    content: "";
    position: absolute;
    left: -12px;
    top: 20px;
    width: 12px;
    height: 2px;
    background: #444;
  }
  .level-1 { margin-left: 0; }
  .level-2 { margin-left: 20px; }
  .level-3 { margin-left: 40px; }
  .level-4 { margin-left: 60px; }

  .node-card {
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 10px;
    max-width: 500px;
  }
  .node-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #333;
    padding-bottom: 5px;
    margin-bottom: 8px;
  }
  .node-id { font-family: monospace; font-size: 0.75rem; color: #888; }
  .node-level-tag {
    font-size: 0.65rem;
    background: #333;
    color: #aaa;
    padding: 2px 5px;
    border-radius: 3px;
    text-transform: uppercase;
  }
  .node-body p { margin: 3px 0; font-size: 0.75rem; }
  .decider { color: #569cd6; font-weight: bold; }
  .status {
    text-transform: uppercase;
    font-weight: bold;
    font-size: 0.7rem;
  }
  .status.evaluated { color: #4ec9b0; }
  .status.pending { color: #dcdcaa; }
  .selected-option { color: #ce9178; font-weight: bold; }
  .reason {
    margin-top: 5px !important;
    color: #888;
    background: #2d2d2d;
    padding: 4px;
    border-radius: 3px;
  }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
