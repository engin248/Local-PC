<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TaskTabs from "../components/TaskTabs.svelte";
  import TaskDetail from "../components/TaskDetail.svelte";
  import PlanningStatus from "../components/PlanningStatus.svelte";
  import DecisionMap from "../components/DecisionMap.svelte";
  import AlternativePanel from "../components/AlternativePanel.svelte";
  import RiskPanel from "../components/RiskPanel.svelte";
  import ApprovalPanel from "../components/ApprovalPanel.svelte";
  import CheckpointPanel from "../components/CheckpointPanel.svelte";
  import TestPanel from "../components/TestPanel.svelte";
  import RollbackPanel from "../components/RollbackPanel.svelte";
  import LiveLog from "../components/LiveLog.svelte";
  import ReportPanel from "../components/ReportPanel.svelte";
  import DefinitiveAnswerPanel from "../components/DefinitiveAnswerPanel.svelte";
  import AIConnectionsPanel from "../components/AIConnectionsPanel.svelte";
  import SystemConnectionsPanel from "../components/SystemConnectionsPanel.svelte";

  let tasks = $state<any[]>([]);
  let selectedTaskId = $state<string | null>(null);
  let selectedTask = $derived(tasks.find(t => t.id === selectedTaskId) || null);

  let logs = $state<any[]>([]);
  let decisions = $state<any[]>([]);
  let alternatives = $state<any[]>([]);
  let approvals = $state<any[]>([]);
  let checkpoints = $state<any[]>([]);
  let tests = $state<any[]>([]);
  let reports = $state<any[]>([]);
  let aiProviderHealth = $state<any[]>([]);
  let systemConnectorHealth = $state<any[]>([]);

  let activeSection = $state("planning");
  let footerTab = $state("agent_stream"); // "planning", "decisions", "security", "connections", "execution"
  let globalError = $state<string | null>(null);
  let alarmEvents = $state<any[]>([]);
  let voiceRepliesEnabled = $state(true);
  let voiceAvailable = $state(true);
  let lastSpokenVoiceKey = "";
  let speechQueue = $state<{ text: string; key: string }[]>([]);
  let isSpeaking = $state(false);

  let audioCtx: AudioContext | null = null;
  let alarmInterval: any = null;

  function formatError(err: unknown) {
    if (err instanceof Error) return err.message;
    return String(err);
  }

  async function safeInvoke(cmd: string, args?: any): Promise<any> {
    if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
      return await invoke(cmd, args);
    }
    if (!import.meta.env.DEV) {
      throw new Error(`Tauri köprüsü yok: ${cmd} komutu production ortamında çalıştırılamaz.`);
    }
    await new Promise(resolve => setTimeout(resolve, 80));

    switch (cmd) {
      case "get_tasks_cmd":
        return [];
      case "get_task_logs_cmd":
        return [];
      case "get_decisions_cmd":
        return [];
      case "get_alternatives_cmd":
        return [];
      case "get_approvals_cmd":
        return [];
      case "get_checkpoints_cmd":
        return [];
      case "get_tests_cmd":
        return [];
      case "get_reports_cmd":
        return [];
      case "get_system_health_cmd":
        return [];
      case "get_ai_provider_health_cmd":
        return [];
      case "get_system_connector_health_cmd":
        return [];
      case "create_task_cmd":
        throw new Error("Tauri bağlantısı olmadan gerçek görev oluşturulamaz.");
      case "save_plan_cmd":
        return null;
      case "execute_task_cmd":
        throw new Error("Tauri bağlantısı olmadan gerçek görev yürütülemez.");
      case "submit_approval_cmd":
        return null;
      case "rollback_task_cmd":
        return true;
      default:
        throw new Error(`Bilinmeyen komut: ${cmd}`);
    }
  }

  function playSiren() {
    try {
      if (!audioCtx) {
        audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
      }
      if (audioCtx.state === 'suspended') {
        audioCtx.resume();
      }
      
      if (alarmInterval) {
        clearInterval(alarmInterval);
        alarmInterval = null;
      }
      
      // Çift bip uyarısı çal ve bitir (Sürekli kafa ütüleyen döngüyü engeller!)
      const playBeep = (delay: number, freq: number) => {
        setTimeout(() => {
          if (!audioCtx) return;
          try {
            const osc = audioCtx.createOscillator();
            const gain = audioCtx.createGain();
            osc.connect(gain);
            gain.connect(audioCtx.destination);
            osc.type = "sine";
            osc.frequency.setValueAtTime(freq, audioCtx.currentTime);
            gain.gain.setValueAtTime(0.04, audioCtx.currentTime);
            gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.15);
            osc.start();
            osc.stop(audioCtx.currentTime + 0.18);
          } catch (err) {
            console.error("Beep calinamadi:", err);
          }
        }, delay);
      };

      // Tatlı bir çift uyarı melodisi çal (Cevap sesini bastırmaz ve kafa karıştırmaz)
      playBeep(0, 880);
      playBeep(180, 1100);
    } catch (e) {
      console.error("Siren sesi calisma hatasi:", e);
    }
  }

  function stopSiren() {
    if (alarmInterval) {
      clearInterval(alarmInterval);
      alarmInterval = null;
    }
  }

  function raiseCriticalAlarm(source: string, err: unknown) {
    const message = `${source}: ${formatError(err)}`;
    globalError = message;
    alarmEvents = [
      {
        id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
        source,
        message,
        timestamp: new Date().toLocaleString("tr-TR")
      },
      ...alarmEvents
    ].slice(0, 8);

    playSiren();
    speakReply(`Acil sistem alarmı. ${message}`, `critical:${message}`, true);
  }

  function speakReply(text: string, key = text, force = true) {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      voiceAvailable = false;
      return;
    }

    if (!force && key === lastSpokenVoiceKey) return;
    if (!force && !voiceRepliesEnabled) return;

    lastSpokenVoiceKey = key;

    // Eğer otomatik (force = false) bir durum güncellemesi tetiklendiyse veya alarm ise,
    // kuyruktaki tüm eski bayat mesajları temizle ve çalan eski sesi iptal et.
    // Bu sayede aynı anda sadece TEYİT EDİLMİŞ TEK BİR SES çalacaktır.
    if (!force || key.startsWith("critical")) {
      speechQueue = [];
      isSpeaking = false;
      window.speechSynthesis.cancel();
    }

    // Mesajı ses kuyruğuna ekle (Mesaj kaybını önler)
    speechQueue.push({ text, key });

    // Eğer şu an herhangi bir ses çalmıyorsa kuyruk işlemcisini başlat
    if (!isSpeaking) {
      processSpeechQueue();
    }
  }

  function processSpeechQueue() {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) return;

    const synth = window.speechSynthesis;

    // Eğer kuyrukta çalacak ses kalmadıysa durumu sıfırla
    if (speechQueue.length === 0) {
      isSpeaking = false;
      return;
    }

    isSpeaking = true;
    const currentItem = speechQueue[0];

    const utterance = new SpeechSynthesisUtterance(currentItem.text);
    const voices = synth.getVoices();
    const turkishVoice = voices.find((voice) => voice.lang.toLowerCase().startsWith("tr"));

    utterance.lang = "tr-TR";
    utterance.rate = 0.95;
    utterance.pitch = 1;
    utterance.volume = 1;
    if (turkishVoice) {
      utterance.voice = turkishVoice;
    }

    // Ses başarıyla bittiğinde kuyruktan çıkar ve sıradakine geç
    utterance.onend = () => {
      speechQueue.shift();
      processSpeechQueue();
    };

    // Ses hatayla kesildiğinde veya çalmadığında da takılmaması için sıradakine geç
    utterance.onerror = (e) => {
      console.error("Speech Synthesis Error:", e);
      speechQueue.shift();
      processSpeechQueue();
    };

    synth.speak(utterance);
  }

  function stopVoiceReply() {
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      speechQueue = []; // Kuyruğu temizle
      isSpeaking = false;
      window.speechSynthesis.cancel(); // Mevcut çalmayı durdur
    }
  }

  function toggleVoiceReplies() {
    voiceRepliesEnabled = !voiceRepliesEnabled;
    localStorage.setItem("voiceRepliesEnabled", String(voiceRepliesEnabled));

    if (voiceRepliesEnabled) {
      speakReply("Sesli cevap açıldı.", "voice-enabled", true);
    } else {
      stopVoiceReply();
    }
  }

  $effect(() => {
    if (globalError) {
      playSiren();
    } else {
      stopSiren();
    }
  });

  async function loadTasks() {
    try {
      tasks = await safeInvoke("get_tasks_cmd");
      if (tasks.length > 0 && !selectedTaskId) {
        selectedTaskId = tasks[0].id;
      }
      if (selectedTaskId) {
        await refreshTaskDetails(selectedTaskId);
      }
    } catch (err) {
      console.error("Yükleme hatası:", err);
      raiseCriticalAlarm("Görevler yüklenirken hata oluştu", err);
    }
  }

  async function checkSystemHealth() {
    try {
      const issues: any[] = await safeInvoke("get_system_health_cmd");
      const blockers = issues.filter((issue) => issue.severity === "error");
      if (blockers.length > 0) {
        raiseCriticalAlarm(
          "Sistem kök doğrulaması başarısız",
          blockers.map((issue) => `${issue.code}: ${issue.message}`).join(" | ")
        );
      }
    } catch (err) {
      raiseCriticalAlarm("Sistem kök doğrulaması çalıştırılamadı", err);
    }
  }

  async function refreshConnectionHealth(writeAudit = false) {
    try {
      aiProviderHealth = await safeInvoke("get_ai_provider_health_cmd", { writeAudit });
      systemConnectorHealth = await safeInvoke("get_system_connector_health_cmd", { writeAudit });
    } catch (err) {
      console.error("Bağlantı health-check hatası:", err);
      raiseCriticalAlarm("Bağlantı health-check sırasında hata oluştu", err);
    }
  }

  async function refreshTaskDetails(taskId: string) {
    try {
      logs = await safeInvoke("get_task_logs_cmd", { taskId });
      decisions = await safeInvoke("get_decisions_cmd", { taskId });
      alternatives = await safeInvoke("get_alternatives_cmd", { taskId });
      approvals = await safeInvoke("get_approvals_cmd", { taskId });
      checkpoints = await safeInvoke("get_checkpoints_cmd", { taskId });
      tests = await safeInvoke("get_tests_cmd", { taskId });
      reports = await safeInvoke("get_reports_cmd", { taskId });
    } catch (err) {
      console.error("Detay yükleme hatası:", err);
      raiseCriticalAlarm("Görev detayları yüklenirken hata oluştu", err);
    }
  }

  async function handleSelectTask(id: string) {
    selectedTaskId = id;
    await refreshTaskDetails(id);
  }

  async function handleCreateTask(title: string, userRequest: string) {
    try {
      const newTask: any = await safeInvoke("create_task_cmd", { title, userRequest });
      selectedTaskId = newTask.id;
      await loadTasks();
      speakReply("Görev kaydedildi. Kesin cevap için planlama ve güvenlik kapıları bekleniyor.", `task-created:${newTask.id}`, true);
    } catch (err) {
      console.error("Görev oluşturulamadı:", err);
      raiseCriticalAlarm("Görev oluşturulamadı", err);
    }
  }

  async function handleSavePlan(planInput: any) {
    if (!selectedTaskId) return;
    try {
      await safeInvoke("save_plan_cmd", { taskId: selectedTaskId, plan: planInput });
      await loadTasks();
      speakReply("Plan kaydedildi. Planlama alanları doğrulandı.", `plan-saved:${selectedTaskId}`, true);
      alert("Plan kaydedildi, 17/17 alan doğrulandı.");
    } catch (err) {
      console.error("Plan kaydedilemedi:", err);
      raiseCriticalAlarm("Plan kaydedilemedi", err);
    }
  }

  async function handleExecute() {
    if (!selectedTaskId) return;
    try {
      const res: any = await safeInvoke("execute_task_cmd", { taskId: selectedTaskId });
      await loadTasks();
      speakReply(res.message || "Yürütme tamamlandı.", `execution:${selectedTaskId}:${res.message || ""}`, true);
      alert(res.message);
    } catch (err) {
      console.error("Yürütme hatası:", err);
      raiseCriticalAlarm("Yürütme sırasında hata oluştu", err);
      await loadTasks();
    }
  }

  async function handleApproval(approvalId: string, approve: boolean, userNote: string, approverId: string, approverRole: string) {
    try {
      await safeInvoke("submit_approval_cmd", { approvalId, approve, userNote, approverId, approverRole });
      await loadTasks();
      speakReply(approve ? "İşlem onaylandı." : "İşlem reddedildi.", `approval:${approvalId}:${approve}`, true);
      alert(approve ? "İşlem onaylandı." : "İşlem reddedildi.");
    } catch (err) {
      console.error("Onay gönderme hatası:", err);
      raiseCriticalAlarm("Onay işlemi sırasında hata oluştu", err);
    }
  }

  async function handleRollback() {
    if (!selectedTaskId) return;
    try {
      const success: boolean = await safeInvoke("rollback_task_cmd", { taskId: selectedTaskId });
      await loadTasks();
      speakReply(success ? "Rollback başarıyla tamamlandı." : "Geri alınacak bir snapshot bulunamadı.", `rollback:${selectedTaskId}:${success}`, true);
      alert(success ? "Rollback başarıyla tamamlandı!" : "Geri alınacak bir snapshot bulunamadı.");
    } catch (err) {
      console.error("Rollback hatası:", err);
      raiseCriticalAlarm("Rollback işlemi sırasında hata oluştu", err);
    }
  }

  // 3 saniyede bir log ve durum güncellemesi yapalım (canlı izleme)
  onMount(() => {
    const savedVoiceSetting = localStorage.getItem("voiceRepliesEnabled");
    if (savedVoiceSetting !== null) {
      voiceRepliesEnabled = savedVoiceSetting === "true";
    }

    if (import.meta.env.DEV && new URLSearchParams(window.location.search).has("alarmTest")) {
      raiseCriticalAlarm(
        "Otomatik görsel alarm testi",
        "Test amaçlı hata enjeksiyonu: alarm banner, aktif hata kayıtları ve sesli kritik alarm akışı doğrulanıyor."
      );
    }

    checkSystemHealth();
    refreshConnectionHealth(true);
    loadTasks();
    const interval = setInterval(() => {
      if (selectedTaskId) {
        loadTasks();
      }
    }, 3000);
    return () => clearInterval(interval);
  });
</script>

<div class="app-layout">
  {#if globalError}
    <div class="alarm-flashing-overlay"></div>
  {/if}
  <div class="sidebar">
    <div class="logo-area">
      <img src="/tauri.svg" alt="Tauri Logo" class="brand-logo" />
      <div class="brand-text">
        <h1>LOKAL BİLGİSAYAR</h1>
        <span>KONTROL PANELİ</span>
      </div>
    </div>
    <TaskTabs 
      tasks={tasks} 
      selectedTaskId={selectedTaskId} 
      onSelect={handleSelectTask} 
      onCreate={handleCreateTask} 
    />
  </div>

  <div class="main-workspace">
        <div class="agent-status-bar">
      <strong>AJAN DURUMLARI:</strong>
      {#each aiProviderHealth as agent}
        <span class="agent-badge" class:agent-ok={agent.enabled} class:agent-disabled={!agent.enabled}>
          {agent.name.split(' ')[0]}
          {#if agent.enabled}
            <span class="status-dot green"></span>
          {:else}
            <span class="status-dot red"></span>
          {/if}
        </span>
      {/each}
    </div>
      <div class="workspace-header">
      <div class="navigation-tabs">
        <button class="nav-btn" class:active={activeSection === 'planning'} onclick={() => activeSection = 'planning'}>PLANLAMA (GATE 1)</button>
        <button class="nav-btn" class:active={activeSection === 'decisions'} onclick={() => activeSection = 'decisions'}>KARAR AĞACI & ALTERNATİFLER (GATE 2-4)</button>
        <button class="nav-btn" class:active={activeSection === 'security'} onclick={() => activeSection = 'security'}>GÜVENLİK DUVARI & ONAY (GATE 5-7)</button>
        <button class="nav-btn" class:active={activeSection === 'connections'} onclick={() => activeSection = 'connections'}>BAĞLANTILAR</button>
        <button class="nav-btn" class:active={activeSection === 'execution'} onclick={() => activeSection = 'execution'}>TEST VE RAPOR (GATE 8)</button>
      </div>
      <div class="voice-controls">
        <button
          class="voice-btn"
          class:active={voiceRepliesEnabled}
          disabled={!voiceAvailable}
          onclick={toggleVoiceReplies}
        >
          {voiceRepliesEnabled ? "Sesli Cevap Açık" : "Sesli Cevap Kapalı"}
        </button>
        <button class="voice-btn stop" disabled={!voiceAvailable} onclick={stopVoiceReply}>Cevap Sesini Durdur</button>
      </div>
    </div>

    {#if globalError}
      <div class="global-error-banner">
        <span class="error-icon">ALARM</span>
        <div class="error-message">
          <strong>SİSTEM HATASI TESPİT EDİLDİ</strong>
          <span>{globalError}</span>
          <small>Alarm sesi hata görünür olduğu sürece kapatılmaz. Normal sesli cevap durdurma düğmesi bu alarmı susturmaz.</small>
        </div>
      </div>

      <div class="alarm-history-panel">
        <div class="alarm-history-title">AKTİF HATA KAYITLARI</div>
        {#each alarmEvents as alarm}
          <div class="alarm-history-item">
            <span>{alarm.timestamp}</span>
            <strong>{alarm.source}</strong>
            <p>{alarm.message}</p>
          </div>
        {/each}
      </div>
    {/if}

    <div class="workspace-scroll-area">
      <TaskDetail task={selectedTask} onExecute={handleExecute} />
      <DefinitiveAnswerPanel
        task={selectedTask}
        approvals={approvals}
        tests={tests}
        reports={reports}
        voiceRepliesEnabled={voiceRepliesEnabled}
        onSpeakAnswer={speakReply}
        onStopVoice={stopVoiceReply}
      />

      {#if activeSection === 'connections'}
        <AIConnectionsPanel providers={aiProviderHealth} onRefresh={() => refreshConnectionHealth(true)} />
        <SystemConnectionsPanel connectors={systemConnectorHealth} onRefresh={() => refreshConnectionHealth(true)} />
      {:else if selectedTask}
        {#if activeSection === 'planning'}
          <PlanningStatus task={selectedTask} onSavePlan={handleSavePlan} />
        {:else if activeSection === 'decisions'}
          <DecisionMap decisions={decisions} />
          <AlternativePanel alternatives={alternatives} />
        {:else if activeSection === 'security'}
          <RiskPanel task={selectedTask} />
          <ApprovalPanel approvals={approvals} onSubmitApproval={handleApproval} />
          <RollbackPanel task={selectedTask} onRollback={handleRollback} />
        {:else if activeSection === 'execution'}
          <CheckpointPanel checkpoints={checkpoints} />
          <TestPanel tests={tests} />
          <ReportPanel reports={reports} />
        {/if}
      {/if}
    </div>

        <div class="workspace-footer">
      <div class="footer-tabs">
        <button class="footer-tab-btn" class:active={footerTab === 'agent_stream'} onclick={() => footerTab = 'agent_stream'}>AJAN İLETİŞİM & RAPORLAR</button>
        <button class="footer-tab-btn" class:active={footerTab === 'system_logs'} onclick={() => footerTab = 'system_logs'}>SİSTEM & AUDIT LOGLARI</button>
      </div>
      <div class="footer-content">
        {#if footerTab === 'system_logs'}
          <LiveLog logs={logs} />
        {:else}
          <div class="agent-stream-panel">
             <!-- Ajan raporları ve niyetlerini göstereceğimiz yalıtılmış alan -->
             <div class="stream-header">
               <h4>Operasyonel Ajan Akışı</h4>
               <p>Ajanların aldıkları kararlar ve ürettikleri raporlar teknik loglardan bağımsız olarak burada listelenir.</p>
             </div>
             <div class="stream-body">
               {#if reports.length > 0}
                 {#each reports as rep}
                   <div class="agent-msg report-msg">
                     <strong>[Rapor: {rep.report_type.toUpperCase()}]</strong>
                     <pre>{rep.content}</pre>
                   </div>
                 {/each}
               {:else}
                 <div class="empty-stream">Henüz bir ajan raporu veya kararı bulunmuyor.</div>
               {/if}
             </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: 'Outfit', 'Inter', sans-serif;
    background-color: #0c0c0d;
    overflow: hidden;
  }

  .app-layout {
    display: flex;
    width: 100vw;
    height: 100vh;
    background: #0c0c0d;
  }

  .sidebar {
    width: 320px;
    display: flex;
    flex-direction: column;
    background: #111112;
    border-right: 1px solid #1f1f21;
  }

  .logo-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 20px;
    background: #18181a;
    border-bottom: 1px solid #1f1f21;
  }

  .brand-logo {
    height: 32px;
    filter: drop-shadow(0 0 8px #007acc);
  }

  .brand-text h1 {
    font-size: 0.95rem;
    margin: 0;
    letter-spacing: 2px;
    color: #fff;
    font-weight: 800;
  }

  .brand-text span {
    font-size: 0.55rem;
    letter-spacing: 1px;
    color: #007acc;
    font-weight: 600;
  }

  .main-workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #141416;
  }

  .workspace-header {
    background: #18181a;
    border-bottom: 1px solid #1f1f21;
    padding: 0 15px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 48px;
  }

  .navigation-tabs {
    display: flex;
    gap: 5px;
    min-width: 0;
  }

  .nav-btn {
    background: transparent;
    border: none;
    color: #888;
    padding: 8px 16px;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.8px;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .nav-btn:hover {
    color: #fff;
    background: #252528;
  }

  .nav-btn.active {
    color: #fff;
    background: #007acc;
  }

  .voice-controls {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .voice-btn {
    border: 1px solid #3b3b40;
    background: #222226;
    color: #c8c8cc;
    padding: 7px 10px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 800;
    cursor: pointer;
  }

  .voice-btn.active {
    background: #0e639c;
    border-color: #1177bb;
    color: #fff;
  }

  .voice-btn.stop {
    background: #2a2020;
    border-color: #4a2b2b;
  }

  .voice-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .workspace-scroll-area {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  

  .global-error-banner {
    display: flex;
    align-items: center;
    gap: 15px;
    background: rgba(244, 71, 71, 0.22);
    border: 2px solid rgba(244, 71, 71, 0.9);
    border-left-width: 8px;
    border-radius: 6px;
    padding: 14px 20px;
    margin: 15px;
    color: #ffd7d7;
    font-size: 0.9rem;
    backdrop-filter: blur(8px);
    box-shadow: 0 0 22px rgba(244, 71, 71, 0.35);
    animation: slideDown 0.3s ease-out, alarmPulse 0.8s infinite alternate;
  }

  .error-icon {
    background: #f44747;
    color: #fff;
    border-radius: 4px;
    padding: 8px 10px;
    font-size: 0.75rem;
    font-weight: 900;
    letter-spacing: 1px;
  }

  .error-message {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .error-message strong {
    color: #fff;
    font-size: 0.92rem;
    letter-spacing: 0.8px;
  }

  .error-message span {
    color: #ffd7d7;
    line-height: 1.35;
  }

  .error-message small {
    color: #ffb3b3;
    font-size: 0.73rem;
    font-weight: 700;
  }

  .alarm-history-panel {
    margin: 0 15px 15px 15px;
    background: #191112;
    border: 1px solid rgba(244, 71, 71, 0.45);
    border-radius: 4px;
    color: #f1d0d0;
    max-height: 160px;
    overflow-y: auto;
  }

  .alarm-history-title {
    padding: 8px 12px;
    border-bottom: 1px solid rgba(244, 71, 71, 0.25);
    color: #ff8a8a;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.8px;
  }

  .alarm-history-item {
    display: grid;
    grid-template-columns: 140px 180px 1fr;
    gap: 10px;
    align-items: start;
    padding: 8px 12px;
    border-bottom: 1px solid rgba(244, 71, 71, 0.14);
    font-size: 0.75rem;
  }

  .alarm-history-item span {
    color: #b98b8b;
    font-family: monospace;
  }

  .alarm-history-item strong {
    color: #fff;
  }

  .alarm-history-item p {
    margin: 0;
    color: #f3c2c2;
    overflow-wrap: anywhere;
  }

  @keyframes slideDown {
    from {
      transform: translateY(-10px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  @keyframes alarmPulse {
    0% {
      border-color: rgba(244, 71, 71, 0.55);
      box-shadow: 0 0 12px rgba(244, 71, 71, 0.2);
    }
    100% {
      border-color: rgba(255, 70, 70, 1);
      box-shadow: 0 0 28px rgba(244, 71, 71, 0.55);
    }
  }

  .alarm-flashing-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    border: 8px solid rgba(255, 71, 71, 0.6);
    box-sizing: border-box;
    pointer-events: none;
    z-index: 9999;
    animation: pulseBorder 1s infinite alternate;
  }

  @keyframes pulseBorder {
    0% {
      border-color: rgba(255, 71, 71, 0.2);
      box-shadow: inset 0 0 15px rgba(255, 71, 71, 0.1);
    }
    100% {
      border-color: rgba(255, 71, 71, 0.9);
      box-shadow: inset 0 0 40px rgba(255, 71, 71, 0.5);
    }
  }
  .agent-status-bar {
    display: flex;
    gap: 12px;
    align-items: center;
    background: #111113;
    padding: 8px 16px;
    border-bottom: 1px solid #2d2d31;
    font-size: 12px;
    color: #8d8d95;
    overflow-x: auto;
  }
  .agent-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: #18181a;
    border: 1px solid #2d2d31;
    border-radius: 4px;
    color: #f4f4f5;
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .status-dot.green { background: #47d18c; box-shadow: 0 0 5px #47d18c; }
  .status-dot.red { background: #e03131; }
  .workspace-footer {
    height: 250px;
    border-top: 1px solid #1f1f21;
    background: #111112;
    display: flex;
    flex-direction: column;
  }
  .footer-tabs {
    display: flex;
    background: #0c0c0d;
    border-bottom: 1px solid #1f1f21;
  }
  .footer-tab-btn {
    background: none;
    border: none;
    color: #8d8d95;
    padding: 8px 16px;
    font-size: 11px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }
  .footer-tab-btn.active {
    color: #f4f4f5;
    border-bottom-color: #0b74de;
    background: #111112;
  }
  .footer-content {
    flex: 1;
    overflow: hidden;
  }
  .agent-stream-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 12px;
  }
  .stream-header {
    border-bottom: 1px solid #2d2d31;
    padding-bottom: 8px;
    margin-bottom: 12px;
  }
  .stream-header h4 { margin: 0; color: #f2f2f4; font-size: 14px; }
  .stream-header p { margin: 4px 0 0; color: #8d8d95; font-size: 12px; }
  .stream-body {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .agent-msg {
    background: #18181a;
    border: 1px solid #2d2d31;
    border-left: 3px solid #0b74de;
    padding: 10px;
    border-radius: 4px;
  }
  .agent-msg strong { display: block; margin-bottom: 6px; color: #47d18c; font-size: 12px; }
  .agent-msg pre { margin: 0; font-family: monospace; font-size: 12px; color: #b8b8bf; white-space: pre-wrap; }
  .empty-stream { color: #8d8d95; font-size: 13px; font-style: italic; }
  .progress-bar-container { display: flex; align-items: center; justify-content: center; padding: 12px 20px; background: #0c0c0d; border-bottom: 1px solid #1f1f21; }
  .progress-step { font-size: 11px; font-weight: bold; color: #555; padding: 4px 10px; border-radius: 12px; background: #18181a; border: 1px solid #2d2d31; }
  .progress-step.active { color: #fff; background: #0b74de; border-color: #0b74de; }
  .progress-step.done { color: #47d18c; border-color: #47d18c; }
  .progress-line { flex: 1; height: 2px; background: #2d2d31; margin: 0 10px; max-width: 50px; }
</style>
