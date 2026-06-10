<script lang="ts">
  let {
    tasks = [],
    selectedTask = null,
    providers = [],
    connectors = [],
    allocations = [],
    askerMotoruStatus = null,
    dbSizeBytes = 0,
    runtimeMode = "browser_preview",
    criticalAlarmCounter = 0,
    lastCriticalAlarmSource = null,
    lastCriticalAlarmAt = null,
    onRefresh,
    onOpenConnections,
    onNewTask,
    onSelectTask
  }: {
    tasks: any[];
    selectedTask: any | null;
    providers: any[];
    connectors: any[];
    allocations: any[];
    askerMotoruStatus: any | null;
    dbSizeBytes: number;
    runtimeMode: string;
    criticalAlarmCounter: number;
    lastCriticalAlarmSource: string | null;
    lastCriticalAlarmAt: string | null;
    onRefresh: () => void;
    onOpenConnections: () => void;
    onNewTask: () => void;
    onSelectTask: (id: string) => void;
  } = $props();

  const okStatuses = new Set(["available", "read_only_configured", "ok", "ready", "waiting"]);

  function countWhere(items: any[], predicate: (item: any) => boolean) {
    return items.filter(predicate).length;
  }

  function statusOf(value: string | undefined | null) {
    return String(value || "unknown").toLowerCase();
  }

  function taskCount(status: string) {
    return countWhere(tasks, task => statusOf(task.status) === status);
  }

  function providerReady(provider: any) {
    return provider.enabled && okStatuses.has(statusOf(provider.status));
  }

  function connectorReady(connector: any) {
    return connector.enabled && okStatuses.has(statusOf(connector.status));
  }

  function fileCount(exists: boolean) {
    return countWhere(askerMotoruStatus?.files || [], file => Boolean(file.exists) === exists);
  }

  function lastUpdatedAt() {
    const stamps = [
      ...providers.map(provider => provider.last_checked_at),
      ...connectors.map(connector => connector.last_checked_at)
    ].filter(Boolean);
    return stamps[0] || "henüz yok";
  }

  function formatBytes(bytes: number) {
    if (!bytes) return "0.00 MB";
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  function taskLabel(task: any) {
    const status = statusOf(task.status).replace("_", " ");
    return `${status} / ${task.current_gate || "kapı bekliyor"}`;
  }
</script>

<section class="command-center" aria-label="Asker Motoru merkez paneli">
  <div class="hero">
    <div>
      <span class="eyebrow">ASKER MOTORU MERKEZ PANEL</span>
      <h2>Ajanlar, sağlayıcılar, görevler ve durumlar tek komuta ekranında.</h2>
      <p>
        {runtimeMode === "tauri_runtime"
          ? "Tauri köprüsü aktif; health-check, veritabanı ve Asker Motoru durum dosyaları canlı izleniyor."
          : "Tarayıcı önizleme modu; panel gerçek Tauri runtime'a geçtiğinde aynı kanallardan canlı veri toplar."}
      </p>
    </div>
    <div class="hero-actions">
      <button type="button" class="primary" onclick={onRefresh}>Durumları Topla</button>
      <button type="button" onclick={onOpenConnections}>AI Sağlayıcıları</button>
      <button type="button" onclick={onNewTask}>Yeni Görev</button>
    </div>
  </div>

  <div class="metric-grid">
    <article class="metric-card accent-green">
      <span class="metric-label">Ajan İzleme</span>
      <strong>{allocations.length}</strong>
      <p>aktif swarm tahsisi</p>
      <small>{allocations.length ? allocations.map(item => `${item.platform}:${item.status}`).join(" | ") : "Henüz görev ajanlara tahsis edilmedi."}</small>
    </article>

    <article class="metric-card accent-blue">
      <span class="metric-label">AI Sağlayıcıları</span>
      <strong>{countWhere(providers, providerReady)} / {providers.length}</strong>
      <p>hazır provider</p>
      <small>{providers.length ? `${countWhere(providers, provider => provider.enabled)} etkin, ${countWhere(providers, provider => !provider.enabled)} kapalı` : "Provider health kaydı bekleniyor."}</small>
    </article>

    <article class="metric-card accent-amber">
      <span class="metric-label">Görevler</span>
      <strong>{tasks.length}</strong>
      <p>{taskCount("in_progress")} yürütülüyor, {taskCount("pending")} bekliyor, {taskCount("completed")} tamamlandı</p>
      <small>Seçili: {selectedTask?.title || "yok"}</small>
    </article>

    <article class="metric-card accent-red">
      <span class="metric-label">Durum Toplama</span>
      <strong>{countWhere(connectors, connectorReady)} / {connectors.length}</strong>
      <p>hazır connector</p>
      <small>Son kontrol: {lastUpdatedAt()}</small>
    </article>

    <article class="metric-card accent-purple">
      <span class="metric-label">Asker Motoru Köprüsü</span>
      <strong>{fileCount(true)} / {fileCount(true) + fileCount(false)}</strong>
      <p>durum dosyası bulundu</p>
      <small>DB: {formatBytes(dbSizeBytes)}</small>
    </article>
  </div>

  <div class="operations-grid">
    <article class="panel-block">
      <div class="block-header">
        <h3>Görev Radarı</h3>
        <span>{tasks.length} kayıt</span>
      </div>
      {#if tasks.length}
        <div class="task-radar">
          {#each tasks.slice(0, 5) as task}
            <button type="button" class:active={selectedTask?.id === task.id} onclick={() => onSelectTask(task.id)}>
              <strong>{task.title}</strong>
              <span>{taskLabel(task)}</span>
            </button>
          {/each}
        </div>
      {:else}
        <p class="empty">Henüz görev yok; yeni görev başlatıldığında merkez panel burada takip eder.</p>
      {/if}
    </article>

    <article class="panel-block">
      <div class="block-header">
        <h3>Sağlayıcı ve Connector Sağlığı</h3>
        <span>{providers.length + connectors.length} kanal</span>
      </div>
      <div class="health-lines">
        {#each providers.slice(0, 4) as provider}
          <div class="health-line">
            <span class:ok={providerReady(provider)} class:warn={!providerReady(provider)}></span>
            <strong>{provider.name}</strong>
            <small>{provider.status || "unknown"} / {provider.model || provider.provider_type}</small>
          </div>
        {/each}
        {#each connectors.slice(0, 4) as connector}
          <div class="health-line">
            <span class:ok={connectorReady(connector)} class:warn={!connectorReady(connector)}></span>
            <strong>{connector.name}</strong>
            <small>{connector.status || "unknown"} / {connector.connector_type}</small>
          </div>
        {/each}
        {#if providers.length + connectors.length === 0}
          <p class="empty">Health-check listesi boş; bağlantılar sekmesinden canlı kontrol çalıştırılabilir.</p>
        {/if}
      </div>
    </article>

    <article class="panel-block">
      <div class="block-header">
        <h3>Sistem Alarmı ve Bridge</h3>
        <span class:alarm={criticalAlarmCounter > 0}>{criticalAlarmCounter > 0 ? "alarm geçmişi" : "sakin"}</span>
      </div>
      <div class="bridge-lines">
        <div>
          <strong>Son alarm</strong>
          <span>{lastCriticalAlarmSource || "yok"} {lastCriticalAlarmAt ? `(${lastCriticalAlarmAt})` : ""}</span>
        </div>
        <div>
          <strong>Kök tarama</strong>
          <span>{askerMotoruStatus?.roots_checked?.length || 0} yol kontrol edildi</span>
        </div>
        <div>
          <strong>Durum dosyaları</strong>
          <span>{fileCount(true)} bulundu, {fileCount(false)} eksik</span>
        </div>
      </div>
    </article>
  </div>
</section>

<style>
  .command-center {
    flex: 0 0 auto;
    box-sizing: border-box;
    border: 1px solid #25252a;
    border-radius: 12px;
    background: linear-gradient(135deg, #18181a 0%, #121216 100%);
    color: #f4f4f5;
    margin: 15px 15px 20px;
    overflow: hidden;
    box-shadow: 0 20px 45px rgba(0, 0, 0, 0.24);
  }

  .hero {
    display: flex;
    justify-content: space-between;
    gap: 20px;
    padding: 22px;
    border-bottom: 1px solid #25252a;
    background:
      radial-gradient(circle at top left, rgba(11, 116, 222, 0.24), transparent 34%),
      radial-gradient(circle at top right, rgba(245, 158, 11, 0.16), transparent 30%);
  }

  .eyebrow,
  .metric-label,
  .block-header span {
    color: #8d8d95;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.9px;
    text-transform: uppercase;
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    max-width: 760px;
    margin-top: 6px;
    font-size: 28px;
    line-height: 1.1;
  }

  .hero p {
    max-width: 780px;
    margin-top: 10px;
    color: #b8b8bf;
    font-size: 14px;
  }

  .hero-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-content: flex-start;
    gap: 10px;
    min-width: 260px;
  }

  button {
    border: 1px solid #34343a;
    border-radius: 8px;
    background: #1d1d21;
    color: #f4f4f5;
    cursor: pointer;
    font: inherit;
    font-size: 13px;
    font-weight: 700;
    padding: 10px 12px;
  }

  button:hover {
    border-color: #0b74de;
  }

  button.primary {
    background: #0b74de;
    border-color: #0b74de;
    color: #fff;
  }

  .metric-grid {
    display: grid;
    grid-template-columns: repeat(5, minmax(130px, 1fr));
    gap: 12px;
    padding: 14px;
  }

  .metric-card {
    min-height: 128px;
    padding: 14px;
    border: 1px solid #2d2d31;
    border-top: 3px solid #3b3b40;
    border-radius: 10px;
    background: #111113;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .metric-card strong {
    font-size: 30px;
    line-height: 1;
  }

  .metric-card p,
  .metric-card small,
  .empty,
  .health-line small,
  .bridge-lines span,
  .task-radar span {
    color: #b8b8bf;
  }

  .metric-card small {
    font-size: 11px;
    overflow-wrap: anywhere;
  }

  .accent-green { border-top-color: #47d18c; }
  .accent-blue { border-top-color: #0b74de; }
  .accent-amber { border-top-color: #f8c14a; }
  .accent-red { border-top-color: #f44747; }
  .accent-purple { border-top-color: #a78bfa; }

  .operations-grid {
    display: grid;
    grid-template-columns: 1.1fr 1.1fr 0.8fr;
    gap: 14px;
    padding: 0 14px 14px;
  }

  .panel-block {
    border: 1px solid #2d2d31;
    border-radius: 10px;
    background: #151517;
    padding: 14px;
    min-height: 168px;
  }

  .block-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }

  .block-header h3 {
    font-size: 15px;
  }

  .block-header span.alarm {
    color: #f8c14a;
  }

  .task-radar,
  .health-lines,
  .bridge-lines {
    display: grid;
    gap: 8px;
  }

  .task-radar button {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    background: #111113;
    text-align: left;
  }

  .task-radar button.active {
    border-color: #0b74de;
    box-shadow: inset 3px 0 0 #0b74de;
  }

  .task-radar strong,
  .health-line strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .health-line {
    display: grid;
    grid-template-columns: 10px minmax(120px, 1fr) minmax(120px, 1fr);
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: 8px;
    background: #111113;
  }

  .health-line > span:first-child {
    width: 9px;
    height: 9px;
    border-radius: 50%;
  }

  .health-line .ok {
    background: #47d18c;
    box-shadow: 0 0 10px rgba(71, 209, 140, 0.6);
  }

  .health-line .warn {
    background: #f8c14a;
    box-shadow: 0 0 10px rgba(248, 193, 74, 0.5);
  }

  .bridge-lines div {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px;
    border-radius: 8px;
    background: #111113;
  }

  @media (max-width: 1280px) {
    .metric-grid,
    .operations-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .hero {
      flex-direction: column;
    }

    .hero-actions {
      justify-content: flex-start;
    }
  }
</style>
