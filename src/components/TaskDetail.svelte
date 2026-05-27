<script lang="ts">
  let { task = null, onExecute } = $props<{
    task: any;
    onExecute: () => void;
  }>();

  let createdAt = $derived(
    task?.created_at ? task.created_at.substring(0, 19).replace('T', ' ') : "kayıt yok"
  );
</script>

<div class="task-detail-container">
  {#if task}
    <div class="header">
      <h2>GÖREV DETAYI: {task.title}</h2>
      <span class="id">ID: {task.id}</span>
    </div>

    <div class="detail-grid">
      <div class="card">
        <span class="label">Kullanıcı Talebi</span>
        <p class="value">{task.user_request}</p>
      </div>

      <div class="card-row">
        <div class="card mini">
          <span class="label">Genel Durum</span>
          <span class="badge {task.status}">{task.status}</span>
        </div>
        <div class="card mini">
          <span class="label">Risk Seviyesi</span>
          <span class="badge {task.risk_level}">{task.risk_level}</span>
        </div>
        <div class="card mini">
          <span class="label">Onay Durumu</span>
          <span class="badge {task.approval_status}">{task.approval_status}</span>
        </div>
        <div class="card mini">
          <span class="label">Planlama Durumu</span>
          <span class="badge {task.planning_status}">{task.planning_status}</span>
        </div>
      </div>

      <div class="card-row">
        <div class="card mini">
          <span class="label">Mevcut Kapı (Gate)</span>
          <span class="badge current-gate">{task.current_gate || "yok"}</span>
        </div>
        <div class="card mini">
          <span class="label">Oluşturulma</span>
          <span class="time">{createdAt}</span>
        </div>
      </div>
    </div>

    <div class="action-bar">
      {#if task.planning_status === 'planning_complete' && task.status !== 'completed'}
        <button class="btn execute-btn" onclick={onExecute}>Execution Engine Başlat (8 Kapı)</button>
      {:else if task.status === 'completed'}
        <span class="completed-msg">İşlem 8 kapıdan geçti ve tamamlandı.</span>
      {:else}
        <span class="warning-msg">İşlemin başlaması için Planlama Standardı (17/17) kilit açma formu doldurulmalıdır.</span>
      {/if}
    </div>
  {:else}
    <p class="no-task">Lütfen sol panelden bir işlem sekmesi seçin veya yeni bir görev başlatın.</p>
  {/if}
</div>

<style>
  .task-detail-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 2px solid #222;
    padding-bottom: 8px;
    margin-bottom: 12px;
  }
  h2 { margin: 0; font-size: 1.1rem; color: #fff; }
  .id { font-family: monospace; font-size: 0.75rem; color: #888; }
  .detail-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card {
    background: #252526;
    border: 1px solid #2d2d2d;
    padding: 10px;
    border-radius: 4px;
  }
  .card.mini {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .card-row {
    display: flex;
    gap: 10px;
  }
  .label { font-size: 0.7rem; text-transform: uppercase; color: #888; letter-spacing: 0.5px; }
  .value { font-size: 0.85rem; margin: 5px 0 0 0; color: #fff; line-height: 1.4; }
  
  .badge {
    font-size: 0.75rem;
    font-weight: bold;
    padding: 4px 8px;
    border-radius: 3px;
    text-transform: uppercase;
    width: fit-content;
  }
  .badge.completed { background: #4ec9b0; color: #1e1e1e; }
  .badge.failed { background: #f44747; color: #fff; }
  .badge.executing { background: #dcdcaa; color: #1e1e1e; }
  .badge.planning_incomplete { background: #ce9178; color: white; }
  .badge.planning_complete { background: #4ec9b0; color: #1e1e1e; }
  .badge.incomplete { background: #ce9178; color: white; }
  .badge.low { background: #4ec9b0; color: #1e1e1e; }
  .badge.medium { background: #dcdcaa; color: #1e1e1e; }
  .badge.high { background: #ce9178; color: white; }
  .badge.critical { background: #f44747; color: white; }
  .badge.current-gate { background: #007acc; color: white; }
  .badge.approved { background: #4ec9b0; color: #1e1e1e; }
  .badge.pending { background: #dcdcaa; color: #1e1e1e; }
  .badge.not_required { background: #555; color: #eee; }
  
  .time { font-size: 0.8rem; color: #eee; font-family: monospace; }
  .action-bar {
    margin-top: 15px;
    display: flex;
    justify-content: flex-end;
  }
  .btn {
    padding: 10px 20px;
    font-weight: bold;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .execute-btn { background: #0e639c; color: white; }
  .execute-btn:hover { background: #1177bb; }
  
  .completed-msg { color: #4ec9b0; font-weight: bold; font-size: 0.85rem; }
  .warning-msg { color: #ce9178; font-weight: bold; font-size: 0.85rem; }
  .no-task { text-align: center; color: #666; font-size: 0.9rem; padding: 40px 0; }
</style>
