<script lang="ts">
  let { reports = [] } = $props<{
    reports: any[];
  }>();
</script>

<div class="report-panel-container">
  <h3>İŞLEM VE DENETİM RAPORU</h3>

  {#if reports.length > 0}
    <div class="reports-list">
      {#each reports as rep}
        <div class="report-card">
          <div class="report-header">
            <span>Rapor Tipi: {rep.report_type.toUpperCase()}</span>
          </div>
          <div class="report-body">
            <pre>{rep.content}</pre>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Nihai rapor henüz oluşturulmadı. 8 Kapı başarıyla tamamlandığında rapor üretilecektir.</p>
  {/if}
</div>

<style>
  .report-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .report-card {
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 12px;
  }
  .report-header {
    border-bottom: 1px solid #333;
    padding-bottom: 5px;
    margin-bottom: 8px;
    font-size: 0.7rem;
    color: #888;
    font-weight: bold;
  }
  .report-body pre {
    margin: 0;
    white-space: pre-wrap;
    font-family: monospace;
    font-size: 0.75rem;
    color: #dcdcaa;
    line-height: 1.5;
  }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
