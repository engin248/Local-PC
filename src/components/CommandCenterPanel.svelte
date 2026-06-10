<script lang="ts">
  let {
    snapshot = null,
    swarmAllocations = [],
    taskId = null,
    alarmEvents = [],
    auditTrail = [],
    onRefresh
  }: {
    snapshot: any | null;
    swarmAllocations: any[];
    taskId: string | null;
    alarmEvents: any[];
    auditTrail: any[];
    onRefresh: () => void;
  } = $props();

  const formatBytes = (bytes: number | undefined) => {
    if (!bytes) return "0.00 MB";
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };
</script>

<section class="command-center-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">TEK MERKEZ PANEL</span>
      <h3>Command Center + AI Sağlayıcıları</h3>
      <p>Asker Motoru sözleşmesi, sağlayıcı health ve bağlantı durumu tek snapshot üzerinden izlenir.</p>
    </div>
    <button type="button" onclick={onRefresh}>Merkezi health-check</button>
  </div>

  {#if !snapshot}
    <p class="empty">Merkez snapshot bekleniyor.</p>
  {:else}
    <div class="summary-grid">
      <article>
        <span class="label">Asker Motoru</span>
        <strong class:ok={snapshot.asker_motoru.health.status === "available"} class:warn={snapshot.asker_motoru.health.status !== "available"}>
          {snapshot.asker_motoru.health.status}
        </strong>
        <small>{snapshot.asker_motoru.health.roots_available}/{snapshot.asker_motoru.health.roots_total} kök erişilebilir</small>
      </article>
      <article>
        <span class="label">AI sağlayıcı</span>
        <strong>{snapshot.ai_providers.filter((provider: any) => provider.enabled).length}/{snapshot.ai_providers.length}</strong>
        <small>aktif yapılandırma</small>
      </article>
      <article>
        <span class="label">Connector</span>
        <strong>{snapshot.system_connectors.filter((connector: any) => connector.status === "available").length}/{snapshot.system_connectors.length}</strong>
        <small>erişilebilir bağlantı</small>
      </article>
      <article>
        <span class="label">DB boyutu</span>
        <strong>{formatBytes(snapshot.db_size_bytes)}</strong>
        <small>{snapshot.generated_at}</small>
      </article>
    </div>

    <div class="contract-grid">
      {#each [
        snapshot.asker_motoru.contract.health,
        snapshot.asker_motoru.contract.status,
        snapshot.asker_motoru.contract.events,
        snapshot.asker_motoru.contract.command
      ] as endpoint}
        <article>
          <span class="method">{endpoint.method}</span>
          <strong>{endpoint.path}</strong>
          <p>{endpoint.description}</p>
          {#if endpoint.allowed_commands?.length}
            <small>{endpoint.allowed_commands.join(", ")}</small>
          {/if}
        </article>
      {/each}
    </div>

    <div class="split-grid">
      <section>
        <h4>AI Sağlayıcıları</h4>
        <div class="rows">
          {#each snapshot.ai_providers as provider}
            <article class="row">
              <div>
                <strong>{provider.name}</strong>
                <span>{provider.provider_type} / {provider.model}</span>
              </div>
              <b class:ok={provider.status === "available"} class:warn={provider.status !== "available"}>{provider.status}</b>
              <small>{provider.api_key_status} · {provider.dependency_level}</small>
            </article>
          {/each}
        </div>
      </section>

      <section>
        <h4>Asker Motoru Health / Status</h4>
        <div class="rows">
          {#each snapshot.asker_motoru.roots as root}
            <article class="row">
              <div>
                <strong>{root.label}</strong>
                <span>{root.path}</span>
              </div>
              <b class:ok={root.exists} class:warn={!root.exists}>{root.exists ? "var" : "yok"}</b>
              <small>{root.required ? "zorunlu" : "opsiyonel"}</small>
            </article>
          {/each}
          {#each snapshot.asker_motoru.files as file}
            <article class="status-file" class:missing={!file.exists}>
              <strong>{file.path}</strong>
              <pre>{file.preview}</pre>
            </article>
          {/each}
        </div>
      </section>
    </div>

    <div class="split-grid">
      <section>
        <h4>Olay Akışı</h4>
        <div class="rows">
          {#each snapshot.asker_motoru.events.sources as event}
            <article class="status-file" class:missing={!event.exists}>
              <strong>{event.path}</strong>
              <pre>{event.preview}</pre>
            </article>
          {/each}
          {#each alarmEvents.slice(0, 4) as alarm}
            <article class="row">
              <div>
                <strong>{alarm.source}</strong>
                <span>{alarm.message}</span>
              </div>
              <small>{alarm.createdAt}</small>
            </article>
          {/each}
          {#if snapshot.asker_motoru.events.sources.length === 0 && alarmEvents.length === 0}
            <p class="empty">Olay kaynağı yok.</p>
          {/if}
        </div>
      </section>

      <section>
        <h4>Komut Yüzeyi</h4>
        <article class="command-card">
          <span class="method">{snapshot.asker_motoru.command.method}</span>
          <strong>{snapshot.asker_motoru.command.path}</strong>
          <p>{snapshot.asker_motoru.command.allowed_commands.join(", ")}</p>
          <small>{snapshot.asker_motoru.command.requires_approval ? "onay gerekli" : "onaysız"} · {snapshot.asker_motoru.command.available ? "hazır" : "kök bekliyor"}</small>
        </article>
        <h4>Swarm Tahsisi</h4>
        {#if !taskId}
          <p class="empty">Görev seçin; platform tahsisleri burada görünür.</p>
        {:else if swarmAllocations.length === 0}
          <p class="empty">Bu görev için henüz swarm tahsisi yok.</p>
        {:else}
          <div class="rows">
            {#each swarmAllocations as row}
              <article class="row">
                <div>
                  <strong>{row.platform}</strong>
                  <span>{row.payload_path}</span>
                </div>
                <b>{row.status}</b>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <section>
      <h4>Sistem Connectorları</h4>
      <div class="rows">
        {#each snapshot.system_connectors as connector}
          <article class="row">
            <div>
              <strong>{connector.name}</strong>
              <span>{connector.connector_type} · {connector.target || "hedef yok"}</span>
            </div>
            <b class:ok={connector.status === "available"} class:warn={connector.status !== "available"}>{connector.status}</b>
            <small>{connector.read_only ? "read-only" : "write"} · {connector.dependency_level}</small>
          </article>
        {/each}
      </div>
    </section>

    <section>
      <h4>Son Operasyon Kayıtları</h4>
      <div class="rows">
        {#each auditTrail.slice(0, 5) as audit}
          <article class="row">
            <div>
              <strong>{audit.action}</strong>
              <span>{audit.details}</span>
            </div>
            <b class:ok={audit.status === "PASS"} class:warn={audit.status !== "PASS"}>{audit.status}</b>
            <small>{audit.created_at}</small>
          </article>
        {/each}
      </div>
    </section>
  {/if}
</section>

<style>
  .command-center-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 8px;
    margin-bottom: 16px;
    color: #d9dce3;
  }

  .panel-header,
  .summary-grid,
  .contract-grid,
  .split-grid {
    display: grid;
    gap: 12px;
  }

  .panel-header {
    grid-template-columns: 1fr auto;
    align-items: start;
    margin-bottom: 16px;
  }

  .summary-grid {
    grid-template-columns: repeat(4, minmax(140px, 1fr));
    margin-bottom: 14px;
  }

  .contract-grid,
  .split-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-bottom: 14px;
  }

  article,
  .command-card {
    padding: 12px;
    background: #111113;
    border: 1px solid #2d2d31;
    border-radius: 6px;
  }

  h3,
  h4,
  strong,
  b {
    color: #f4f4f5;
  }

  h3,
  h4,
  p {
    margin: 0;
  }

  h4 {
    margin-bottom: 8px;
  }

  button {
    background: #0b74de;
    color: white;
    border: 0;
    border-radius: 6px;
    padding: 9px 12px;
    cursor: pointer;
  }

  .eyebrow,
  .label,
  small,
  span {
    color: #9ca3af;
  }

  .eyebrow,
  .label,
  .method {
    display: block;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .method {
    color: #8fdaff;
  }

  .rows {
    display: grid;
    gap: 8px;
  }

  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: 12px;
    align-items: center;
  }

  .row span,
  .status-file strong {
    display: block;
    overflow-wrap: anywhere;
  }

  .status-file pre {
    max-height: 120px;
    overflow: auto;
    white-space: pre-wrap;
    color: #c9d1d9;
    font-size: 11px;
    margin: 8px 0 0;
  }

  .missing {
    border-color: #5a3a20;
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }

  .empty {
    color: #8d8d95;
  }

  @media (max-width: 980px) {
    .panel-header,
    .summary-grid,
    .contract-grid,
    .split-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
