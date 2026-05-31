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
  import StructuredReportPanel from "../components/StructuredReportPanel.svelte";
  import SwarmMonitorPanel from "../components/SwarmMonitorPanel.svelte";
  import { isTauriRuntime } from "../lib/runtime";
  import DefinitiveAnswerPanel from "../components/DefinitiveAnswerPanel.svelte";
  import AIConnectionsPanel from "../components/AIConnectionsPanel.svelte";
  import SystemConnectionsPanel from "../components/SystemConnectionsPanel.svelte";
  import IntakePanel from "../components/IntakePanel.svelte";
  import LiveExecutionTracker from "../components/LiveExecutionTracker.svelte";
  import OperationDoctrinePanel from "../components/OperationDoctrinePanel.svelte";
  import OperationPackagePanel from "../components/OperationPackagePanel.svelte";


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
  let breakdowns = $state<any[]>([]);
  let operationPackages = $state<any[]>([]);
  let swarmAllocations = $state<any[]>([]);
  let askerMotoruStatus = $state<any | null>(null);
  let dbSizeBytes = $state<number>(0);
  let aiProviderHealth = $state<any[]>([]);
  let systemConnectorHealth = $state<any[]>([]);

  let activeSection = $state("planning");
  let footerTab = $state("agent_stream"); // "planning", "decisions", "security", "connections", "execution"
  let globalError = $state<string | null>(null);
  let alarmMuted = $state(false);
  let lastAlarmKey = "";
  let runtimeMode = $state("browser_preview");
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

  function detectTauriRuntime() {
    return isTauriRuntime();
  }

  function readDemoStore(key: string, fallback: any) {
    try {
      const raw = localStorage.getItem(key);
      return raw ? JSON.parse(raw) : fallback;
    } catch {
      return fallback;
    }
  }

  function writeDemoStore(key: string, value: any) {
    localStorage.setItem(key, JSON.stringify(value));
  }

  function demoBreakdowns(taskId: string, request: string) {
    const source = request || "Kullanici talebi";
    const phases = [
      ["Cozumleme", "Konu / alt konu / kriter / alt kriter cikarimi"],
      ["Alternatif Analizi", "Real hayattaki alternatiflerin cikarimi"],
      ["Dogru Secimi", "Kabul edilmis dogrunun secimi"],
      ["Uygulanabilir Secim", "En iyi uygulanabilir secenegin secimi"],
      ["Kontrol ve Onay", "Kontrol, bagimsiz dogrulama ve son onay"]
    ];
    return phases.map((phase, index) => ({
      id: `${taskId}-bd-${index + 1}`,
      task_id: taskId,
      parent_id: null,
      level: index + 1,
      topic: phase[0],
      subtopic: phase[1],
      criterion: "Plan / etki alani / teknoloji / test / rollback zorunlulugu",
      subcriterion: "Rol ayrimi ve alt birim paketi",
      description: `${source} -> ${phase[0]}`,
      risk_pre_label: index === 4 ? "HIGH" : "MEDIUM",
      probable_connector: "user_instruction",
      decision_node_required: "Evet"
    }));
  }

  async function safeInvoke(cmd: string, args?: any): Promise<any> {
    if (detectTauriRuntime()) {
      return await invoke(cmd, args);
    }
    if (!import.meta.env.DEV) {
      throw new Error(`Tauri koprusu yok: ${cmd} komutu production ortaminda calistirilamaz.`);
    }
    await new Promise(resolve => setTimeout(resolve, 80));

    const demoTasksKey = "localControlPanel.demo.tasks";
    const demoDetailsKey = (taskId: string, type: string) => `localControlPanel.demo.${type}.${taskId}`;

    switch (cmd) {
      case "get_tasks_cmd":
        return readDemoStore(demoTasksKey, []);
      case "get_task_logs_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "logs"), []);
      case "get_decisions_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "decisions"), []);
      case "get_alternatives_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "alternatives"), []);
      case "get_approvals_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "approvals"), []);
      case "get_checkpoints_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "checkpoints"), []);
      case "get_tests_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "tests"), []);
      case "get_reports_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "reports"), []);
      case "get_task_breakdowns_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "breakdowns"), []);
      case "get_operation_packages_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "operationPackages"), []);
      case "get_swarm_allocations_cmd":
        return readDemoStore(demoDetailsKey(args?.taskId, "swarmAllocations"), []);
      case "get_asker_motoru_status_cmd":
        return { roots_checked: [], files: [] };
      case "sync_supabase_cmd":
        return { enabled: false, last_result: "onizleme", pushed_rows: 0 };
      case "get_db_size_cmd":
        return 0;
      case "get_system_health_cmd":
        return [];
      case "get_ai_provider_health_cmd":
        return [];
      case "get_system_connector_health_cmd":
        return [];
      case "create_task_cmd": {
        const demoTasks = readDemoStore(demoTasksKey, []);
        const id = `demo-${Date.now()}`;
        const task = {
          id,
          title: args.title,
          user_request: args.userRequest,
          status: "pending",
          planning_status: "planning_incomplete",
          execution_status: "not_started",
          current_gate: "Intake Gate",
          last_valid_state_id: null,
          risk_level: "high",
          approval_status: "browser_preview",
          created_at: new Date().toISOString()
        };
        writeDemoStore(demoTasksKey, [task, ...demoTasks]);
        writeDemoStore(demoDetailsKey(id, "breakdowns"), demoBreakdowns(id, args.userRequest));
        writeDemoStore(demoDetailsKey(id, "logs"), [
          {
            id: `${id}-log-1`,
            timestamp: new Date().toISOString(),
            level: "info",
            message: "Tarayici onizleme modunda gorev parcalandi.",
            gate_name: "Intake Gate"
          }
        ]);
        return task;
      }
      case "save_plan_cmd": {
        const demoTasks = readDemoStore(demoTasksKey, []).map((task: any) =>
          task.id === args.taskId
            ? {
                ...task,
                planning_status: "planning_complete",
                risk_level: args.plan?.risk_analysis || task.risk_level,
                current_gate: "Planning Gate"
              }
            : task
        );
        writeDemoStore(demoTasksKey, demoTasks);
        writeDemoStore(demoDetailsKey(args.taskId, "checkpoints"), [
          { id: `${args.taskId}-cp-1`, checkpoint_type: "planning_contract", status: "passed", result: "Plan, teknoloji, etki alani, test ve rollback mevcut." },
          { id: `${args.taskId}-cp-2`, checkpoint_type: "role_separation", status: "passed", result: "Yapan, koruyan, kontrol eden, dogrulayan ve onaylayan ayrildi." }
        ]);
        writeDemoStore(demoDetailsKey(args.taskId, "alternatives"), (args.plan?.alternatives || []).map((title: string, index: number) => ({
          id: `${args.taskId}-alt-${index + 1}`,
          decision_node_id: `${args.taskId}-bd-${index + 1}`,
          title,
          description: index === 2 ? "Secilen en iyi uygulanabilir alternatif." : "Gercek hayat alternatifi olarak degerlendirildi.",
          accuracy_score: index === 2 ? 9 : 7,
          safety_score: index === 2 ? 9 : 6,
          dependency_score: index === 2 ? 3 : 5,
          selected: index === 2 ? 1 : 0,
          reason: index === 2 ? args.plan?.selected_best_option_reason : "Kriter karsilastirmasinda elendi."
        })));
        writeDemoStore(demoDetailsKey(args.taskId, "operationPackages"), [
          {
            id: `${args.taskId}-pkg-1`,
            package_order: 1,
            package_type: "analysis_and_execution_contract",
            subject: args.plan?.topic,
            sub_topic: args.plan?.sub_topic,
            criterion: args.plan?.criterion,
            sub_criterion: args.plan?.sub_criterion,
            accepted_truth: args.plan?.accepted_correct_approach_reason,
            selected_best_alternative: args.plan?.alternatives?.[2] || args.plan?.selected_best_option_reason,
            operation_sequence: JSON.stringify(args.plan?.operation_sequence || []),
            technology: args.plan?.technology_selection,
            impact_area: args.plan?.impact_area,
            control_point: args.plan?.checkpoints?.[0] || "Kontrol noktasi",
            control_criteria: JSON.stringify(args.plan?.control_criteria || []),
            test_plan: JSON.stringify(args.plan?.test_criteria || []),
            rollback_plan: args.plan?.rollback_plan,
            executor_role: args.plan?.executor_role,
            correctness_guard_role: args.plan?.correctness_guard_role,
            controller_role: args.plan?.controller_role,
            independent_verifier_role: args.plan?.independent_verifier_role,
            final_approver_role: args.plan?.final_approver_role,
            status: "ready_for_subunit"
          }
        ]);
        return null;
      }
      case "execute_task_cmd": {
        const demoTasks = readDemoStore(demoTasksKey, []).map((task: any) =>
          task.id === args.taskId
            ? { ...task, status: "completed", execution_status: "completed", current_gate: "Report Gate" }
            : task
        );
        writeDemoStore(demoTasksKey, demoTasks);
        writeDemoStore(demoDetailsKey(args.taskId, "tests"), [
          { id: `${args.taskId}-test-1`, test_name: "browser_preview_contract", expected_result: "passed", actual_result: "passed", status: "passed" }
        ]);
        writeDemoStore(demoDetailsKey(args.taskId, "reports"), [
          {
            id: `${args.taskId}-report-1`,
            report_type: "final",
            content: "Tarayici onizleme modu: plan akisi, rol ayrimi, test ve rollback sozlesmesi dogrulandi. Gercek veritabani islemleri icin Tauri uygulamasi kullanilir."
          }
        ]);
        return { success: true, message: "Onizleme modunda 8 kapi akisi tamamlandi. Gercek uygulama icin Tauri runtime gerekir." };
      }      case "submit_approval_cmd":
        return null;
      case "rollback_task_cmd":
        return true;
      default:
        throw new Error(`Bilinmeyen komut: ${cmd}`);
    }
  }

  function playSiren() {
    if (alarmMuted) return;
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
      
      // Ã‡ift bip uyarÄ±sÄ± Ã§al ve bitir (SÃ¼rekli kafa Ã¼tÃ¼leyen dÃ¶ngÃ¼yÃ¼ engeller!)
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

      // TatlÄ± bir Ã§ift uyarÄ± melodisi Ã§al (Cevap sesini bastÄ±rmaz ve kafa karÄ±ÅŸtÄ±rmaz)
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
    const alarmKey = `${source}:${message}`;
    if (globalError === message && lastAlarmKey === alarmKey) {
      return;
    }

    globalError = message;
    lastAlarmKey = alarmKey;
    alarmMuted = false;
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
    speakReply(`Acil sistem alarmi. ${message}`, `critical:${message}`, true);
  }

  function muteAlarm() {
    alarmMuted = true;
    stopSiren();
    stopVoiceReply();
  }

  function clearAlarm() {
    globalError = null;
    lastAlarmKey = "";
    alarmMuted = false;
    stopSiren();
    stopVoiceReply();
  }

  function speakReply(text: string, key = text, force = true) {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      voiceAvailable = false;
      return;
    }

    if (!force && key === lastSpokenVoiceKey) return;
    if (!force && !voiceRepliesEnabled) return;

    lastSpokenVoiceKey = key;

    // EÄŸer otomatik (force = false) bir durum gÃ¼ncellemesi tetiklendiyse veya alarm ise,
    // kuyruktaki tÃ¼m eski bayat mesajlarÄ± temizle ve Ã§alan eski sesi iptal et.
    // Bu sayede aynÄ± anda sadece TEYÄ°T EDÄ°LMÄ°Å TEK BÄ°R SES Ã§alacaktÄ±r.
    if (!force || key.startsWith("critical")) {
      speechQueue = [];
      isSpeaking = false;
      window.speechSynthesis.cancel();
    }

    // MesajÄ± ses kuyruÄŸuna ekle (Mesaj kaybÄ±nÄ± Ã¶nler)
    speechQueue.push({ text, key });

    // EÄŸer ÅŸu an herhangi bir ses Ã§almÄ±yorsa kuyruk iÅŸlemcisini baÅŸlat
    if (!isSpeaking) {
      processSpeechQueue();
    }
  }

  function processSpeechQueue() {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) return;

    const synth = window.speechSynthesis;

    // EÄŸer kuyrukta Ã§alacak ses kalmadÄ±ysa durumu sÄ±fÄ±rla
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

    // Ses baÅŸarÄ±yla bittiÄŸinde kuyruktan Ã§Ä±kar ve sÄ±radakine geÃ§
    utterance.onend = () => {
      speechQueue.shift();
      processSpeechQueue();
    };

    // Ses hatayla kesildiÄŸinde veya Ã§almadÄ±ÄŸÄ±nda da takÄ±lmamasÄ± iÃ§in sÄ±radakine geÃ§
    utterance.onerror = (e) => {
      console.error("Speech Synthesis Error:", e);
      speechQueue.shift();
      processSpeechQueue();
    };

    synth.speak(utterance);
  }

  function stopVoiceReply() {
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      speechQueue = []; // KuyruÄŸu temizle
      isSpeaking = false;
      window.speechSynthesis.cancel(); // Mevcut Ã§almayÄ± durdur
    }
  }

  function toggleVoiceReplies() {
    voiceRepliesEnabled = !voiceRepliesEnabled;
    localStorage.setItem("voiceRepliesEnabled", String(voiceRepliesEnabled));

    if (voiceRepliesEnabled) {
      speakReply("Sesli cevap aÃ§Ä±ldÄ±.", "voice-enabled", true);
    } else {
      stopVoiceReply();
    }
  }

  $effect(() => {
    if (!globalError) {
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
      console.error("Yukleme hatasi:", err);
      raiseCriticalAlarm("Gorevler yuklenirken hata olustu", err);
    }
  }

  async function checkSystemHealth() {
    try {
      const issues: any[] = await safeInvoke("get_system_health_cmd");
      const blockers = issues.filter((issue) => issue.severity === "error");
      if (blockers.length > 0) {
        raiseCriticalAlarm(
          "Sistem kok dogrulamasi basarisiz",
          blockers.map((issue) => `${issue.code}: ${issue.message}`).join(" | ")
        );
      }
    } catch (err) {
      raiseCriticalAlarm("Sistem kok dogrulamasi calistirilamadi", err);
    }
  }

  async function refreshConnectionHealth(writeAudit = false) {
    try {
      aiProviderHealth = await safeInvoke("get_ai_provider_health_cmd", { writeAudit });
      systemConnectorHealth = await safeInvoke("get_system_connector_health_cmd", { writeAudit });
      askerMotoruStatus = await safeInvoke("get_asker_motoru_status_cmd");
      dbSizeBytes = await safeInvoke("get_db_size_cmd");
    } catch (err) {
      console.error("Baglanti health-check hatasi:", err);
      raiseCriticalAlarm("Baglanti health-check sirasinda hata olustu", err);
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
      breakdowns = await safeInvoke("get_task_breakdowns_cmd", { taskId });
      operationPackages = await safeInvoke("get_operation_packages_cmd", { taskId });
      swarmAllocations = await safeInvoke("get_swarm_allocations_cmd", { taskId });
    } catch (err) {
      console.error("Detay yukleme hatasi:", err);
      raiseCriticalAlarm("Gorev detaylari yuklenirken hata olustu", err);
    }
  }

  async function handleSelectTask(id: string | null) {
    selectedTaskId = id;
    if (id) {
      await refreshTaskDetails(id);
    }
  }


  async function handleCreateTask(title: string, userRequest: string) {
    try {
      const newTask: any = await safeInvoke("create_task_cmd", { title, userRequest });
      selectedTaskId = newTask.id;
      await loadTasks();
      speakReply("Gorev kaydedildi. Kesin cevap icin planlama ve guvenlik kapilari bekleniyor.", `task-created:${newTask.id}`, true);
    } catch (err) {
      console.error("Gorev olusturulamadi:", err);
      raiseCriticalAlarm("Gorev olusturulamadi", err);
    }
  }

  async function handleSavePlan(planInput: any) {
    if (!selectedTaskId) return;
    try {
      await safeInvoke("save_plan_cmd", { taskId: selectedTaskId, plan: planInput });
      await loadTasks();
      speakReply("Plan kaydedildi. Planlama alanlari dogrulandi.", `plan-saved:${selectedTaskId}`, true);
      alert("Plan kaydedildi, 17/17 alan dogrulandi.");
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
      speakReply(res.message || "Yurutme tamamlandi.", `execution:${selectedTaskId}:${res.message || ""}`, true);
      alert(res.message);
    } catch (err) {
      console.error("Yurutme hatasi:", err);
      raiseCriticalAlarm("Yurutme sirasinda hata olustu", err);
      await loadTasks();
    }
  }

  async function handleApproval(approvalId: string, approve: boolean, userNote: string, approverId: string, approverRole: string) {
    try {
      await safeInvoke("submit_approval_cmd", { approvalId, approve, userNote, approverId, approverRole });
      await loadTasks();
      speakReply(approve ? "Islem onaylandi." : "Islem reddedildi.", `approval:${approvalId}:${approve}`, true);
      alert(approve ? "Islem onaylandi." : "Islem reddedildi.");
    } catch (err) {
      console.error("Onay gonderme hatasi:", err);
      raiseCriticalAlarm("Onay islemi sirasinda hata olustu", err);
    }
  }

  async function handleRollback() {
    if (!selectedTaskId) return;
    try {
      const success: boolean = await safeInvoke("rollback_task_cmd", { taskId: selectedTaskId });
      await loadTasks();
      speakReply(success ? "Rollback basariyla tamamlandi." : "Geri alinacak bir snapshot bulunamadi.", `rollback:${selectedTaskId}:${success}`, true);
      alert(success ? "Rollback basariyla tamamlandi!" : "Geri alinacak bir snapshot bulunamadi.");
    } catch (err) {
      console.error("Rollback hatasi:", err);
      raiseCriticalAlarm("Rollback islemi sirasinda hata olustu", err);
    }
  }

  // 3 saniyede bir log ve durum gÃ¼ncellemesi yapalÄ±m (canlÄ± izleme)
  onMount(() => {
    runtimeMode = isTauriRuntime() ? "tauri_runtime" : "browser_preview";

    const savedVoiceSetting = localStorage.getItem("voiceRepliesEnabled");
    if (savedVoiceSetting !== null) {
      voiceRepliesEnabled = savedVoiceSetting === "true";
    }

    if (import.meta.env.DEV && new URLSearchParams(window.location.search).has("alarmTest")) {
      raiseCriticalAlarm(
        "Otomatik gorsel alarm testi",
        "Test amacli hata enjeksiyonu: alarm banner, aktif hata kayitlari ve sesli kritik alarm akisi dogrulaniyor."
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
        <h1>LOKAL BILGISAYAR</h1>
        <span>KONTROL PANELI</span>
      </div>
    </div>
    <div class="brain-display">
      <img src="/brain_logo.png" alt="AI Brain Core" />
    </div>
    <TaskTabs 
      tasks={tasks} 
      selectedTaskId={selectedTaskId} 
      onSelect={handleSelectTask} 
    />

  </div>

    <div class="main-workspace">
      <div class="progress-bar-container">
         <div class="progress-step" class:active={activeSection === 'planning'} class:done={activeSection !== 'planning'}>1. PLANLAMA (Gate 1)</div>
         <div class="progress-line"></div>
         <div class="progress-step" class:active={activeSection === 'decisions'} class:done={activeSection === 'security' || activeSection === 'execution'}>2. KARAR (Gate 2-4)</div>
         <div class="progress-line"></div>
         <div class="progress-step" class:active={activeSection === 'security'} class:done={activeSection === 'execution'}>3. ONAY (Gate 5-7)</div>
         <div class="progress-line"></div>
         <div class="progress-step" class:active={activeSection === 'execution'}>4. TEST & RAPOR (Gate 8)</div>
      </div>
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
      <div class="runtime-banner" class:real={runtimeMode === "tauri_runtime"}>
        {#if runtimeMode === "tauri_runtime"}
          GERCEK CALISMA MODU: Tauri koprusu aktif, veritabani ve sistem komutlari gercek kayda gider.
        {:else}
          TARAYICI ONIZLEME MODU: localhost:200 arayuzu cevap verir; gercek veritabani/yazma islemleri icin Tauri uygulamasi kullanilir.
        {/if}
      </div>
      <div class="navigation-tabs">
        <button class="nav-btn" class:active={activeSection === 'planning'} onclick={() => activeSection = 'planning'}>PLANLAMA (GATE 1)</button>
        <button class="nav-btn" class:active={activeSection === 'decisions'} onclick={() => activeSection = 'decisions'}>KARAR AGACI & ALTERNATIFLER (GATE 2-4)</button>
        <button class="nav-btn" class:active={activeSection === 'security'} onclick={() => activeSection = 'security'}>GUVENLIK DUVARI & ONAY (GATE 5-7)</button>
        <button class="nav-btn" class:active={activeSection === 'connections'} onclick={() => activeSection = 'connections'}>BAGLANTILAR</button>
        <button class="nav-btn" class:active={activeSection === 'execution'} onclick={() => activeSection = 'execution'}>TEST VE RAPOR (GATE 8)</button>
      </div>
      <div class="voice-controls">
        <button
          class="voice-btn"
          class:active={voiceRepliesEnabled}
          disabled={!voiceAvailable}
          onclick={toggleVoiceReplies}
        >
          {voiceRepliesEnabled ? "Sesli Cevap Acik" : "Sesli Cevap Kapali"}
        </button>
        <button class="voice-btn stop" disabled={!voiceAvailable} onclick={stopVoiceReply}>Cevap Sesini Durdur</button>
      </div>
    </div>

    {#if globalError}
      <div class="global-error-banner">
        <span class="error-icon">ALARM</span>
        <div class="error-message">
          <strong>SISTEM HATASI TESPIT EDILDI</strong>
          <span>{globalError}</span>
          <small>{alarmMuted ? "Alarm susturuldu; hata kaydi ekranda kalir." : "Ayni hata tekrar ederse ses yeniden baslatilmaz."}</small>
        </div>
        <div class="alarm-actions">
          <button class="alarm-action-btn" onclick={muteAlarm} disabled={alarmMuted}>Alarmi Sustur</button>
          <button class="alarm-action-btn secondary" onclick={clearAlarm}>Hatayi Kapat</button>
        </div>
      </div>

      <div class="alarm-history-panel">
        <div class="alarm-history-title">AKTIF HATA KAYITLARI</div>
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
      <OperationDoctrinePanel />
      <TaskDetail task={selectedTask} onExecute={handleExecute} />
      <OperationPackagePanel packages={operationPackages} />
      <DefinitiveAnswerPanel
        task={selectedTask}
        approvals={approvals}
        tests={tests}
        reports={reports}
        voiceRepliesEnabled={voiceRepliesEnabled}
        onSpeakAnswer={speakReply}
        onStopVoice={stopVoiceReply}
      />

      <LiveExecutionTracker task={selectedTask} breakdowns={breakdowns} />

      {#if activeSection === 'connections'}
        <AIConnectionsPanel providers={aiProviderHealth} onRefresh={() => refreshConnectionHealth(true)} />
        <SystemConnectionsPanel connectors={systemConnectorHealth} onRefresh={() => refreshConnectionHealth(true)} />
        <SwarmMonitorPanel allocations={swarmAllocations} taskId={selectedTaskId} />
        {#if askerMotoruStatus}
          <div class="asker-bridge-panel">
            <h3>Asker Motoru Durum Köprüsü</h3>
            <p>DB boyutu: {(dbSizeBytes / (1024 * 1024)).toFixed(2)} MB</p>
            {#each askerMotoruStatus.files as file}
              <div class="asker-file" class:missing={!file.exists}>
                <strong>{file.path}</strong>
                <pre>{file.preview}</pre>
              </div>
            {/each}
          </div>
        {/if}
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
          <StructuredReportPanel reports={reports} />
        {/if}
      {:else}
        <IntakePanel onCreate={handleCreateTask} />
      {/if}

    </div>

        <div class="workspace-footer">
      <div class="footer-tabs">
        <button class="footer-tab-btn" class:active={footerTab === 'agent_stream'} onclick={() => footerTab = 'agent_stream'}>AJAN ILETISIM & RAPORLAR</button>
        <button class="footer-tab-btn" class:active={footerTab === 'system_logs'} onclick={() => footerTab = 'system_logs'}>SISTEM & AUDIT LOGLARI</button>
      </div>
      <div class="footer-content">
        {#if footerTab === 'system_logs'}
          <LiveLog logs={logs} />
        {:else}
          <div class="agent-stream-panel">
             <!-- Ajan raporlarÄ± ve niyetlerini gÃ¶stereceÄŸimiz yalÄ±tÄ±lmÄ±ÅŸ alan -->
             <div class="stream-header">
               <h4>Operasyonel Ajan AkÄ±ÅŸÄ±</h4>
               <p>AjanlarÄ±n aldÄ±klarÄ± kararlar ve Ã¼rettikleri raporlar teknik loglardan baÄŸÄ±msÄ±z olarak burada listelenir.</p>
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
                 <div class="empty-stream">HenÃ¼z bir ajan raporu veya kararÄ± bulunmuyor.</div>
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

  .brain-display {
    width: 100%;
    padding: 15px 20px;
    background: #111112;
    border-bottom: 1px solid #1f1f21;
    display: flex;
    justify-content: center;
    align-items: center;
    box-sizing: border-box;
  }
  
  .brain-display img {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    box-shadow: 0 4px 15px rgba(245, 158, 11, 0.22);
    border: 1px solid #1f1f21;
    transition: transform 0.3s ease, box-shadow 0.3s ease;
  }
  
  .brain-display img:hover {
    transform: scale(1.02);
    box-shadow: 0 6px 20px rgba(245, 158, 11, 0.42);
  }

  .brand-logo {
    height: 32px;
    filter: drop-shadow(0 0 8px #f59e0b);
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
    color: #f59e0b;
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
    padding: 8px 15px;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    min-height: 48px;
  }

  .runtime-banner {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 10px;
    border: 1px solid #3b3b40;
    border-left: 4px solid #f59e0b;
    border-radius: 4px;
    background: #1a1710;
    color: #facc15;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.4px;
    text-transform: uppercase;
  }

  .runtime-banner.real {
    border-left-color: #47d18c;
    background: #101d17;
    color: #47d18c;
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
    background: #f59e0b;
    color: #15110a;
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

  .alarm-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .alarm-action-btn {
    border: 1px solid rgba(255, 215, 215, 0.55);
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    border-radius: 4px;
    padding: 8px 10px;
    cursor: pointer;
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.4px;
  }

  .alarm-action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.16);
  }

  .alarm-action-btn:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .alarm-action-btn.secondary {
    border-color: rgba(255, 179, 179, 0.28);
    color: #ffd7d7;
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
    border-bottom-color: #f59e0b;
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
    border-left: 3px solid #f59e0b;
    padding: 10px;
    border-radius: 4px;
  }
  .agent-msg strong { display: block; margin-bottom: 6px; color: #47d18c; font-size: 12px; }
  .agent-msg pre { margin: 0; font-family: monospace; font-size: 12px; color: #b8b8bf; white-space: pre-wrap; }
  .empty-stream { color: #8d8d95; font-size: 13px; font-style: italic; }
  .progress-bar-container { display: flex; align-items: center; justify-content: center; padding: 12px 20px; background: #0c0c0d; border-bottom: 1px solid #1f1f21; }
  .progress-step { font-size: 11px; font-weight: bold; color: #555; padding: 4px 10px; border-radius: 12px; background: #18181a; border: 1px solid #2d2d31; }
  .progress-step.active { color: #15110a; background: #f59e0b; border-color: #f59e0b; }
  .progress-step.done { color: #47d18c; border-color: #47d18c; }
  .progress-line { flex: 1; height: 2px; background: #2d2d31; margin: 0 10px; max-width: 50px; }
</style>

