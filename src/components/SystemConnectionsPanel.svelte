<script lang="ts">
  let {
    connectors = [],
    onRefresh
  }: {
    connectors: any[];
    onRefresh: () => void;
  } = $props();
</script>

<section class="connection-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">SİSTEM BAĞLANTILARI</span>
      <h3>Connector Health</h3>
    </div>
    <button type="button" onclick={onRefresh}>Health-check</button>
  </div>

  <div class="connection-grid">
    {#each connectors as connector}
      <article class="connection-row">
        <div>
          <strong>{connector.name}</strong>
          <span>{connector.connector_type}</span>
        </div>
        <div>
          <span class="label">Durum</span>
          <b class:ok={connector.status === "available" || connector.status === "read_only_configured"} class:warn={connector.status !== "available" && connector.status !== "read_only_configured"}>{connector.status}</b>
        </div>
        <div>
          <span class="label">Read-only</span>
          <b>{connector.read_only ? "aktif" : "kapalı"}</b>
        </div>
        <div>
          <span class="label">Enabled</span>
          <b>{connector.enabled ? "açık" : "kapalı"}</b>
        </div>
        <div>
          <span class="label">Risk</span>
          <b>{connector.dependency_level}</b>
        </div>
        <div class="wide">
          <span class="label">Hedef</span>
          <code>{connector.target || "tanımlı değil"}</code>
        </div>
        <div class="wide">
          <span class="label">Approval / Rollback / Test</span>
          <span>{connector.approval_required_actions.join(", ") || "yok"} / {connector.rollback_required_actions.join(", ") || "yok"} / {connector.test_required_actions.join(", ") || "yok"}</span>
        </div>
        <p>{connector.last_error || "Son hata yok."}</p>
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
    grid-template-columns: minmax(180px, 1.4fr) repeat(4, minmax(90px, 0.7fr));
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
  p,
  code {
    color: #b8b8bf;
    margin: 0;
  }

  code {
    overflow-wrap: anywhere;
  }

  .wide,
  p {
    grid-column: 1 / -1;
  }

  p {
    font-size: 12px;
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }
</style>
