<script lang="ts">
  let { allocations = [], taskId = null } = $props<{
    allocations: any[];
    taskId: string | null;
  }>();
</script>

<div class="swarm-panel">
  <h3>Swarm Tahsis Durumu</h3>
  {#if !taskId}
    <p class="empty-msg">Görev seçin; platform inbox tahsisleri burada görünür.</p>
  {:else if allocations.length === 0}
    <p class="empty-msg">Bu görev için henüz swarm tahsisi yok.</p>
  {:else}
    <ul>
      {#each allocations as row}
        <li>
          <div class="swarm-head">
            <strong>{row.platform}</strong>
            <span class="status">{row.status}</span>
            <span class="source">{row.source_kind || "unavailable"}</span>
          </div>
          <div class="meta">Task: {row.task_status || "bilinmiyor"} / Worker: {row.worker_status || "heartbeat_missing"} / Rapor: {row.report_returned ? "döndü" : "bekleniyor"}</div>
          <div class="path">Inbox: {row.inbox_path || row.payload_path} ({row.inbox_exists ? "var" : "yok"})</div>
          <div class="path">Outbox: {row.outbox_path || "tanımlı değil"} ({row.outbox_exists ? "var" : "yok"})</div>
          <div class="path">Son rapor: {row.last_report_at || "yok"}</div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .swarm-panel { padding: 0.5rem 0; }
  .empty-msg { color: #888; font-size: 0.9rem; }
  ul { list-style: none; padding: 0; margin: 0; }
  li { padding: 0.5rem 0; border-bottom: 1px solid #333; }
  .swarm-head { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .status,
  .source {
    padding: 2px 6px;
    border-radius: 999px;
    background: #202026;
    color: #f8c14a;
    font-size: 0.72rem;
    font-weight: 800;
  }
  .source { color: #9fd3ff; }
  .meta { font-size: 0.78rem; color: #c8c8cc; margin-top: 4px; }
  .path { font-size: 0.75rem; color: #aaa; word-break: break-all; }
</style>
