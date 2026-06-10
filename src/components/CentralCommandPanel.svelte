<script lang="ts">
  let {
    providers = [],
    connectors = [],
    askerMotoruStatus = null,
    dbSizeBytes = 0,
    allocations = [],
    taskId = null,
    onRefresh
  }: {
    providers: any[];
    connectors: any[];
    askerMotoruStatus: any | null;
    dbSizeBytes: number;
    allocations: any[];
    taskId: string | null;
    onRefresh: () => void;
  } = $props();

  const enabledProviders = $derived(providers.filter((provider) => provider.enabled));
  const availableProviders = $derived(providers.filter((provider) => provider.status === "available"));
  const availableConnectors = $derived(connectors.filter((connector) => connector.status === "available" || connector.status === "read_only_configured"));
  const askerEndpoints = $derived(askerMotoruStatus?.api?.endpoints || []);
  const askerContract = $derived(askerMotoruStatus?.contract || []);
  const askerRoots = $derived(askerMotoruStatus?.roots || []);
  const askerFiles = $derived(askerMotoruStatus?.files || []);
  const availableAskerEndpoints = $derived(askerEndpoints.filter((endpoint: any) => endpoint.status === "available"));

  function statusClass(status: string) {
    if (["available", "ok", "read_only_configured"].includes(status)) return "ok";
    if (["disabled", "contract_only"].includes(status)) return "muted";
    return "warn";
  }

  function formatDbSize(bytes: number) {
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }
</script>

<section class="central-command-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">TEK MERKEZ PANEL</span>
      <h3>Command Center + AI Sağlayıcıları</h3>
      <p>AI, connector ve Asker Motoru health verileri aynı gerçek refresh akışından gelir.</p>
    </div>
    <button type="button" onclick={onRefresh}>Health-check</button>
  </div>

  <div class="summary-grid">
    <article>
      <span>AI Sağlayıcıları</span>
      <strong>{availableProviders.length}/{enabledProviders.length}</strong>
      <small>aktif sağlayıcı erişilebilir</small>
    </article>
    <article>
      <span>Sistem Connectorları</span>
      <strong>{availableConnectors.length}/{connectors.length}</strong>
      <small>bağlantı kullanılabilir</small>
    </article>
    <article>
      <span>Asker Motoru API</span>
      <strong>{availableAskerEndpoints.length}/{askerEndpoints.length}</strong>
      <small>{askerMotoruStatus?.api?.enabled ? "endpoint erişilebilir" : "API config kapalı"}</small>
    </article>
    <article>
      <span>Yerel Kayıt</span>
      <strong>{formatDbSize(dbSizeBytes)}</strong>
      <small>SQLite log/veri boyutu</small>
    </article>
  </div>

  <div class="section-grid">
    <article class="section-card">
      <div class="section-title">
        <span class="eyebrow">AI PROVIDERS</span>
        <strong>Gerçek Provider Health</strong>
      </div>
      {#if providers.length}
        <div class="row-list">
          {#each providers as provider}
            <div class="data-row">
              <div>
                <strong>{provider.name}</strong>
                <span>{provider.provider_type} / {provider.model}</span>
              </div>
              <b class={statusClass(provider.status)}>{provider.status}</b>
              <span>{provider.api_key_status}</span>
              <span>{provider.enabled ? "açık" : "kapalı"}</span>
              <small>{provider.last_error || "Son hata yok."}</small>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">Provider health verisi bekleniyor.</p>
      {/if}
    </article>

    <article class="section-card">
      <div class="section-title">
        <span class="eyebrow">CONNECTORS</span>
        <strong>Sistem Bağlantıları</strong>
      </div>
      {#if connectors.length}
        <div class="row-list">
          {#each connectors as connector}
            <div class="data-row">
              <div>
                <strong>{connector.name}</strong>
                <span>{connector.connector_type}</span>
              </div>
              <b class={statusClass(connector.status)}>{connector.status}</b>
              <span>{connector.read_only ? "read-only" : "write kapalı değil"}</span>
              <span>{connector.dependency_level}</span>
              <small>{connector.target || connector.last_error || "Hedef tanımlı değil."}</small>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">Connector health verisi bekleniyor.</p>
      {/if}
    </article>

    <article class="section-card wide">
      <div class="section-title">
        <span class="eyebrow">ASKER MOTORU</span>
        <strong>/health /status /events /command Sözleşmesi</strong>
      </div>
      <div class="contract-grid">
        {#each askerContract as endpoint}
          <div class="contract-item">
            <span>{endpoint.name}</span>
            <strong>{endpoint.method} {endpoint.path}</strong>
          </div>
        {/each}
      </div>
      <div class="asker-grid">
        <div>
          <h4>Kökler</h4>
          {#each askerRoots as root}
            <div class="mini-row" class:missing={!root.exists}>
              <strong>{root.id}</strong>
              <span>{root.role} · {root.resolved_path}</span>
            </div>
          {/each}
        </div>
        <div>
          <h4>Endpoint Durumu</h4>
          {#each askerEndpoints as endpoint}
            <div class="mini-row">
              <strong class={statusClass(endpoint.status)}>{endpoint.method} {endpoint.path}</strong>
              <span>{endpoint.status}{endpoint.http_status ? ` · HTTP ${endpoint.http_status}` : ""}</span>
            </div>
          {/each}
        </div>
      </div>
      {#if askerFiles.length}
        <div class="file-list">
          {#each askerFiles as file}
            <div class="asker-file" class:missing={!file.exists}>
              <strong>{file.root_id} / {file.name}</strong>
              <code>{file.path}</code>
              <pre>{file.preview}</pre>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">Config köklerinde durum dosyası bulunamadı veya kökler henüz erişilebilir değil.</p>
      {/if}
    </article>

    <article class="section-card wide">
      <div class="section-title">
        <span class="eyebrow">SWARM</span>
        <strong>Görev Ajan Dağılımı</strong>
      </div>
      {#if allocations.length}
        <div class="row-list compact">
          {#each allocations as allocation}
            <div class="data-row">
              <div>
                <strong>{allocation.platform}</strong>
                <span>{allocation.task_id}</span>
              </div>
              <b class={statusClass(allocation.status)}>{allocation.status}</b>
              <span>{allocation.payload_file_path || "payload yok"}</span>
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">{taskId ? "Seçili görev için swarm allocation yok." : "Görev seçilmedi."}</p>
      {/if}
    </article>
  </div>
</section>

<style>
  .central-command-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #151517;
    border-radius: 8px;
    margin-bottom: 16px;
  }

  .panel-header,
  .section-title {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  .panel-header {
    margin-bottom: 16px;
  }

  .eyebrow {
    display: block;
    color: #8d8d95;
    font-size: 11px;
    letter-spacing: 0.6px;
    text-transform: uppercase;
  }

  h3,
  h4,
  strong,
  b {
    color: #f4f4f5;
  }

  h3 {
    margin: 4px 0;
    font-size: 20px;
  }

  h4 {
    margin: 0 0 8px;
    font-size: 13px;
  }

  p,
  span,
  small,
  code,
  pre {
    color: #b8b8bf;
  }

  p {
    margin: 0;
  }

  button {
    background: #0b74de;
    color: white;
    border: 0;
    border-radius: 6px;
    padding: 9px 12px;
    cursor: pointer;
  }

  .summary-grid,
  .section-grid,
  .asker-grid,
  .contract-grid {
    display: grid;
    gap: 12px;
  }

  .summary-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin-bottom: 14px;
  }

  .summary-grid article,
  .section-card,
  .contract-item {
    background: #101012;
    border: 1px solid #2d2d31;
    border-radius: 7px;
  }

  .summary-grid article {
    padding: 12px;
  }

  .summary-grid strong {
    display: block;
    margin: 4px 0;
    font-size: 22px;
  }

  .section-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .section-card {
    padding: 14px;
    min-width: 0;
  }

  .wide {
    grid-column: 1 / -1;
  }

  .row-list {
    display: grid;
    gap: 8px;
    margin-top: 12px;
  }

  .data-row {
    display: grid;
    grid-template-columns: minmax(170px, 1.4fr) 120px 110px 110px minmax(180px, 1.6fr);
    gap: 10px;
    align-items: center;
    padding: 10px;
    background: #17171a;
    border: 1px solid #2b2b30;
    border-radius: 6px;
    font-size: 12px;
  }

  .compact .data-row {
    grid-template-columns: minmax(180px, 1fr) 120px minmax(200px, 1fr);
  }

  .data-row div,
  .mini-row {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }

  .muted {
    color: #8d8d95;
  }

  .empty {
    margin-top: 12px;
    color: #8d8d95;
    font-size: 13px;
  }

  .contract-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin: 12px 0;
  }

  .contract-item {
    padding: 10px;
  }

  .contract-item strong {
    display: block;
    margin-top: 4px;
    font-family: ui-monospace, monospace;
    font-size: 12px;
  }

  .asker-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 8px;
  }

  .mini-row {
    padding: 9px;
    border: 1px solid #2b2b30;
    background: #17171a;
    border-radius: 6px;
    margin-bottom: 8px;
    overflow-wrap: anywhere;
    font-size: 12px;
  }

  .missing {
    border-color: rgba(248, 193, 74, 0.55);
  }

  .file-list {
    display: grid;
    gap: 8px;
    margin-top: 12px;
  }

  .asker-file {
    padding: 10px;
    border: 1px solid #2b2b30;
    background: #111113;
    border-radius: 6px;
  }

  .asker-file code,
  .asker-file pre {
    display: block;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    margin: 6px 0 0;
    font-size: 12px;
  }

  @media (max-width: 1100px) {
    .summary-grid,
    .section-grid,
    .asker-grid,
    .contract-grid {
      grid-template-columns: 1fr;
    }

    .data-row,
    .compact .data-row {
      grid-template-columns: 1fr;
    }
  }
</style>
