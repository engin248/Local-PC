<script lang="ts">
  let { task, onRollback } = $props<{
    task: any;
    onRollback: () => void;
  }>();
</script>

<div class="rollback-panel-container">
  <h3>ROLLBACK & SNAPSHOT KORUMASI</h3>

  {#if task && task.status === 'completed'}
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
  .rollback-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .status-box {
    padding: 12px;
    border-radius: 4px;
    font-size: 0.8rem;
    line-height: 1.5;
  }
  .status-box.secured {
    background: #1a332d;
    border: 1px solid #4ec9b0;
    color: #4ec9b0;
  }
  .status-box.rolled-back {
    background: #3c2419;
    border: 1px solid #ce9178;
    color: #ce9178;
  }
  .rollback-action {
    margin-top: 10px;
    border-top: 1px solid #333;
    padding-top: 10px;
  }
  .rollback-action p { margin: 0 0 8px 0; color: #aaa; font-size: 0.75rem; }
  .rollback-btn {
    background: #ce9178;
    color: #1e1e1e;
    font-weight: bold;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }
  .rollback-btn:hover { background: #dfa289; }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
