<script lang="ts">
  let { task, onRollback } = $props<{
    task: any;
    onRollback: () => void;
  }>();
</script>

<div class="rollback-panel-container">
  <h3>ROLLBACK & SNAPSHOT KORUMASI</h3>

  {#if task && (task.status === 'completed' || task.status === 'in_progress')}
    <div class="status-box secured">
      <strong>Snapshot Koruma Güvencesi Aktif:</strong> Yazma öncesi alınan snapshot yedeği başarıyla `storage/snapshots` dizininde saklanmıştır.
      <div class="rollback-action">
        <p>Eğer yapılan değişikliklerin beklenmedik bir yan etkisi oluştuysa, işlemi geri alarak en son kararlı duruma dönebilirsiniz:</p>
        <button class="rollback-btn" onclick={onRollback}>Tüm Sistemi Geri Al (Rollback)</button>
      </div>
    </div>
  {:else if task && task.status === 'rollback_done'}
    <div class="status-box rolled-back">
      <strong>ROLLBACK BAŞARIYLA TAMAMLANDI:</strong> Sistem hata veya talep nedeniyle en son kararlı yedeğe döndürülmüş ve çalışma güvenli bir şekilde sonlandırılmıştır.
    </div>
  {:else}
    <p class="empty-msg">Yedekleme ve geri alma durumu henüz tetiklenmedi.</p>
  {/if}
</div>

<style>
  
  
  
  
  
  
  .rollback-action p { margin: 0 0 8px 0; color: #aaa; font-size: 0.75rem; }
  
  
  
  .rollback-panel-container { padding: 20px; background: rgba(224, 49, 49, 0.03); border: 1px solid rgba(224, 49, 49, 0.2); border-radius: 6px; margin-top: 20px;}
  h3 { color: #f03e3e; margin-top: 0; }
  .status-box { padding: 15px; border-radius: 4px; border-left: 4px solid; }
  .status-box.secured { background: #1a1a1c; border-color: #f03e3e; }
  .status-box.rolled-back { background: #1a1a1c; border-color: #47d18c; }
  .rollback-action { margin-top: 15px; text-align: center; }
  .rollback-btn { padding: 12px 24px; background: #e03131; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; font-size: 14px; text-transform: uppercase; box-shadow: 0 4px 10px rgba(224,49,49,0.3); transition: all 0.2s;}
  .rollback-btn:hover { background: #c92a2a; transform: scale(1.02); }
  .empty-msg { color: #8d8d95; font-style: italic; }
</style>
