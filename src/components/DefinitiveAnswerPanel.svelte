<script lang="ts">
  let {
    task = null,
    approvals = [],
    tests = [],
    reports = [],
    voiceRepliesEnabled = false,
    onSpeakAnswer,
    onStopVoice
  } = $props<{
    task: any;
    approvals: any[];
    tests: any[];
    reports: any[];
    voiceRepliesEnabled?: boolean;
    onSpeakAnswer?: (text: string, key?: string, force?: boolean) => void;
    onStopVoice?: () => void;
  }>();

  let failedTests = $derived(tests.filter((test: any) => test.status === "failed"));
  let passedTests = $derived(tests.filter((test: any) => test.status === "passed"));
  let pendingApprovals = $derived(approvals.filter((approval: any) => approval.status === "pending"));
  let approvedApprovals = $derived(approvals.filter((approval: any) => approval.status === "approved"));
  let latestReport = $derived(reports.length > 0 ? reports[reports.length - 1] : null);

  let answerState = $derived.by(() => {
    if (!task) {
      return {
        tone: "neutral",
        label: "Kesin cevap yok",
        summary: "Bir görev seçilmedi.",
        action: "Sol panelden görev seçin veya yeni görev başlatın."
      };
    }

    if (task.status === "failed") {
      return {
        tone: "danger",
        label: "Hayır",
        summary: "İşlem tamamlanmadı veya güvenlik kapılarından biri geçilemedi.",
        action: "Hata kaydı, test sonucu ve rollback durumunu kontrol edin."
      };
    }

    if (failedTests.length > 0) {
      return {
        tone: "danger",
        label: "Hayır",
        summary: `${failedTests.length} test başarısız olduğu için işlem güvenli tamamlanmış kabul edilemez.`,
        action: "Test panelindeki başarısız kriterleri düzeltin."
      };
    }

    if (pendingApprovals.length > 0 || task.approval_status === "pending") {
      return {
        tone: "warning",
        label: "Onay bekleniyor",
        summary: "Kesin yürütme cevabı verilemez; yetkili kullanıcı onayı bekleniyor.",
        action: "Güvenlik Duvarı & Onay bölümünden geçerli rol ile onay verin veya reddedin."
      };
    }

    if (task.planning_status !== "planning_complete") {
      return {
        tone: "warning",
        label: "Plan eksik",
        summary: "Planlama standardı tamamlanmadığı için yürütme cevabı kesinleşmedi.",
        action: "Planlama bölümündeki zorunlu alanları tamamlayın."
      };
    }

    if (task.status === "completed") {
      return {
        tone: "success",
        label: "Evet",
        summary: "Görev mevcut kayıtlara göre tamamlandı.",
        action: latestReport ? "Nihai rapor oluşturuldu; rapor panelinden ayrıntıları inceleyin." : "Rapor kaydı yoksa rapor panelini yenileyin."
      };
    }

    return {
      tone: "neutral",
      label: "Henüz değil",
      summary: "Görev kayıtlı, ancak yürütme sonucu henüz kesinleşmedi.",
      action: "Plan tamamlandıysa Execution Engine'i başlatın."
    };
  });

  let voiceText = $derived(`${answerState.label}. ${answerState.summary} ${answerState.action}`);
  let voiceKey = $derived([
    task?.id || "no-task",
    task?.status || "no-status",
    task?.approval_status || "no-approval-status",
    answerState.label,
    failedTests.length,
    pendingApprovals.length,
    reports.length
  ].join(":"));

  $effect(() => {
    if (voiceRepliesEnabled && task && onSpeakAnswer) {
      onSpeakAnswer(voiceText, voiceKey, false);
    }
  });
</script>

<div class="definitive-answer {answerState.tone}">
  <div class="answer-main">
    <span class="eyebrow">KESİN CEVAP</span>
    <strong>{answerState.label}</strong>
    <p>{answerState.summary}</p>
    <div class="voice-actions">
      <button type="button" onclick={() => onSpeakAnswer?.(voiceText, voiceKey, true)}>Sesli Oku</button>
      <button type="button" onclick={() => onStopVoice?.()}>Sesi Durdur</button>
    </div>
  </div>

  <div class="answer-evidence">
    <div>
      <span>Durum</span>
      <strong>{task?.status || "yok"}</strong>
    </div>
    <div>
      <span>Onay</span>
      <strong>{approvedApprovals.length}/{approvals.length}</strong>
    </div>
    <div>
      <span>Test</span>
      <strong>{passedTests.length} geçti, {failedTests.length} kaldı</strong>
    </div>
    <div>
      <span>Rapor</span>
      <strong>{latestReport ? "var" : "yok"}</strong>
    </div>
  </div>

  <div class="next-action">{answerState.action}</div>
</div>

<style>
  .definitive-answer {
    display: grid;
    grid-template-columns: minmax(220px, 1fr) minmax(300px, 1.1fr);
    gap: 12px;
    align-items: stretch;
    margin: 12px 15px;
    padding: 14px;
    border: 1px solid #303033;
    border-left-width: 4px;
    border-radius: 6px;
    background: #202022;
    color: #ddd;
  }

  .definitive-answer.success {
    border-left-color: #4ec9b0;
  }

  .definitive-answer.warning {
    border-left-color: #dcdcaa;
  }

  .definitive-answer.danger {
    border-left-color: #f44747;
  }

  .definitive-answer.neutral {
    border-left-color: #007acc;
  }

  .answer-main {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }

  .eyebrow {
    font-size: 0.68rem;
    color: #8a8a8d;
    font-weight: 800;
    letter-spacing: 0.8px;
  }

  .answer-main strong {
    font-size: 1.2rem;
    color: #fff;
  }

  .answer-main p {
    margin: 0;
    font-size: 0.84rem;
    line-height: 1.4;
    color: #cfcfd2;
  }

  .voice-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .voice-actions button {
    border: 1px solid #3c3c3f;
    background: #18181a;
    color: #e6e6e8;
    border-radius: 4px;
    padding: 6px 9px;
    font-size: 0.72rem;
    font-weight: 700;
    cursor: pointer;
  }

  .voice-actions button:hover {
    background: #2a2a2d;
  }

  .answer-evidence {
    display: grid;
    grid-template-columns: repeat(4, minmax(70px, 1fr));
    gap: 8px;
  }

  .answer-evidence div {
    background: #18181a;
    border: 1px solid #303033;
    border-radius: 4px;
    padding: 8px;
    min-width: 0;
  }

  .answer-evidence span {
    display: block;
    color: #858589;
    font-size: 0.65rem;
    font-weight: 800;
    text-transform: uppercase;
    margin-bottom: 5px;
  }

  .answer-evidence strong {
    display: block;
    color: #f1f1f1;
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  .next-action {
    grid-column: 1 / -1;
    border-top: 1px solid #303033;
    padding-top: 10px;
    color: #aeb8c2;
    font-size: 0.78rem;
    line-height: 1.35;
  }

  @media (max-width: 900px) {
    .definitive-answer {
      grid-template-columns: 1fr;
    }

    .answer-evidence {
      grid-template-columns: repeat(2, minmax(120px, 1fr));
    }
  }
</style>
