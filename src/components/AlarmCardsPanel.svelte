<script lang="ts">
  let {
    alarms = []
  }: {
    alarms: any[];
  } = $props();
</script>

<section class="connection-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">ALARM KAYNAKLARI</span>
      <h3>Runtime / JSON / SQLite Alarm Durumu</h3>
    </div>
  </div>

  <div class="connection-grid">
    {#each alarms as alarm}
      <article class="connection-row">
        <div>
          <strong>{alarm.title}</strong>
          <span class="source" class:preview={alarm.preview || alarm.source_kind === "mock"}>
            {alarm.preview || alarm.source_kind === "mock" ? "PREVIEW / MOCK" : alarm.source_kind}
          </span>
        </div>
        <div>
          <span class="label">Health</span>
          <b class:ok={alarm.health === "available" || alarm.health === "available_empty"} class:warn={alarm.health !== "available" && alarm.health !== "available_empty"}>
            {alarm.health}
          </b>
        </div>
        <div>
          <span class="label">Kalıcılık</span>
          <b>{alarm.runtime_only ? "runtime only" : "kalıcı/kaynaklı"}</b>
        </div>
        <div>
          <span class="label">Son kontrol</span>
          <b>{alarm.last_checked || "yok"}</b>
        </div>
        <div class="wide">
          <span class="label">Kaynak</span>
          <code>{alarm.source_path || "bağlı değil"}</code>
        </div>
        <p>{alarm.error || alarm.details || "Son hata yok."}</p>
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

  .connection-grid {
    display: grid;
    gap: 10px;
  }

  .connection-row {
    display: grid;
    grid-template-columns: minmax(180px, 1.4fr) repeat(3, minmax(90px, 0.7fr));
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

  .source {
    display: inline-block;
    width: fit-content;
    margin-top: 4px;
    padding: 2px 6px;
    border: 1px solid #3b3b40;
    border-radius: 999px;
    color: #9fd3ff;
    font-size: 10px;
    font-weight: 800;
  }

  .source.preview {
    color: #f8c14a;
    border-color: rgba(248, 193, 74, 0.5);
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }
</style>
