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
          <strong>{row.platform}</strong> — {row.status}
          <div class="path">{row.payload_path}</div>
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
  .path { font-size: 0.75rem; color: #aaa; word-break: break-all; }
</style>
