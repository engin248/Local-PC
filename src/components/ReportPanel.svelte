<script lang="ts">
  let { reports = [] } = $props<{
    reports: any[];
  }>();

  const sections = [
    { key: "analysis", title: "Çözümleme", marker: "## A. Çözümleme Raporu" },
    { key: "plan", title: "Uygulama Planı", marker: "## B. Uygulama Planı" },
    { key: "monitoring", title: "Uygulama İzleme", marker: "## C. Uygulama İzleme Raporu" }
  ];

  function hasSection(content: string, marker: string) {
    return content.includes(marker);
  }

  function sectionContent(content: string, marker: string) {
    const start = content.indexOf(marker);
    if (start < 0) return "";
    const next = sections
      .map((section) => content.indexOf(section.marker, start + marker.length))
      .filter((index) => index > start)
      .sort((a, b) => a - b)[0];
    return content.slice(start, next || content.length).trim();
  }
</script>

<div class="report-panel-container">
  <h3>İŞLEM VE DENETİM RAPORU</h3>

  {#if reports.length > 0}
    <div class="reports-list">
      {#each reports as rep}
        <div class="report-card">
          <div class="report-header">
            <span>Rapor Tipi: {rep.report_type.toUpperCase()}</span>
          </div>
          <div class="report-body">
            <div class="structured-report">
              {#each sections as section}
                <section class:missing={!hasSection(rep.content, section.marker)}>
                  <h4>{section.title}</h4>
                  {#if hasSection(rep.content, section.marker)}
                    <pre>{sectionContent(rep.content, section.marker)}</pre>
                  {:else}
                    <p class="missing-text">Eksik rapor bölümü: {section.title}</p>
                  {/if}
                </section>
              {/each}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Nihai rapor henüz oluşturulmadı. 8 Kapı başarıyla tamamlandığında rapor üretilecektir.</p>
  {/if}
</div>

<style>
  .report-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .report-card {
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 12px;
  }
  .report-header {
    border-bottom: 1px solid #333;
    padding-bottom: 5px;
    margin-bottom: 8px;
    font-size: 0.7rem;
    color: #888;
    font-weight: bold;
  }
  .report-body pre {
    margin: 0;
    white-space: pre-wrap;
    font-family: monospace;
    font-size: 0.75rem;
    color: #dcdcaa;
    line-height: 1.5;
  }
  .structured-report {
    display: grid;
    gap: 10px;
  }
  .structured-report section {
    border: 1px solid #333;
    border-radius: 4px;
    padding: 10px;
    background: #1e1e1e;
  }
  .structured-report section.missing {
    border-color: #f44747;
    background: #2a1818;
  }
  h4 {
    margin: 0 0 8px 0;
    font-size: 0.75rem;
    color: #9cdcfe;
    text-transform: uppercase;
  }
  .missing-text {
    margin: 0;
    color: #f44747;
    font-size: 0.8rem;
    font-weight: bold;
  }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
