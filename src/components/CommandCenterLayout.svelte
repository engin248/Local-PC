<script lang="ts">
  import CommandCenterPanel from "./CommandCenterPanel.svelte";
  import ColonelBurhanPanel from "./ColonelBurhanPanel.svelte";
  import SwarmMonitorPanel from "./SwarmMonitorPanel.svelte";
  import StructuredReportPanel from "./StructuredReportPanel.svelte";

  let {
    commandFeed = [],
    burhanEvents = [],
    lastBurhanDispatch = null,
    selectedTaskId = null,
    swarmAllocations = [],
    reports = [],
    voiceRepliesEnabled = true,
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
    onCommandSubmitted?: (result: any) => void;
    onSpeakReport?: (text: string, key?: string) => void;
  }>();
</script>

<section class="command-center-layout">
  <div class="grid">
    <CommandCenterPanel {onCommandSubmitted} liveFeed={commandFeed} />
    <ColonelBurhanPanel
      burhanEvents={burhanEvents}
      lastDispatch={lastBurhanDispatch}
      taskId={selectedTaskId}
    />
    <SwarmMonitorPanel allocations={swarmAllocations} taskId={selectedTaskId} liveMode={true} />
    <StructuredReportPanel {reports} {voiceRepliesEnabled} {onSpeakReport} liveMode={true} />
  </div>
</section>

<style>
  .command-center-layout {
    margin-bottom: 18px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }
  @media (max-width: 1100px) {
    .grid { grid-template-columns: 1fr; }
  }
</style>
