<script lang="ts">
  import CommandCenterLayout from "./CommandCenterLayout.svelte";
  import AskerModuleInventoryPanel from "./AskerModuleInventoryPanel.svelte";
  import AIConnectionsPanel from "./AIConnectionsPanel.svelte";
  import SystemConnectionsPanel from "./SystemConnectionsPanel.svelte";
  import AlarmCardsPanel from "./AlarmCardsPanel.svelte";
  import type { CommandFlowStage } from "../lib/commandFlow";

  let {
    commandFeed = [],
    burhanEvents = [],
    lastBurhanDispatch = null,
    selectedTaskId = null,
    swarmAllocations = [],
    reports = [],
    voiceRepliesEnabled = true,
    flowStage = "awaiting_task",
    providers = [],
    connectors = [],
    alarms = [],
    askerMotoruStatus = null,
    askerMotoruLiveStatus = null,
    dbSizeBytes = 0,
    activeAlarmEvents = [],
    onCommandSubmitted,
    onSpeakReport,
    onRefresh,
  } = $props<{
    commandFeed?: any[];
    burhanEvents?: any[];
    lastBurhanDispatch?: string | null;
    selectedTaskId?: string | null;
    swarmAllocations?: any[];
    reports?: any[];
    voiceRepliesEnabled?: boolean;
    flowStage?: CommandFlowStage;
    providers?: any[];
    connectors?: any[];
    alarms?: any[];
    askerMotoruStatus?: any | null;
    askerMotoruLiveStatus?: any | null;
    dbSizeBytes?: number;
    activeAlarmEvents?: any[];
    onCommandSubmitted?: (result: any) => void;
    onSpeakReport?: (text: string, key?: string) => void;
    onRefresh?: () => void;
  }>();
</script>

<section class="kontrol-departmani" aria-label="Kontrol Departmanı komuta paneli">
  <header class="dept-header">
    <div>
      <span class="eyebrow">KONTROL DEPARTMANI</span>
      <h2>Asker Motoru Komuta Merkezi</h2>
      <p>
        Kurucu görev ataması, Albay Burhan, ajan emirleri, raporlar, 314 modül envanteri ve bağlantı
        durumları tek sayfada izlenir.
      </p>
    </div>
    <button type="button" class="refresh-btn" onclick={() => onRefresh?.()}>
      Durumları Yenile
    </button>
  </header>

  <CommandCenterLayout
    {commandFeed}
    {burhanEvents}
    {lastBurhanDispatch}
    {selectedTaskId}
    {swarmAllocations}
    {reports}
    {voiceRepliesEnabled}
    {flowStage}
    {onCommandSubmitted}
    {onSpeakReport}
  />

  <AskerModuleInventoryPanel />

  <div class="connections-block">
    <h3>Bağlantı ve Sağlık Durumu</h3>
    <AIConnectionsPanel {providers} onRefresh={() => onRefresh?.()} />
    <SystemConnectionsPanel {connectors} onRefresh={() => onRefresh?.()} />
    <AlarmCardsPanel {alarms} />
  </div>

  {#if activeAlarmEvents.length > 0}
    <div class="alarm-code-panel">
      <h3>Aktif Alarm Kodları</h3>
      {#each activeAlarmEvents as alarm}
        <div class="alarm-code-item">
          <strong>{alarm.alarm_code || "000"}</strong>
          <span>{alarm.source}</span>
          <p>{alarm.message}</p>
        </div>
      {/each}
    </div>
  {/if}

  {#if askerMotoruLiveStatus}
    <div class="asker-live-panel">
      <h3>Asker Motoru Canlı API</h3>
      <p>
        Durum: {askerMotoruLiveStatus.health} / Bağlı:
        {askerMotoruLiveStatus.connected ? "evet" : "hayır"}
      </p>
      {#if askerMotoruLiveStatus.last_error}
        <pre>{askerMotoruLiveStatus.last_error}</pre>
      {/if}
    </div>
  {/if}

  {#if askerMotoruStatus}
    <div class="asker-bridge-panel">
      <h3>Asker Motoru Durum Köprüsü</h3>
      <p>DB boyutu: {(dbSizeBytes / (1024 * 1024)).toFixed(2)} MB</p>
      {#each askerMotoruStatus.root_sources || [] as root}
        <div class="asker-file" class:missing={root.health !== "available"}>
          <strong>{root.kind}: {root.source_path || "bağlı değil"}</strong>
          <span>{root.source_kind} / {root.health}</span>
          <pre>{root.error || "Kaynak erişilebilir."}</pre>
        </div>
      {/each}
      {#each askerMotoruStatus.files as file}
        <div class="asker-file" class:missing={!file.exists}>
          <strong>{file.path}</strong>
          <span>{file.source_kind} / {file.health}</span>
          <pre>{file.preview}</pre>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .kontrol-departmani {
    display: grid;
    gap: 14px;
    margin-bottom: 18px;
  }
  .dept-header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
    padding: 18px;
    border: 1px solid #244a66;
    border-radius: 8px;
    background: linear-gradient(135deg, #101820 0%, #141418 100%);
    color: #f4f4f5;
  }
  .eyebrow {
    display: block;
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    color: #9fd3ff;
    margin-bottom: 6px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 1.35rem;
  }
  .dept-header p {
    margin: 0;
    color: #a8b0b8;
    font-size: 0.86rem;
    line-height: 1.45;
    max-width: 720px;
  }
  .refresh-btn {
    background: #0b74de;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 10px 16px;
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
  }
  .connections-block h3 {
    margin: 0 0 10px;
    color: #f2f2f4;
    font-size: 1rem;
  }
  .alarm-code-panel,
  .asker-live-panel,
  .asker-bridge-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 6px;
    color: #f4f4f5;
  }
  .alarm-code-item {
    padding: 8px;
    margin-top: 8px;
    border: 1px solid #5a2020;
    border-radius: 6px;
    background: #1a1010;
  }
  .alarm-code-item strong {
    color: #ff8a8a;
    margin-right: 8px;
  }
  .asker-live-panel {
    border-color: #2f4a66;
    background: #101820;
  }
  .asker-file {
    padding: 10px;
    margin-top: 8px;
    border: 1px solid #2d2d31;
    border-radius: 6px;
    background: #111113;
  }
  .asker-file.missing {
    border-color: rgba(248, 193, 74, 0.35);
  }
  .asker-file strong,
  .asker-file span {
    display: block;
    color: #dfe4ec;
    overflow-wrap: anywhere;
  }
  .asker-file span {
    width: fit-content;
    margin-top: 4px;
    padding: 2px 6px;
    border: 1px solid #3b3b40;
    border-radius: 999px;
    color: #9fd3ff;
    font-size: 10px;
    font-weight: 800;
  }
  .asker-file pre {
    margin: 8px 0 0;
    color: #b8b8bf;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: 12px;
  }
</style>
