<script lang="ts">
  let {
    providers = [],
    onRefresh
  }: {
    providers: any[];
    onRefresh: () => void;
  } = $props();
</script>

<section class="connection-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">AI SAĞLAYICILARI VE YEREL AJANLAR</span>
      <h3>Sağlayıcı / Ajan Sağlığı</h3>
    </div>
    <button type="button" onclick={onRefresh}>Health-check</button>
  </div>

  <div class="connection-grid">
    {#each providers as provider}
      <article class="connection-row">
        <div>
          <strong>{provider.name}</strong>
          <span>{provider.provider_type} / {provider.model}</span>
        </div>
        <div>
          <span class="label">Durum</span>
          <b class:ok={provider.status === "available"} class:warn={provider.status !== "available"}>{provider.status}</b>
        </div>
        <div>
          <span class="label">API Key</span>
          <b>{provider.api_key_status}</b>
        </div>
        <div>
          <span class="label">Enabled</span>
          <b>{provider.enabled ? "açık" : "kapalı"}</b>
        </div>
        <div>
          <span class="label">Bağımlılık</span>
          <b>{provider.dependency_level}</b>
        </div>
        <div class="wide">
          <span class="label">Görev Yetkinlikleri</span>
          <span>{provider.allowed_task_types?.join(", ") || "tanımlı değil"}</span>
        </div>
        <p>{provider.last_error || "Son hata yok."}</p>
      </article>
    {/each}
  </div>
</section>

<style>
  .connection-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 6px;
    margin-bottom: 16px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: center;
    margin-bottom: 14px;
  }

  .eyebrow,
  .label {
    display: block;
    color: #8d8d95;
    font-size: 11px;
    text-transform: uppercase;
  }

  h3 {
    margin: 4px 0 0;
    color: #f2f2f4;
    font-size: 18px;
  }

  button {
    background: #0b74de;
    color: white;
    border: 0;
    border-radius: 6px;
    padding: 9px 12px;
    cursor: pointer;
  }

  .connection-grid {
    display: grid;
    gap: 10px;
  }

  .connection-row {
    display: grid;
    grid-template-columns: minmax(180px, 1.6fr) repeat(4, minmax(90px, 0.8fr));
    gap: 12px;
    align-items: center;
    padding: 12px;
    background: #111113;
    border: 1px solid #2d2d31;
    border-radius: 6px;
  }

  strong,
  b {
    color: #f4f4f5;
  }

  span,
  p {
    color: #b8b8bf;
    margin: 0;
  }

  p {
    grid-column: 1 / -1;
    font-size: 12px;
  }

  .wide {
    grid-column: 1 / -1;
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }
</style>
