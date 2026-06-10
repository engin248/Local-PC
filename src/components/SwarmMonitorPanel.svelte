<script lang="ts">
  type AgentDefinition = {
    platform: string;
    name: string;
    role: string;
    lane: string;
    description: string;
  };

  type Allocation = {
    platform: string;
    payload_path: string;
    status: string;
  };

  let { allocations = [], taskId = null, task = null } = $props<{
    allocations: Allocation[];
    taskId: string | null;
    task?: any;
  }>();

  const agents: AgentDefinition[] = [
    {
      platform: "cursor",
      name: "Cursor",
      role: "Çalışma ortamı ve dosya düzenleme",
      lane: "Workspace",
      description: "Agentler Cursor içinde çalışır; panel görev, inbox ve durum görünürlüğünü toplar."
    },
    {
      platform: "codex",
      name: "Codex",
      role: "Kod analizi ve uygulama",
      lane: "Execution",
      description: "Kod değişikliği, terminal doğrulaması ve uygulama adımlarını üstlenir."
    },
    {
      platform: "open_agent_manager",
      name: "OAM",
      role: "Denetim ve orkestrasyon",
      lane: "Control",
      description: "Ajan tahsislerini, politika kapılarını ve rapor teslimlerini takip eder."
    },
    {
      platform: "antigravity",
      name: "AntiGrav",
      role: "Çapraz kontrol",
      lane: "Review",
      description: "Riskli değişikliklerde bağımsız kontrol ve güvenlik sinyali sağlar."
    },
    {
      platform: "perplexity",
      name: "Perplexity",
      role: "Araştırma",
      lane: "Research",
      description: "Dış kaynak araştırması ve kanıt toplama işleri için ayrılır."
    },
    {
      platform: "verdent",
      name: "Verdent",
      role: "Alternatif üretimi",
      lane: "Strategy",
      description: "Plan alternatifleri ve uygulanabilir yol seçeneklerini görünür kılar."
    }
  ];

  const statusLabels: Record<string, string> = {
    waiting: "Bekliyor",
    processing: "Çalışıyor",
    submitted: "Teslim edildi",
    failed: "Hata",
    rejected: "Reddedildi",
    not_assigned: "Atanmadı"
  };

  function normalizePlatform(platform: string) {
    return String(platform || "").toLowerCase();
  }

  function getAllocation(platform: string): Allocation | undefined {
    const normalizedPlatform = normalizePlatform(platform);
    return allocations.find((row: Allocation) => normalizePlatform(row.platform) === normalizedPlatform);
  }

  function assignedCount() {
    return allocations.length;
  }

  function statusLabel(status?: string) {
    return statusLabels[status || "not_assigned"] || status || "Bilinmiyor";
  }

  function statusClass(status?: string) {
    return `status-pill ${status || "not_assigned"}`;
  }
</script>

<section class="swarm-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">CURSOR AGENT PANELI</span>
      <h3>Agent Görünürlük ve Yönetim Panosu</h3>
      <p>Agentler Cursor'da çalışır; panel onları görev tahsisi, inbox dosyası ve durum etiketiyle yönetilebilir kılar.</p>
    </div>
    <div class="summary-grid">
      <div>
        <span>Çalışma Ortamı</span>
        <strong>Cursor</strong>
      </div>
      <div>
        <span>Atanan Agent</span>
        <strong>{assignedCount()}</strong>
      </div>
      <div>
        <span>Aktif Görev</span>
        <strong>{task?.title || "Seçilmedi"}</strong>
      </div>
    </div>
  </div>

  {#if !taskId}
    <div class="empty-state">
      <strong>Görev seçilmedi.</strong>
      <span>Bir görev oluşturduğunuzda seçilen agent'lar burada Cursor çalışma alanı altında görünür.</span>
    </div>
  {:else}
    <div class="agent-grid">
      {#each agents as agent}
        {@const allocation = getAllocation(agent.platform)}
        <article class="agent-card" class:assigned={!!allocation}>
          <div class="agent-card-header">
            <div>
              <span class="lane">{agent.lane}</span>
              <h4>{agent.name}</h4>
            </div>
            <span class={statusClass(allocation?.status)}>{statusLabel(allocation?.status)}</span>
          </div>
          <p>{agent.role}</p>
          <small>{agent.description}</small>

          {#if allocation}
            <div class="payload-box">
              <span>Cursor inbox payload</span>
              <code>{allocation.payload_path}</code>
            </div>
            <div class="control-row">
              <span>Görünürlük: aktif</span>
              <span>Rapor bekleme: açık</span>
            </div>
          {:else}
            <div class="not-assigned">
              Bu görev için seçilmedi; yeni görev intake ekranında işaretlenebilir.
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .swarm-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 8px;
    color: #f4f4f5;
    margin-bottom: 16px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    align-items: flex-start;
    margin-bottom: 18px;
    border-bottom: 1px solid #2d2d31;
    padding-bottom: 16px;
  }

  .eyebrow {
    color: #f59e0b;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.8px;
  }

  h3,
  h4,
  p {
    margin: 0;
  }

  h3 {
    margin-top: 4px;
    font-size: 19px;
  }

  .panel-header p {
    margin-top: 6px;
    color: #b8b8bf;
    font-size: 13px;
    max-width: 680px;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(110px, 1fr));
    gap: 8px;
    min-width: 420px;
  }

  .summary-grid div {
    background: #111113;
    border: 1px solid #2d2d31;
    border-radius: 6px;
    padding: 10px;
  }

  .summary-grid span,
  .lane {
    display: block;
    color: #8d8d95;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .summary-grid strong {
    display: block;
    margin-top: 4px;
    color: #f4f4f5;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 18px;
    border: 1px dashed #3b3b40;
    border-radius: 6px;
    color: #b8b8bf;
    background: #111113;
  }

  .empty-state strong {
    color: #f4f4f5;
  }

  .agent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(270px, 1fr));
    gap: 12px;
  }

  .agent-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #111113;
    border: 1px solid #2d2d31;
    border-radius: 8px;
    padding: 14px;
    min-height: 210px;
  }

  .agent-card.assigned {
    border-color: rgba(245, 158, 11, 0.62);
    box-shadow: 0 0 18px rgba(245, 158, 11, 0.08);
  }

  .agent-card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  h4 {
    margin-top: 3px;
    font-size: 16px;
  }

  .agent-card p {
    color: #f4f4f5;
    font-weight: 700;
    font-size: 13px;
  }

  .agent-card small {
    color: #a1a1aa;
    line-height: 1.4;
  }

  .status-pill {
    border-radius: 999px;
    padding: 5px 9px;
    font-size: 10px;
    font-weight: 800;
    background: #242428;
    color: #a1a1aa;
    white-space: nowrap;
  }

  .status-pill.waiting {
    background: rgba(245, 158, 11, 0.16);
    color: #f8c14a;
  }

  .status-pill.processing {
    background: rgba(11, 116, 222, 0.2);
    color: #8fdaff;
  }

  .status-pill.submitted {
    background: rgba(71, 209, 140, 0.18);
    color: #47d18c;
  }

  .status-pill.failed,
  .status-pill.rejected {
    background: rgba(224, 49, 49, 0.2);
    color: #ff9aa6;
  }

  .payload-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: auto;
    padding: 10px;
    background: #18181a;
    border: 1px solid #2d2d31;
    border-radius: 6px;
  }

  .payload-box span {
    color: #8fdaff;
    font-size: 11px;
    font-weight: 800;
  }

  code {
    color: #d6d6dd;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .control-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .control-row span {
    border: 1px solid #2d2d31;
    border-radius: 999px;
    padding: 5px 8px;
    color: #cfcfd6;
    font-size: 11px;
    background: #18181a;
  }

  .not-assigned {
    margin-top: auto;
    padding: 10px;
    border: 1px dashed #34343a;
    border-radius: 6px;
    color: #8d8d95;
    font-size: 12px;
    line-height: 1.4;
  }

  @media (max-width: 900px) {
    .panel-header {
      flex-direction: column;
    }

    .summary-grid {
      min-width: 0;
      width: 100%;
      grid-template-columns: 1fr;
    }
  }
</style>
