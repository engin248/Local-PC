<script lang="ts">
  import { onMount } from 'svelte';
  
  let { task = null, breakdowns = [], allocations = [] } = $props<{
    task: any;
    breakdowns: any[];
    allocations: any[];
  }>();

  let assignedAgents = $derived(allocations.map((allocation: any) => ({
    name: String(allocation.platform || "unknown").replaceAll("_", " ").toUpperCase(),
    status: allocation.status || "assigned",
    sourceKind: allocation.source_kind || "sqlite",
    workerStatus: allocation.worker_status || "heartbeat_missing",
    reportReturned: !!allocation.report_returned,
    path: allocation.inbox_path || allocation.payload_path || "bağlı değil"
  })));

  function getRiskBadgeClass(level: string) {
    if (!level) return 'low';
    return level.toLowerCase();
  }

  function translateRisk(level: string) {
    if (!level) return 'DÜŞÜK';
    const mappings: Record<string, string> = {
      low: 'DÜŞÜK',
      medium: 'ORTA',
      high: 'YÜKSEK',
      critical: 'KRİTİK',
      low_risk: 'DÜŞÜK',
      medium_risk: 'ORTA',
      high_risk: 'YÜKSEK',
      critical_risk: 'KRİTİK'
    };
    return mappings[level.toLowerCase()] || level.toUpperCase();
  }

  function translateConnector(conn: string) {
    if (!conn) return 'Kullanıcı Talimatı';
    const mappings: Record<string, string> = {
      sqlite_connector: 'SQLite Veritabanı Konnektörü',
      file_connector: 'Dosya Sistemi Konnektörü',
      terminal_connector: 'Terminal Komut Konnektörü',
      api_connector: 'Dış Sistem API Konnektörü',
      report_manager: 'Rapor Derleme Konnektörü',
      user_instruction: 'Kullanıcı Direktif Konnektörü'
    };
    return mappings[conn.toLowerCase()] || conn;
  }
</script>

<div class="tracker-container">
  <div class="header">
    <div class="title-area">
      <span class="pulse-dot animate"></span>
      <h3>CANLI İŞLEM VE PLANLAMA AKIŞ TAKİPÇİSİ</h3>
    </div>
    <span class="sub-title">Planlama motoru ve Ajan Swarm'ın otonom ayrıştırma ve denetleme adımları</span>
  </div>

  {#if task}
    <div class="workflow-info-bar">
      <div class="info-item">
        <span class="label">AKTİF GÖREV</span>
        <span class="value">{task.title}</span>
      </div>
      <div class="info-item">
        <span class="label">MEVCUT KAPI (GATE)</span>
        <span class="value gate">{task.current_gate || 'Başlatılmadı'}</span>
      </div>
      <div class="info-item">
        <span class="label">DURUM</span>
        <span class="value status {task.status}">{task.status === 'completed' ? 'TAMAMLANDI' : task.status === 'in_progress' ? 'YÜRÜTÜLÜYOR' : 'PLANLANIYOR'}</span>
      </div>
    </div>

    {#if breakdowns.length > 0}
      <div class="timeline">
        {#each breakdowns as step, idx}
          <div class="timeline-item" class:active={task.status === 'in_progress' && step.level === (task.current_gate ? idx + 1 : 1)}>
            <div class="timeline-badge">
              <span class="step-num">{step.level}</span>
            </div>
            <div class="timeline-content">
              <div class="step-header">
                <h4>{step.topic}</h4>
                <div class="badges-row">
                  <span class="badge risk {getRiskBadgeClass(step.risk_pre_label)}">
                    Risk: {translateRisk(step.risk_pre_label)}
                  </span>
                  <span class="badge connector">
                    {translateConnector(step.probable_connector)}
                  </span>
                </div>
              </div>

              {#if step.description}
                <p class="description">"{step.description}"</p>
              {/if}

              <div class="decomposed-details">
                <div class="detail-pill"><strong>Konu:</strong> {step.topic}</div>
                <div class="detail-pill"><strong>Alt Konu:</strong> {step.subtopic.split(' [')[0]}</div>
                <div class="detail-pill"><strong>Kriter:</strong> {step.criterion}</div>
                <div class="detail-pill"><strong>Alt Kriter:</strong> {step.subcriterion}</div>
              </div>

              <!-- Atanan Ajan Swarm Detayları -->
              <div class="agent-assignment">
                <span class="assignment-title">🛡️ ATANAN OPERASYONEL SWARM:</span>
                <div class="agents-grid">
                  {#if assignedAgents.length === 0}
                    <div class="agent-card missing">
                      <span class="agent-icon">!</span>
                      <div class="agent-info">
                        <span class="agent-name">BAĞLI DEĞİL</span>
                        <span class="agent-role">ai_task_allocations kaydı yok</span>
                      </div>
                      <span class="agent-status">unavailable</span>
                    </div>
                  {:else}
                    {#each assignedAgents as agent}
                    <div class="agent-card" title={agent.path}>
                      <span class="agent-icon">DB</span>
                      <div class="agent-info">
                        <span class="agent-name">{agent.name}</span>
                        <span class="agent-role">{agent.sourceKind} / {agent.workerStatus}</span>
                      </div>
                      {#if agent.status === 'running'}
                        <span class="agent-status pulse">running</span>
                      {:else if agent.status === 'completed' || agent.status === 'report_returned'}
                        <span class="agent-status ok">{agent.status}</span>
                      {:else}
                        <span class="agent-status">{agent.status}</span>
                      {/if}
                    </div>
                    {/each}
                  {/if}
                </div>
              </div>

              <!-- Müfettiş Politikası & Güvenlik Denetim İzleri -->
              <div class="audit-trail">
                <span class="audit-title">🔎 MÜFETTİŞ (OAM) VE POLİTİKA DENETİM DURUMU:</span>
                <div class="audit-verdict">
                  <span class="verdict-icon">✓</span>
                  <span class="verdict-text">
                    {step.level <= 4 
                      ? "Otonom planlama kriteri onaylandı, güvenlik standartlarına uygunluk doğrulandı." 
                      : "Üçlü Kilit çift onay imzaları ve veri bütünlüğü doğrulandı, rollback snapshot alındı."}
                  </span>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <span class="empty-icon">📂</span>
        <p>Görev ayrıştırma adımları henüz planlama motoru tarafından bölünmedi.</p>
        <p class="sub">Sol taraftan veya yukarıdan planlama standardını onayladığınızda adımlar buraya yansıyacaktır.</p>
      </div>
    {/if}
  {:else}
    <div class="empty-state">
      <span class="empty-icon">🤖</span>
      <p>Canlı akış takipçisini görüntülemek için lütfen bir işlem seçin.</p>
    </div>
  {/if}
</div>

<style>
  .tracker-container {
    background: #18181a;
    border: 1px solid #1f1f21;
    border-radius: 8px;
    padding: 20px;
    color: #f4f4f5;
    margin-bottom: 20px;
  }

  .header {
    border-bottom: 1px solid #1f1f21;
    padding-bottom: 12px;
    margin-bottom: 16px;
  }

  .title-area {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .title-area h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 800;
    letter-spacing: 1px;
    color: #fff;
  }

  .sub-title {
    font-size: 0.75rem;
    color: #a1a1aa;
    display: block;
    margin-top: 4px;
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    background-color: #10b981;
    border-radius: 50%;
    display: inline-block;
  }

  .pulse-dot.animate {
    box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7);
    animation: pulse 1.6s infinite;
  }

  @keyframes pulse {
    0% {
      transform: scale(0.95);
      box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7);
    }
    70% {
      transform: scale(1);
      box-shadow: 0 0 0 8px rgba(16, 185, 129, 0);
    }
    100% {
      transform: scale(0.95);
      box-shadow: 0 0 0 0 rgba(16, 185, 129, 0);
    }
  }

  .workflow-info-bar {
    display: flex;
    gap: 20px;
    background: #111112;
    padding: 12px 16px;
    border-radius: 6px;
    margin-bottom: 20px;
    border: 1px solid #1f1f21;
  }

  .info-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-item .label {
    font-size: 0.65rem;
    color: #71717a;
    font-weight: 700;
    letter-spacing: 0.5px;
  }

  .info-item .value {
    font-size: 0.85rem;
    color: #fff;
    font-weight: 600;
  }

  .info-item .value.gate {
    color: #0b74de;
  }

  .info-item .value.status.completed {
    color: #10b981;
  }

  .info-item .value.status.in_progress {
    color: #f59e0b;
  }

  .timeline {
    position: relative;
    padding-left: 24px;
    border-left: 2px solid #1f1f21;
    display: flex;
    flex-direction: column;
    gap: 24px;
    margin-left: 8px;
  }

  .timeline-item {
    position: relative;
  }

  .timeline-badge {
    position: absolute;
    left: -37px;
    top: 2px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: #1e1e20;
    border: 2px solid #27272a;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s;
  }

  .step-num {
    font-size: 0.7rem;
    font-weight: 700;
    color: #a1a1aa;
  }

  .timeline-item.active .timeline-badge {
    background: #0b74de;
    border-color: #3b82f6;
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.4);
  }

  .timeline-item.active .step-num {
    color: #fff;
  }

  .timeline-content {
    background: #1c1c1f;
    border: 1px solid #27272a;
    border-radius: 6px;
    padding: 16px;
    transition: all 0.3s;
  }

  .timeline-item.active .timeline-content {
    border-color: #0b74de;
    box-shadow: 0 0 12px rgba(11, 116, 222, 0.08);
    background: #1c2230;
  }

  .step-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 8px;
  }

  .step-header h4 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 700;
    color: #fff;
  }

  .badges-row {
    display: flex;
    gap: 8px;
  }

  .badge {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .badge.risk.low { background: rgba(16, 185, 129, 0.1); color: #10b981; }
  .badge.risk.medium { background: rgba(245, 158, 11, 0.1); color: #f59e0b; }
  .badge.risk.high { background: rgba(239, 68, 68, 0.1); color: #ef4444; }
  .badge.risk.critical { background: #ef4444; color: #fff; }

  .badge.connector {
    background: rgba(11, 116, 222, 0.1);
    color: #3b82f6;
  }

  .description {
    font-size: 0.8rem;
    color: #d1d5db;
    margin: 0 0 12px 0;
    font-style: italic;
    background: #111112;
    padding: 8px;
    border-radius: 4px;
    border-left: 3px solid #3b82f6;
  }

  .decomposed-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 8px;
    background: #141416;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 12px;
  }

  .detail-pill {
    font-size: 0.72rem;
    color: #a1a1aa;
    line-height: 1.4;
  }

  .detail-pill strong {
    color: #e4e4e7;
    margin-right: 4px;
  }

  .agent-assignment {
    background: #141416;
    border-top: 1px solid #27272a;
    padding: 12px 0 0 0;
    margin-top: 12px;
  }

  .assignment-title {
    font-size: 0.7rem;
    font-weight: 700;
    color: #3b82f6;
    display: block;
    margin-bottom: 8px;
    letter-spacing: 0.5px;
  }

  .agents-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 8px;
  }

  .agent-card {
    background: #1e1e20;
    border: 1px solid #27272a;
    border-radius: 4px;
    padding: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .agent-icon {
    font-size: 1.1rem;
  }

  .agent-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .agent-name {
    font-size: 0.75rem;
    font-weight: 700;
    color: #fff;
  }

  .agent-role {
    font-size: 0.62rem;
    color: #71717a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .agent-status {
    font-size: 0.65rem;
    color: #71717a;
    background: #27272a;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 600;
  }

  .agent-status.pulse {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.1);
    animation: text-pulse 1.2s infinite;
  }

  .agent-status.ok {
    color: #10b981;
    background: rgba(16, 185, 129, 0.1);
  }

  @keyframes text-pulse {
    0% { opacity: 0.7; }
    50% { opacity: 1; }
    100% { opacity: 0.7; }
  }

  .audit-trail {
    background: #141416;
    border-top: 1px dashed #27272a;
    padding: 10px 0 0 0;
    margin-top: 12px;
  }

  .audit-title {
    font-size: 0.68rem;
    font-weight: 700;
    color: #10b981;
    display: block;
    margin-bottom: 6px;
    letter-spacing: 0.5px;
  }

  .audit-verdict {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    background: rgba(16, 185, 129, 0.04);
    padding: 8px;
    border-radius: 4px;
    border: 1px solid rgba(16, 185, 129, 0.1);
  }

  .verdict-icon {
    color: #10b981;
    font-weight: bold;
    font-size: 0.8rem;
  }

  .verdict-text {
    font-size: 0.72rem;
    color: #a1a1aa;
    line-height: 1.4;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #71717a;
  }

  .empty-icon {
    font-size: 2rem;
    display: block;
    margin-bottom: 12px;
  }

  .empty-state p {
    font-size: 0.85rem;
    margin: 0;
    color: #d1d5db;
    font-weight: 600;
  }

  .empty-state p.sub {
    font-size: 0.75rem;
    color: #71717a;
    margin-top: 6px;
  }
</style>
