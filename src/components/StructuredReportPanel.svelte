<script lang="ts">
  import ReportPanel from "./ReportPanel.svelte";

  let { reports = [] } = $props<{ reports: any[] }>();

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

<div class="structured-report-panel">
  <header>
    <h3>Yapılandırılmış Nihai Rapor</h3>
    <p>Zorunlu bölümler: Çözümleme, Uygulama Planı, Uygulama İzleme.</p>
  </header>
  {#if reports.length > 0}
    {#each reports as rep}
      <div class="score">Tamamlanma: %{completeness(rep.content)}</div>
    {/each}
  {/if}
  <ReportPanel {reports} />
</div>

<style>
  header { margin-bottom: 0.75rem; }
  header h3 { margin: 0 0 0.25rem; }
  header p { margin: 0; color: #888; font-size: 0.85rem; }
  .score { font-size: 0.85rem; color: #6cf; margin-bottom: 0.5rem; }
</style>
