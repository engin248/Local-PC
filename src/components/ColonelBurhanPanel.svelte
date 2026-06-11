<script lang="ts">
  let {
    burhanEvents = [],
    lastDispatch = null,
    taskId = null,
  } = $props<{
    burhanEvents?: any[];
    lastDispatch?: string | null;
    taskId?: string | null;
  }>();
</script>

<div class="burhan-panel">
  <header>
    <h3>Albay Burhan Komuta Hattı</h3>
    <p>Emir alır, yapay zekâlara dağıtır, disiplinle yönetir.</p>
  </header>
  <div class="status-row">
    <span class="badge active">AKTİF</span>
    <span>Görev: {taskId || "bekleniyor"}</span>
  </div>
  {#if lastDispatch}
    <div class="dispatch-box">
      <strong>Son Dağıtım</strong>
      <p>{lastDispatch}</p>
    </div>
  {/if}
  <div class="event-list">
    <h4>Canlı Emir Akışı</h4>
    {#if burhanEvents.length === 0}
      <p class="empty">Albay Burhan hattında henüz olay yok.</p>
    {:else}
      <ul>
        {#each burhanEvents.slice(0, 10) as event}
          <li>
            <span>{event.timestamp || event.created_at}</span>
            <strong>{event.source}</strong>
            <p>{event.message}</p>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .burhan-panel {
    border: 1px solid #3a2f1f;
    background: linear-gradient(180deg, #1a1410 0%, #121010 100%);
    border-radius: 8px;
    padding: 14px;
    color: #f6f0e8;
  }
  header h3 { margin: 0 0 4px; color: #f8d48a; }
  header p { margin: 0 0 10px; color: #b9aa92; font-size: 0.85rem; }
  .status-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; font-size: 0.85rem; }
  .badge {
    padding: 2px 8px;
    border-radius: 999px;
    background: #3a3020;
    color: #f8d48a;
    font-weight: 800;
    font-size: 0.72rem;
  }
  .badge.active { background: #5a4020; }
  .dispatch-box {
    background: #201810;
    border: 1px solid #4a3828;
    border-radius: 6px;
    padding: 10px;
    margin-bottom: 10px;
  }
  .dispatch-box p { margin: 6px 0 0; color: #e8dcc8; }
  .event-list h4 { margin: 0 0 8px; font-size: 0.9rem; }
  ul { list-style: none; margin: 0; padding: 0; max-height: 180px; overflow: auto; }
  li {
    padding: 6px 0;
    border-bottom: 1px solid #2d2418;
    font-size: 0.8rem;
  }
  li p { margin: 4px 0 0; color: #d8ccb8; }
  .empty { color: #8f8272; font-size: 0.85rem; }
</style>
