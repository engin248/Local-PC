<script lang="ts">
  import ReportPanel from "./ReportPanel.svelte";

  let {
    reports = [],
    liveMode = false,
    voiceRepliesEnabled = false,
    onSpeakReport,
  } = $props<{
    reports: any[];
    liveMode?: boolean;
    voiceRepliesEnabled?: boolean;
    onSpeakReport?: (text: string, key?: string) => void;
  }>();

  const requiredMarkers = [
    "## A. Çözümleme Raporu",
    "## B. Uygulama Planı",
    "## C. Uygulama İzleme Raporu"
  ];

  function completeness(content: string) {
    const found = requiredMarkers.filter((m) => content.includes(m)).length;
    return Math.round((found / requiredMarkers.length) * 100);
  }
</script>

<div class="structured-report-panel" class:live={liveMode}>
  <header>
    <h3>{liveMode ? "Rapor Paneli" : "Yapılandırılmış Nihai Rapor"}</h3>
    <p>{liveMode ? "Tamamlanan görev raporları canlı görünür." : "Zorunlu bölümler: Çözümleme, Uygulama Planı, Uygulama İzleme."}</p>
  </header>
  {#if reports.length > 0}
    {#each reports as rep}
      <div class="score-row">
        <div class="score">Tamamlanma: %{completeness(rep.content)}</div>
        {#if liveMode && voiceRepliesEnabled && onSpeakReport}
          <button
            class="speak-btn"
            onclick={() => onSpeakReport?.(`Rapor hazır. ${rep.title || "Görev raporu"}.`, `report:${rep.id}`)}
          >
            Sesli Oku
          </button>
        {/if}
      </div>
    {/each}
  {:else if liveMode}
    <p class="empty">Henüz dönen rapor yok.</p>
  {/if}
  <ReportPanel {reports} />
</div>

<style>
  header { margin-bottom: 0.75rem; }
  header h3 { margin: 0 0 0.25rem; }
  header p { margin: 0; color: #888; font-size: 0.85rem; }
  .score-row { display: flex; gap: 8px; align-items: center; margin-bottom: 0.5rem; }
  .score { font-size: 0.85rem; color: #6cf; }
  .speak-btn {
    background: #1f7a57;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .empty { color: #888; font-size: 0.85rem; }
  .structured-report-panel.live { border: 1px solid #2f4a2f; border-radius: 8px; padding: 14px; background: #101410; }
</style>
