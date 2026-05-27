<script lang="ts">
  let { approvals = [], onSubmitApproval } = $props<{
    approvals: any[];
    onSubmitApproval: (id: string, approve: boolean, note: string, approverId: string, approverRole: string) => void;
  }>();

  let note = $state("");
  let approverId = $state("");
  let approverRole = $state("");

  let approvalFormReady = $derived(
    note.trim().length > 0 && approverId.trim().length > 0 && approverRole.trim().length > 0
  );
</script>

<div class="approval-panel-container">
  <h3>KULLANICI MANUEL ONAY PANELİ</h3>

  {#if approvals.length > 0}
    <div class="approvals-list">
      {#each approvals as app}
        <div class="approval-card" class:pending={app.status === 'pending'}>
          <div class="approval-header">
            <span class="action">Eylem: {app.action}</span>
            <span class="risk-badge {app.risk_level}">{app.risk_level} Risk</span>
          </div>
          
          <div class="approval-body">
            <p><strong>Talep Durumu:</strong> <span class="status-text {app.status}">{app.status.toUpperCase()}</span></p>
            {#if app.approver_id}
              <p><strong>Onay Veren:</strong> {app.approver_id} / {app.approver_role || "rol yok"}</p>
            {/if}
            
            {#if app.status === 'pending'}
              <div class="action-inputs">
                <input placeholder="Onay Gerekçesi / Notu..." bind:value={note} />
                <input placeholder="Onay Veren Kullanıcı ID..." bind:value={approverId} />
                <select bind:value={approverRole}>
                  <option value="" disabled>Rol seçin</option>
                  <option value="admin">admin</option>
                  <option value="owner">owner</option>
                  <option value="security_officer">security_officer</option>
                  <option value="operator">operator</option>
                  <option value="user">user</option>
                </select>
                <div class="buttons">
                  <button class="btn approve-btn" disabled={!approvalFormReady} onclick={() => onSubmitApproval(app.id, true, note, approverId, approverRole)}>İŞLEMİ ONAYLA</button>
                  <button class="btn reject-btn" disabled={!approvalFormReady} onclick={() => onSubmitApproval(app.id, false, note, approverId, approverRole)}>İŞLEMİ REDDET</button>
                </div>
              </div>
            {:else}
              <p class="resolved-msg">Bu onay talebi sonuçlandırılmıştır.</p>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Aktif bir onay talebi bulunmamaktadır.</p>
  {/if}
</div>

<style>
  .approval-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .approvals-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .approval-card {
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 12px;
  }
  .approval-card.pending {
    border-color: #ce9178;
    background: #2b1f1a;
  }
  .approval-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #333;
    padding-bottom: 6px;
    margin-bottom: 8px;
  }
  .action { font-weight: bold; font-size: 0.8rem; color: #eee; }
  .risk-badge {
    font-size: 0.65rem;
    font-weight: bold;
    padding: 2px 5px;
    border-radius: 3px;
  }
  .risk-badge.high { background: #ce9178; color: white; }
  .risk-badge.critical { background: #f44747; color: white; }

  .status-text { font-weight: bold; font-size: 0.75rem; }
  .status-text.pending { color: #ce9178; }
  .status-text.approved { color: #4ec9b0; }
  .status-text.rejected { color: #f44747; }

  .action-inputs {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .action-inputs input {
    padding: 8px;
    background: #1e1e1e;
    border: 1px solid #444;
    color: white;
    font-size: 0.8rem;
    border-radius: 4px;
  }
  .buttons {
    display: flex;
    gap: 10px;
  }
  .btn {
    flex: 1;
    padding: 8px;
    font-weight: bold;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .approve-btn { background: #4ec9b0; color: #1e1e1e; }
  .approve-btn:hover { background: #5fd9c0; }
  .reject-btn { background: #f44747; color: white; }
  .reject-btn:hover { background: #ff5757; }

  .resolved-msg { color: #888; font-size: 0.75rem; font-style: italic; }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
