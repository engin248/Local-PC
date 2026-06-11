<script lang="ts">
  import FounderAssignmentPanel from "./FounderAssignmentPanel.svelte";
  import ColonelBurhanPanel from "./ColonelBurhanPanel.svelte";
  import SwarmMonitorPanel from "./SwarmMonitorPanel.svelte";
  import StructuredReportPanel from "./StructuredReportPanel.svelte";
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
    onCommandSubmitted,
    onSpeakReport,
  } = $props<{
    commandFeed?: any[];
    burhanEvents?: any[];
    lastBurhanDispatch?: string | null;
    selectedTaskId?: string | null;
    swarmAllocations?: any[];
    reports?: any[];
    voiceRepliesEnabled?: boolean;
    flowStage?: CommandFlowStage;
    onCommandSubmitted?: (result: any) => void;
    onSpeakReport?: (text: string, key?: string) => void;
  }>();

  const downstreamUnlocked = $derived(flowStage !== "awaiting_task");
</script>

<section class="command-center-layout">
  <FounderAssignmentPanel
    {onCommandSubmitted}
    liveFeed={commandFeed}
    {flowStage}
  />

  <div class="downstream-grid" class:locked={!downstreamUnlocked}>
    <div class="panel-slot" class:locked={!downstreamUnlocked}>
      <span class="slot-badge">Panel 2</span>
      <ColonelBurhanPanel
        burhanEvents={burhanEvents}
        lastDispatch={lastBurhanDispatch}
        taskId={selectedTaskId}
      />
      {#if !downstreamUnlocked}
        <div class="lock-overlay">Görev atandıktan sonra açılır</div>
      {/if}
    </div>

    <div class="panel-slot" class:locked={!downstreamUnlocked}>
      <span class="slot-badge">Panel 3</span>
      <SwarmMonitorPanel allocations={swarmAllocations} taskId={selectedTaskId} liveMode={true} />
      {#if !downstreamUnlocked}
        <div class="lock-overlay">Görev atandıktan sonra açılır</div>
      {/if}
    </div>

    <div class="panel-slot wide" class:locked={!downstreamUnlocked}>
      <span class="slot-badge">Panel 4</span>
      <StructuredReportPanel {reports} {voiceRepliesEnabled} {onSpeakReport} liveMode={true} />
      {#if !downstreamUnlocked}
        <div class="lock-overlay">Görev atandıktan sonra açılır</div>
      {/if}
    </div>
  </div>
</section>

<style>
  .command-center-layout { margin-bottom: 18px; display: grid; gap: 14px; }
  .downstream-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }
  .panel-slot {
    position: relative;
    min-height: 180px;
  }
  .panel-slot.wide { grid-column: 1 / -1; }
  .slot-badge {
    display: inline-block;
    margin-bottom: 6px;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: #8aa0a8;
  }
  .lock-overlay {
    position: absolute;
    inset: 18px 0 0 0;
    display: grid;
    place-items: center;
    background: rgba(10, 12, 14, 0.72);
    color: #dce5ea;
    border-radius: 8px;
    font-size: 0.86rem;
    font-weight: 600;
    text-align: center;
    padding: 12px;
    z-index: 2;
  }
  .panel-slot.locked :global(> :not(.lock-overlay):not(.slot-badge)) {
    filter: grayscale(0.35);
    opacity: 0.55;
    pointer-events: none;
  }
  @media (max-width: 1100px) {
    .downstream-grid { grid-template-columns: 1fr; }
    .panel-slot.wide { grid-column: auto; }
  }
</style>
