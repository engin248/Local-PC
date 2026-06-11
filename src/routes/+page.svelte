<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
  import AlarmCardsPanel from "../components/AlarmCardsPanel.svelte";
  import IntakePanel from "../components/IntakePanel.svelte";
  import LiveExecutionTracker from "../components/LiveExecutionTracker.svelte";
  import OperationDoctrinePanel from "../components/OperationDoctrinePanel.svelte";
  import OperationPackagePanel from "../components/OperationPackagePanel.svelte";
  import SkillLibraryExplorer from "../components/SkillLibraryExplorer.svelte";
  import CommandCenterLayout from "../components/CommandCenterLayout.svelte";
  import { subscribeLiveFeed, parseMetadata, type LiveFeedEvent } from "../lib/liveFeed";
  import { speakText, stopSpeech, formatAlarmSpeech } from "../lib/voiceService";
  import { resolveFlowStage, isCommandCenterTask } from "../lib/commandFlow";


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
  let alarmCards = $state<any[]>([]);
  let commandFeed = $state<any[]>([]);
  let burhanEvents = $state<any[]>([]);
  let lastBurhanDispatch = $state<string | null>(null);
  let commandFlowStage = $derived(resolveFlowStage(selectedTask, !!lastBurhanDispatch));
  let askerMotoruLiveStatus = $state<any | null>(null);
  let activeAlarmEvents = $state<any[]>([]);

  let activeSection = $state("planning");
  let footerTab = $state("agent_stream"); // "planning", "decisions", "security", "connections", "execution"
  let globalError = $state<string | null>(null);
  let alarmMuted = $state(false);
  let lastAlarmKey = "";
  let runtimeMode = $state("browser_preview");
  let alarmEvents = $state<any[]>([]);
  const alarmSoundFailureThrottleMs = 10000;
  const criticalAlarmsAlwaysAudible = true;
  let lastAlarmSoundFailureAt = 0;
  let voiceRepliesEnabled = $state(true);
  let voiceAvailable = $state(true);
  let lastSpokenVoiceKey = "";
  let speechQueue = $state<{ text: string; key: string }[]>([]);
  let isSpeaking = $state(false);
  let criticalErrorUnlisten: UnlistenFn | null = null;
  let liveFeedUnlisteners: UnlistenFn[] = [];
  let criticalAlarmCounter = $state(0);
  let lastCriticalAlarmAt = $state<string | null>(null);
  let lastCriticalAlarmSource = $state<string | null>(null);
  let alarmPulse = $state(false);
  let alarmPulseTimer: ReturnType<typeof setTimeout> | null = null;
  let operationAuditTrail = $state<any[]>([]);
  let operatorId = $state("local-operator");

  let audioCtx: AudioContext | null = null;
  let alarmInterval: any = null;

  const operationAuditStorageKey = "localControlPanel.operationAuditLog";
  const operatorStorageKey = "localControlPanel.operatorId";

  function formatError(err: unknown) {
    if (err instanceof Error) return err.message;
    return String(err);
  }

  function detectTauriRuntime() {
    return isTauriRuntime();
  }

  function readFallbackStore(key: string, fallback: any) {
    try {
      const raw = localStorage.getItem(key);
      return raw ? JSON.parse(raw) : fallback;
    } catch (err) {
      const fallbackErr = new Error(`Yerel fallback depo okuma hatası: ${key}`);
      console.error(fallbackErr.message);
      raiseCriticalAlarm("Yerel fallback depo okuma hatası", err);
      return fallback;
    }
  }

  function getOperatorId() {
    try {
      const saved = localStorage.getItem(operatorStorageKey);
      if (saved && saved.trim().length > 0) {
        operatorId = saved;
        return saved;
      }
    } catch (err) {
      console.error("Operator kimlik okuma hatası:", err);
      raiseCriticalAlarm("Operator kimlik okuma hatası", err);
    }
    return operatorId || "local-operator";
  }

  function setOperatorId(nextOperatorId: string) {
    try {
      const normalized = String(nextOperatorId || "local-operator").trim() || "local-operator";
      operatorId = normalized;
      localStorage.setItem(operatorStorageKey, normalized);
    } catch (err) {
      console.error("Operator kimlik kaydetme hatası:", err);
      raiseCriticalAlarm("Operator kimlik kaydetme hatası", err);
    }
  }

  function sanitizeOperationMetadata(cmd: string, args?: any) {
    if (!args) return {};
    const clone: any = {};
    for (const [key, value] of Object.entries(args)) {
      if (["taskId", "approvalId", "title", "command", "correlation_id", "plan"].includes(key)) {
        clone[key] = value;
      }
    }
    return { ...clone, command: cmd };
  }

  function safeStringify(value: any): string {
    try {
      return JSON.stringify(value);
    } catch (err) {
      return JSON.stringify({ serialization_error: String(err) });
    }
  }

  function extractOperationTarget(cmd: string, args?: any) {
    if (args?.taskId) return { targetType: "task", targetId: String(args.taskId) };
    if (args?.approvalId) return { targetType: "approval", targetId: String(args.approvalId) };
    if (args?.id) return { targetType: "item", targetId: String(args.id) };
    return { targetType: null, targetId: null };
  }

  async function appendOperationAudit(input: {
    action: string;
    status: "PASS" | "FAIL" | "WARN";
    cmd?: string;
    args?: any;
    errorMessage?: string;
    details?: string;
    durationMs?: number;
    correlationId?: string | null;
    context?: any;
  }) {
    const { targetType, targetId } = extractOperationTarget(input.cmd || "", input.args);
    const normalizedStatus = input.status || "PASS";
    const payload = {
      actor: getOperatorId(),
      action: input.action,
      target_type: targetType,
      target_id: targetId,
      status: normalizedStatus,
      details: input.details || `Action executed: ${input.action}`,
      metadata_json: safeStringify({
        command: input.cmd || input.action,
        args: sanitizeOperationMetadata(input.cmd || input.action, input.args),
        context: input.context || {},
        duration_ms: input.durationMs || 0,
      }),
      error_message: input.errorMessage,
      correlation_id: input.correlationId || null,
    };

    if (detectTauriRuntime()) {
      try {
        await invoke("append_operation_audit_cmd", payload);
      } catch (err) {
        console.error("Audit kayıt hatası:", err);
        raiseCriticalAlarm("Audit kayıt hatası", err);
      }
      return;
    }

    if (import.meta.env.DEV) {
      const previous = readFallbackStore(operationAuditStorageKey, []);
      const logRecord = {
        id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
        actor: payload.actor,
        action: payload.action,
        target_type: payload.target_type,
        target_id: payload.target_id,
        status: payload.status,
        details: payload.details,
        metadata_json: payload.metadata_json,
        error_message: payload.error_message,
        correlation_id: payload.correlation_id,
        created_at: new Date().toLocaleString("tr-TR"),
      };
      previous.unshift(logRecord);
      writeFallbackStore(operationAuditStorageKey, previous.slice(0, 200));
      return;
    }
  }

  async function invokeWithAudit(cmd: string, args: any = undefined, options: {
    action?: string;
    status?: "PASS" | "FAIL" | "WARN";
    details?: string;
    correlationId?: string;
    context?: any;
  } = {}) {
    const action = options.action || cmd;
    const startedAt = Date.now();
    try {
      const result = detectTauriRuntime() ? await invoke(cmd, args) : await safeInvoke(cmd, args);
      await appendOperationAudit({
        action,
        status: options.status || "PASS",
        cmd,
        args,
        details: options.details || `PASS: ${action}`,
        correlationId: options.correlationId,
        context: {
          ...(options.context || {}),
          duration_ms: Date.now() - startedAt,
        },
        durationMs: Date.now() - startedAt,
      });
      await loadOperationAuditTrail();
      return result;
    } catch (err) {
      await appendOperationAudit({
        action,
        status: "FAIL",
        cmd,
        args,
        details: `FAIL: ${action}`,
        durationMs: Date.now() - startedAt,
        errorMessage: formatError(err),
        correlationId: options.correlationId,
        context: {
          ...(options.context || {}),
          duration_ms: Date.now() - startedAt,
        },
      });
      await loadOperationAuditTrail();
      throw err;
    }
  }

  async function loadOperationAuditTrail() {
    if (detectTauriRuntime()) {
      try {
        operationAuditTrail = await safeInvoke("get_operation_audit_logs_cmd", { limit: 20 });
        return;
      } catch (err) {
        console.error("Operasyon audit logu yüklenemedi:", err);
        raiseCriticalAlarm("Operasyon audit logu yüklenemedi", err);
      }
    }

    if (import.meta.env.DEV) {
      operationAuditTrail = readFallbackStore(operationAuditStorageKey, []);
    } else {
      operationAuditTrail = [];
    }
  }

  function writeFallbackStore(key: string, value: any) {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (err) {
      console.error("Yerel fallback depo yazma hatası:", err);
      raiseCriticalAlarm("Yerel fallback depo yazma hatası", err);
    }
  }

  function withPreviewSource(rows: any[], sourceKind: "localStorage" | "mock") {
    return rows.map((row) => ({
      ...row,
      source_kind: row.source_kind || sourceKind,
      preview: true,
      health: row.health || row.status || "unavailable",
      last_checked_at: row.last_checked_at || new Date().toLocaleString("tr-TR"),
    }));
  }

  function generateFallbackBreakdowns(taskId: string, request: string) {
    const source = request || "Kullanıcı talebi";
    const phases = [
      ["Çözümleme", "Konu / alt konu / kriter / alt kriter çıkarımı"],
      ["Alternatif Analizi", "Gerçek hayattaki alternatiflerin çıkarımı"],
      ["Doğru Seçim", "Kabul edilen en doğru seçeneğin seçilmesi"],
      ["Uygulanabilir Seçim", "En iyi uygulanabilir seçeneğin seçimi"],
      ["Kontrol ve Onay", "Kontrol, bağımsız doğrulama ve son onay"]
    ];
    return phases.map((phase, index) => ({
      id: `${taskId}-bd-${index + 1}`,
      task_id: taskId,
      parent_id: null,
      level: index + 1,
      topic: phase[0],
      subtopic: phase[1],
      criterion: "Plan / etki alanı / teknoloji / test / rollback zorunluluğu",
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
      throw new Error(`Tauri köprüsü yok: ${cmd} komutu üretim ortamında çalıştırılamaz.`);
    }
    await new Promise(resolve => setTimeout(resolve, 80));

    const offlineTasksKey = "localControlPanel.tasks.offline";
    const offlineDetailsKey = (taskId: string, type: string) =>
      `localControlPanel.tasksOffline.${type}.${taskId}`;

    switch (cmd) {
      case "get_tasks_cmd":
        return readFallbackStore(offlineTasksKey, []);
      case "get_task_logs_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "logs"), []);
      case "get_decisions_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "decisions"), []);
      case "get_alternatives_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "alternatives"), []);
      case "get_approvals_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "approvals"), []);
      case "get_checkpoints_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "checkpoints"), []);
      case "get_tests_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "tests"), []);
      case "get_reports_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "reports"), []);
      case "get_task_breakdowns_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "breakdowns"), []);
      case "get_operation_packages_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "operationPackages"), []);
      case "get_swarm_allocations_cmd":
        return readFallbackStore(offlineDetailsKey(args?.taskId, "swarmAllocations"), []);
      case "get_asker_motoru_status_cmd":
        return {
          roots_checked: [],
          root_sources: [
            {
              kind: "windows",
              source_kind: "unavailable",
              source_path: null,
              health: "unavailable",
              error: "PREVIEW / MOCK: ASKER_MOTORU_WINDOWS_ROOT bağlı değil."
            },
            {
              kind: "linux",
              source_kind: "unavailable",
              source_path: null,
              health: "unavailable",
              error: "PREVIEW / MOCK: ASKER_MOTORU_LINUX_ROOT bağlı değil."
            }
          ],
          files: []
        };
      case "sync_supabase_cmd":
        return { enabled: false, last_result: "önizleme", pushed_rows: 0 };
      case "get_db_size_cmd":
        return 0;
      case "get_system_health_cmd":
        return [];
      case "get_ai_provider_health_cmd":
        return withPreviewSource(readFallbackStore("localControlPanel.preview.aiProviderHealth", [
          {
            id: "browser-preview-ai",
            name: "Browser Preview AI Bridge",
            provider_type: "preview",
            model: "none",
            endpoint: "bağlı değil",
            enabled: false,
            status: "mock",
            health: "unavailable",
            api_key_status: "not_checked",
            dependency_level: "low",
            network_required: false,
            allowed_task_types: [],
            last_error: "PREVIEW / MOCK: Tauri runtime olmadan gerçek AI provider health çalışmaz."
          }
        ]), "mock");
      case "get_system_connector_health_cmd":
        return withPreviewSource(readFallbackStore("localControlPanel.preview.systemConnectorHealth", [
          {
            id: "browser-preview-connector",
            name: "Browser Preview Connector Bridge",
            connector_type: "preview",
            source_path: null,
            endpoint: null,
            target: "bağlı değil",
            permissions: [],
            enabled: false,
            read_only: true,
            dependency_level: "low",
            live_system: false,
            network_required: false,
            allowed_actions: [],
            approval_required_actions: [],
            rollback_required_actions: [],
            test_required_actions: [],
            status: "mock",
            health: "unavailable",
            last_error: "PREVIEW / MOCK: Tauri runtime olmadan gerçek connector health çalışmaz."
          }
        ]), "mock");
      case "get_alarm_cards_cmd":
        return withPreviewSource([
          ...(args?.runtimeAlarms || []).map((alarm: any, index: number) => ({
            id: `runtime-preview-${index}`,
            title: alarm.source || "Runtime alarm",
            source_kind: "mock",
            health: "runtime_only",
            runtime_only: true,
            source_path: null,
            last_checked: alarm.timestamp || new Date().toLocaleString("tr-TR"),
            error: "runtime only",
            details: alarm.message || "PREVIEW / MOCK runtime alarm"
          })),
          {
            id: "browser-preview-alarm",
            title: "SISTEM_ALARM_DURUMU.json",
            source_kind: "mock",
            health: "unavailable",
            runtime_only: false,
            source_path: null,
            last_checked: new Date().toLocaleString("tr-TR"),
            error: "PREVIEW / MOCK: JSON/SQLite alarm kaynağı bağlı değil.",
            details: "bağlı değil"
          }
        ], "mock");
      case "create_task_cmd": {
        const offlineTasks = readFallbackStore(offlineTasksKey, []);
        const id = `offline-${Date.now()}`;
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
        writeFallbackStore(offlineTasksKey, [task, ...offlineTasks]);
        writeFallbackStore(offlineDetailsKey(id, "breakdowns"), generateFallbackBreakdowns(id, args.userRequest));
        writeFallbackStore(offlineDetailsKey(id, "logs"), [
          {
            id: `${id}-log-1`,
            timestamp: new Date().toISOString(),
            level: "info",
            message: "Tarayıcı önizleme modunda görev parçalandı.",
            gate_name: "Intake Gate"
          }
        ]);
        return task;
      }
      case "save_plan_cmd": {
        const offlineTasks = readFallbackStore(offlineTasksKey, []).map((task: any) =>
          task.id === args.taskId
            ? {
                ...task,
                planning_status: "planning_complete",
                risk_level: args.plan?.risk_analysis || task.risk_level,
                current_gate: "Planning Gate"
              }
            : task
        );
        writeFallbackStore(offlineTasksKey, offlineTasks);
        writeFallbackStore(offlineDetailsKey(args.taskId, "checkpoints"), [
          { id: `${args.taskId}-cp-1`, checkpoint_type: "planning_contract", status: "passed", result: "Plan, teknoloji, etki alanı, test ve rollback mevcut." },
          { id: `${args.taskId}-cp-2`, checkpoint_type: "role_separation", status: "passed", result: "Yapan, koruyan, denetleyen, doğrulayan ve onaylayan ayrıldı." }
        ]);
        writeFallbackStore(offlineDetailsKey(args.taskId, "alternatives"), (args.plan?.alternatives || []).map((title: string, index: number) => ({
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
        writeFallbackStore(offlineDetailsKey(args.taskId, "operationPackages"), [
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
        const offlineTasks = readFallbackStore(offlineTasksKey, []).map((task: any) =>
          task.id === args.taskId
            ? { ...task, status: "completed", execution_status: "completed", current_gate: "Report Gate" }
            : task
        );
        writeFallbackStore(offlineTasksKey, offlineTasks);
        writeFallbackStore(offlineDetailsKey(args.taskId, "tests"), [
          { id: `${args.taskId}-test-1`, test_name: "browser_preview_contract", expected_result: "passed", actual_result: "passed", status: "passed" }
        ]);
        writeFallbackStore(offlineDetailsKey(args.taskId, "reports"), [
          {
            id: `${args.taskId}-report-1`,
            report_type: "final",
            content: "Tarayıcı önizleme modu: plan akışı, rol ayrımı, test ve rollback sözleşmesi doğrulandı. Gerçek veritabanı işlemleri için Tauri uygulaması kullanılır."
          }
        ]);
        return { success: true, message: "Önizleme modunda 8 kapı akışı tamamlandı. Gerçek uygulama için Tauri runtime gerekir." };
      }      case "submit_approval_cmd":
        return null;
      case "rollback_task_cmd":
        return true;
      case "submit_command_sentence_cmd": {
        const offlineTasks = readFallbackStore(offlineTasksKey, []);
        const id = `offline-${Date.now()}`;
        const sentence = args?.sentence || "";
        const task = {
          id,
          title: sentence.slice(0, 80),
          user_request: sentence,
          status: "pending",
          planning_status: "planning_incomplete",
          execution_status: "not_started",
          current_gate: "Command Center Gate",
          last_valid_state_id: null,
          risk_level: "medium",
          approval_status: "browser_preview",
          created_at: new Date().toISOString()
        };
        writeFallbackStore(offlineTasksKey, [task, ...offlineTasks]);
        return {
          task,
          platforms: ["burhan_command", "codex", "open_agent_manager", "education_office"],
          feed_id: `feed-${id}`,
          alarm_scan: [],
          burhan_message: "Albay Burhan emri aldı. Önizleme modu."
        };
      }
      case "get_live_command_feed_cmd":
        return readFallbackStore("localControlPanel.commandFeed", []);
      case "get_alarm_codes_cmd":
        return [{ code: "011", title: "Birinci Algoritma İzleme", severity: "critical", auto_speak: true }];
      case "get_active_alarm_events_cmd":
        return [];
      case "scan_algorithm_health_cmd":
        return { triggered_codes: [], events: [] };
      case "get_asker_motoru_live_status_cmd":
        return {
          connected: false,
          api_base_url: "http://127.0.0.1:3100",
          health: "disabled",
          last_error: "PREVIEW / MOCK: Canlı API devre dışı."
        };
      case "get_pinokio_health_cmd":
        return ["unavailable", "PREVIEW / MOCK: Pinokio erişilemiyor."];
      default:
        throw new Error(`Bilinmeyen komut: ${cmd}`);
    }
  }

  function playSiren() {
    if (alarmMuted && !criticalAlarmsAlwaysAudible) return;
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
      
      // Çift bip uyarısı çal ve bitir (Sürekli kafa üreten veya döngüyü uzatan sesi engeller!)
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
            const now = Date.now();
            if (now - lastAlarmSoundFailureAt > alarmSoundFailureThrottleMs) {
              lastAlarmSoundFailureAt = now;
              appendAlarmEvent("Bip sesi üretilemedi", err);
            }
          }
        }, delay);
      };

      // Tatlı bir çift uyarı melodisi çal (Cevap sesini bastırmaz ve kafa karıştırmaz)
      playBeep(0, 880);
      playBeep(180, 1100);
    } catch (e) {
      console.error("Siren sesi çalışma hatası:", e);
      const now = Date.now();
      if (now - lastAlarmSoundFailureAt > alarmSoundFailureThrottleMs) {
        lastAlarmSoundFailureAt = now;
        appendAlarmEvent("Alarm ses sistemi hatası", e);
      }
    }
  }

  function stopSiren() {
    if (alarmInterval) {
      clearInterval(alarmInterval);
      alarmInterval = null;
    }
  }

  function appendAlarmEvent(source: string, err: unknown) {
    const message = `${source}: ${formatError(err)}`;
    const alarmKey = `${source}:${message}`;
    const isRepeatedError = globalError === message && lastAlarmKey === alarmKey;

    globalError = message;
    lastAlarmKey = alarmKey;
    criticalAlarmCounter += 1;
    lastCriticalAlarmAt = new Date().toLocaleString("tr-TR");
    lastCriticalAlarmSource = source;
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

    alarmPulse = false;
    if (alarmPulseTimer) {
      clearTimeout(alarmPulseTimer);
      alarmPulseTimer = null;
    }
    alarmPulse = true;
    alarmPulseTimer = setTimeout(() => {
      alarmPulse = false;
      alarmPulseTimer = null;
    }, 1800);
    return isRepeatedError ? " (tekrar)" : "";
  }

  function raiseCriticalAlarm(source: string, err: unknown) {
    const message = `${source}: ${formatError(err)}`;
    const repeatedSuffix = appendAlarmEvent(source, err);
    playSiren();
    speakReply(`Acil sistem alarmi. ${message}${repeatedSuffix}`, `critical:${message}:${Date.now()}`, true);
  }

  function muteAlarm() {
    stopSiren();
    stopVoiceReply();
  }

  function clearAlarm() {
    globalError = null;
    lastAlarmKey = "";
    alarmMuted = false;
    stopSiren();
    stopVoiceReply();
    alarmPulse = false;
    if (alarmPulseTimer) {
      clearTimeout(alarmPulseTimer);
      alarmPulseTimer = null;
    }
  }

  function speakReply(text: string, key = text, force = true) {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      voiceAvailable = false;
      return;
    }

    if (!force && key === lastSpokenVoiceKey) return;
    if (!force && !voiceRepliesEnabled) return;

    lastSpokenVoiceKey = key;
    if (speakText(text, key, force)) return;

    // Eğer otomatik (force = false) bir durum güncellemesi tetiklendiyse veya alarm ise,
    // kuyruktaki tüm eski bayat mesajları temizle ve çalan eski sesi iptal et.
    // Bu sayede aynı anda sadece TEK BİR SES çalacaktır.
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
      raiseCriticalAlarm("Seslendirme motoru hatası", e);
      speechQueue.shift();
      processSpeechQueue();
    };

    synth.speak(utterance);
  }

  function stopVoiceReply() {
    stopSpeech();
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      speechQueue = [];
      isSpeaking = false;
      window.speechSynthesis.cancel();
    }
  }

  function handleLiveFeedEvent(event: LiveFeedEvent) {
    const feedItem = {
      id: `${event.timestamp}-${event.source}`,
      event_type: event.event_type,
      source: event.source,
      message: event.message,
      task_id: event.task_id,
      metadata_json: event.metadata_json,
      created_at: event.timestamp,
      timestamp: event.timestamp,
    };

    if (event.event_type === "burhan-dispatch") {
      burhanEvents = [feedItem, ...burhanEvents].slice(0, 20);
      lastBurhanDispatch = event.message;
      if (voiceRepliesEnabled) {
        speakReply(`Albay Burhan emir dağıttı. ${event.message}`, `burhan:${event.timestamp}`, true);
      }
    }

    if (event.event_type === "command-submitted" || event.event_type === "live-feed") {
      commandFeed = [feedItem, ...commandFeed].slice(0, 30);
    }

    if (event.event_type === "agent-status" && event.task_id) {
      if (!selectedTaskId) selectedTaskId = event.task_id;
      void refreshTaskDetails(event.task_id);
    }

    if (event.event_type === "report-returned" && voiceRepliesEnabled) {
      speakReply(`Rapor hazır. ${event.message}`, `report-live:${event.timestamp}`, true);
    }

    if (event.event_type === "alarm-code") {
      const metadata = parseMetadata<{ speak_text?: string; code?: string }>(event.metadata_json);
      const speech = metadata?.speak_text
        || formatAlarmSpeech(metadata?.code || "011", "Sistem alarmı", event.message);
      raiseCriticalAlarm(event.source, event.message);
      speakReply(speech, `alarm:${event.timestamp}`, true);
    }
  }

  async function loadCommandFeed() {
    try {
      commandFeed = await safeInvoke("get_live_command_feed_cmd", { limit: 50 });
    } catch (err) {
      console.error("Komut akışı yüklenemedi:", err);
    }
  }

  async function loadActiveAlarms() {
    try {
      activeAlarmEvents = await safeInvoke("get_active_alarm_events_cmd", { limit: 20 });
      if (selectedTaskId) {
        await safeInvoke("scan_algorithm_health_cmd", { taskId: selectedTaskId });
      }
    } catch (err) {
      console.error("Alarm taraması başarısız:", err);
    }
  }

  async function handleCommandSubmitted(result: any) {
    if (result?.task?.id) {
      selectedTaskId = result.task.id;
      lastBurhanDispatch = result.burhan_message || null;
    }
    await loadTasks();
    await loadCommandFeed();
    await loadActiveAlarms();
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
    if (!globalError) {
      stopSiren();
    }
  });

  async function loadTasks() {
    try {
      tasks = await safeInvoke("get_tasks_cmd");
      const taskIds = tasks.map((task: any) => task.id);

      if (tasks.length === 0) {
        selectedTaskId = null;
      } else if (!selectedTaskId || !taskIds.includes(selectedTaskId)) {
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
          "Sistem sağlık kontrolü başarısız",
          blockers.map((issue) => `${issue.code}: ${issue.message}`).join(" | ")
        );
      }
    } catch (err) {
      raiseCriticalAlarm("Sistem sağlık kontrolü çalıştırılamadı", err);
    }
  }

  async function refreshConnectionHealth(writeAudit = false) {
    try {
      aiProviderHealth = await safeInvoke("get_ai_provider_health_cmd", { writeAudit });
      systemConnectorHealth = await safeInvoke("get_system_connector_health_cmd", { writeAudit });
      askerMotoruStatus = await safeInvoke("get_asker_motoru_status_cmd");
      askerMotoruLiveStatus = await safeInvoke("get_asker_motoru_live_status_cmd");
      alarmCards = await safeInvoke("get_alarm_cards_cmd", { runtimeAlarms: alarmEvents });
      await loadActiveAlarms();
      dbSizeBytes = await safeInvoke("get_db_size_cmd");
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
      breakdowns = await safeInvoke("get_task_breakdowns_cmd", { taskId });
      operationPackages = await safeInvoke("get_operation_packages_cmd", { taskId });
      swarmAllocations = await safeInvoke("get_swarm_allocations_cmd", { taskId });
    } catch (err) {
      console.error("Detay yükleme hatası:", err);
      raiseCriticalAlarm("Görev detayları yüklenirken hata oluştu", err);
    }
  }

  async function handleSelectTask(id: string | null) {
    selectedTaskId = id;
    if (id) {
      await refreshTaskDetails(id);
    }
  }


  async function handleCreateTask(_title: string, _userRequest: string) {
    speakReply(
      "Görev yalnızca üst panelden Albay Burhan'a atanır.",
      "intake-blocked",
      true,
    );
    raiseCriticalAlarm(
      "Akış ihlali",
      "Intake paneli devre dışı. Önce Panel 1'den görev atayın.",
    );
  }

  async function handleSavePlan(planInput: any) {
    if (!selectedTaskId) return;
    try {
      await invokeWithAudit("save_plan_cmd", { taskId: selectedTaskId, plan: planInput }, {
        action: "save_plan",
        details: `Plan saved for task ${selectedTaskId}`,
        context: {
          before: { task_id: selectedTaskId, status: selectedTask?.status || "unknown" },
          after: { risk_level: planInput?.risk_analysis, plan_summary: Object.keys(planInput || {}).length },
        },
      });
      await loadTasks();
      await loadOperationAuditTrail();
      speakReply("Plan kaydedildi. Planlama alanları doğrulandı.", `plan-saved:${selectedTaskId}`, true);
      alert("Plan kaydedildi, 17/17 alan doğrulandı.");
    } catch (err) {
      console.error("Plan kaydedilemedi:", err);
      raiseCriticalAlarm("Plan kaydedilemedi", err);
    }
  }

  async function handleExecute() {
    if (!selectedTaskId) return;
    if (!isCommandCenterTask(selectedTask)) {
      speakReply("Operasyon kilitli. Önce görev atayın.", "execute-blocked", true);
      raiseCriticalAlarm("Operasyon kilitli", "Komuta panelinden görev atılmadan operasyon başlatılamaz.");
      return;
    }
    try {
      const beforeTask = selectedTask;
      const res: any = await invokeWithAudit("execute_task_cmd", { taskId: selectedTaskId });
      await loadTasks();
      await refreshTaskDetails(selectedTaskId);
      const afterTask = selectedTask;
      await loadOperationAuditTrail();
      await appendOperationAudit({
        action: "execute_task_result",
        cmd: "execute_task_cmd",
        status: res?.success ? "PASS" : "WARN",
        args: { taskId: selectedTaskId },
        details: `Execution for task ${selectedTaskId} => ${res?.success ? "success" : "not-ok"} (${res?.message || "no message"})`,
        context: {
          before: beforeTask,
          after: afterTask,
          result: res,
        },
      });
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
      await invokeWithAudit("submit_approval_cmd", { approvalId, approve, userNote, approverId, approverRole }, {
        action: "submit_approval",
        details: `${approve ? "Approve" : "Reject"} approval ${approvalId}`,
        context: {
          before: { status: approvals.find((approval: any) => approval.id === approvalId)?.status || "unknown" },
          after: { approval_id: approvalId, approve, approver_role: approverRole },
        },
      });
      await loadTasks();
      await loadOperationAuditTrail();
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
      const beforeTask = selectedTask;
      const success: boolean = await invokeWithAudit("rollback_task_cmd", { taskId: selectedTaskId });
      await loadTasks();
      await refreshTaskDetails(selectedTaskId);
      const afterTask = selectedTask;
      await loadOperationAuditTrail();
      await appendOperationAudit({
        action: "rollback_task_result",
        cmd: "rollback_task_cmd",
        status: success ? "PASS" : "WARN",
        args: { taskId: selectedTaskId },
        details: success ? `Rollback completed for ${selectedTaskId}` : `Rollback failed/no snapshot for ${selectedTaskId}`,
        context: {
          before: beforeTask,
          after: afterTask,
          success,
        },
      });
      speakReply(success ? "Rollback başarıyla tamamlandı." : "Geri alınacak bir snapshot bulunamadı.", `rollback:${selectedTaskId}:${success}`, true);
      alert(success ? "Rollback başarıyla tamamlandı!" : "Geri alınacak bir snapshot bulunamadı.");
    } catch (err) {
      console.error("Rollback hatası:", err);
      raiseCriticalAlarm("Rollback işlemi sırasında hata oluştu", err);
    }
  }

  // 1 saniyede bir komuta merkezi güncellemesi (canlı izleme)
  onMount(() => {
    runtimeMode = isTauriRuntime() ? "tauri_runtime" : "browser_preview";
    let isMounted = true;
    setOperatorId(getOperatorId());

    const savedVoiceSetting = localStorage.getItem("voiceRepliesEnabled");
    if (savedVoiceSetting !== null) {
      voiceRepliesEnabled = savedVoiceSetting === "true";
    }

    const globalErrorHandler = (event: ErrorEvent) => {
      const detail = event.error instanceof Error ? event.error.message : String(event.message || event.error || "Bilinmeyen hata");
      raiseCriticalAlarm("Beklenmeyen istemci hatası", detail);
    };

    const unhandledRejectionHandler = (event: PromiseRejectionEvent) => {
      const reason = event.reason;
      const detail = reason instanceof Error ? reason.message : String(reason || "Bilinmeyen promise hatası");
      raiseCriticalAlarm("İşlenmemiş Promise hatası", detail);
    };

    const resourceErrorHandler = (event: ErrorEvent) => {
      if (event.error) return;
      const detail = event.message || "Kaynak yükleme hatası";
      raiseCriticalAlarm("Kaynak yükleme hatası", detail);
    };

    window.addEventListener("error", globalErrorHandler);
    window.addEventListener("error", resourceErrorHandler, true);
    window.addEventListener("unhandledrejection", unhandledRejectionHandler);

    void (async () => {
      if (!isTauriRuntime() || !isMounted) return;
      try {
        const unlisten = await listen("critical-error", (event) => {
          const payload = event.payload as Record<string, unknown>;
          const source = typeof payload.source === "string" ? payload.source : typeof payload.command === "string" ? payload.command : "Backend kritik hata";
          const message = typeof payload.message === "string"
            ? payload.message
            : typeof payload.error === "string"
              ? payload.error
              : JSON.stringify(payload);
          const command = typeof payload.command === "string" ? payload.command : "";
          raiseCriticalAlarm(source, message);
        });

        if (!isMounted) {
          unlisten();
          return;
        }

        criticalErrorUnlisten = unlisten;
        liveFeedUnlisteners = await subscribeLiveFeed(handleLiveFeedEvent);
    } catch (error) {
      console.error("Tauri kritik hata dinleyicisi kurulamadı:", error);
      raiseCriticalAlarm("Kritik hata dinleyicisi başlatılamadı", error);
    }
  })();

    checkSystemHealth();
    refreshConnectionHealth(true);
    loadTasks();
    loadCommandFeed();
    loadOperationAuditTrail();
    const interval = setInterval(() => {
      loadTasks();
      loadCommandFeed();
      loadActiveAlarms();
      loadOperationAuditTrail();
    }, 1000);
    return () => {
      isMounted = false;
      clearInterval(interval);
      for (const unlisten of liveFeedUnlisteners) {
        unlisten();
      }
      liveFeedUnlisteners = [];
      if (criticalErrorUnlisten) {
        criticalErrorUnlisten();
        criticalErrorUnlisten = null;
      }
      window.removeEventListener("error", globalErrorHandler);
      window.removeEventListener("error", resourceErrorHandler, true);
      window.removeEventListener("unhandledrejection", unhandledRejectionHandler);
      if (alarmPulseTimer) {
        clearTimeout(alarmPulseTimer);
        alarmPulseTimer = null;
      }
    };
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
          GERÇEK ÇALIŞMA MODU: Tauri köprüsü aktif, veritabanı ve sistem komutları gerçek kayda gider.
        {:else}
          TARAYICI ÖNİZLEME MODU: localhost:200 arayüzü cevap verir; gerçek veritabanı/yazma işlemleri için Tauri uygulaması kullanılır.
        {/if}
      </div>
      <div class="critical-alarm-indicator" class:active={!!globalError} class:idle={criticalAlarmCounter === 0} class:pulsing={alarmPulse}>
        <span class="indicator-dot"></span>
        {#if globalError}
          <span>KRİTİK ALARM: {criticalAlarmCounter} olay (en son: {lastCriticalAlarmSource || "bilinmiyor"})</span>
        {:else if criticalAlarmCounter > 0}
          <span>Son kritik hata: {lastCriticalAlarmSource || "sistem"} ({lastCriticalAlarmAt})</span>
        {:else}
          <span>Kritik alarm hattı izleme: AKTİF</span>
        {/if}
      </div>
      <div class="navigation-tabs">
        <button class="nav-btn" class:active={activeSection === 'planning'} onclick={() => activeSection = 'planning'}>PLANLAMA (GATE 1)</button>
        <button class="nav-btn" class:active={activeSection === 'decisions'} onclick={() => activeSection = 'decisions'}>KARAR AGACI & ALTERNATIFLER (GATE 2-4)</button>
        <button class="nav-btn" class:active={activeSection === 'security'} onclick={() => activeSection = 'security'}>GUVENLIK DUVARI & ONAY (GATE 5-7)</button>
        <button class="nav-btn" class:active={activeSection === 'skills'} onclick={() => activeSection = 'skills'}>BECERİ KÜTÜPHANESİ</button>
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
      <div class="operation-audit-mini-panel">
        <div class="operation-audit-title">İŞLEM KAYIT ÖZETİ</div>
        {#if !operationAuditTrail.length}
          <div class="operation-audit-empty">Henüz işlem kaydı yok.</div>
        {:else}
      <div class="operation-audit-scroll">
            {#each operationAuditTrail as audit}
              <div class="operation-audit-row">
                <span>{audit.created_at}</span>
                <span>{audit.actor}</span>
                <span>{audit.action}</span>
                <span class="audit-details">{audit.details}</span>
                <span class:pass={audit.status === 'PASS'} class:warn={audit.status === 'WARN'} class:fail={audit.status === 'FAIL'}>
                  {audit.status}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    {#if globalError}
      <div class="global-error-banner">
        <span class="error-icon">ALARM</span>
        <div class="error-message">
          <strong>SISTEM HATASI TESPIT EDILDI</strong>
          <span>{globalError}</span>
            <small>{criticalAlarmsAlwaysAudible ? "Kritik alarm panosu ve sesli alarm her hata anında aktif." : "Aynı hata tekrarlanırsa ses otomatik olarak yeniden başlatılmaz."}</small>
        </div>
        <div class="alarm-actions">
          <button class="alarm-action-btn" onclick={muteAlarm}>Geçici Sessizleştir</button>
          <button class="alarm-action-btn secondary" onclick={clearAlarm}>Hata Kaydını Kapat</button>
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
      <CommandCenterLayout
        commandFeed={commandFeed}
        burhanEvents={burhanEvents}
        lastBurhanDispatch={lastBurhanDispatch}
        selectedTaskId={selectedTaskId}
        swarmAllocations={swarmAllocations}
        reports={reports}
        voiceRepliesEnabled={voiceRepliesEnabled}
        flowStage={commandFlowStage}
        onCommandSubmitted={handleCommandSubmitted}
        onSpeakReport={(text, key) => speakReply(text, key, true)}
      />
      <OperationDoctrinePanel />
      <TaskDetail task={selectedTask} onExecute={handleExecute} operationsAllowed={commandFlowStage !== "awaiting_task"} />
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

      <LiveExecutionTracker task={selectedTask} breakdowns={breakdowns} allocations={swarmAllocations} />

      {#if activeSection === 'connections'}
        <AIConnectionsPanel providers={aiProviderHealth} onRefresh={() => refreshConnectionHealth(true)} />
        <SystemConnectionsPanel connectors={systemConnectorHealth} onRefresh={() => refreshConnectionHealth(true)} />
        <AlarmCardsPanel alarms={alarmCards} />
        <SwarmMonitorPanel allocations={swarmAllocations} taskId={selectedTaskId} />
        {#if activeAlarmEvents.length > 0}
          <div class="alarm-code-panel">
            <h3>Aktif Alarm Kodları</h3>
            {#each activeAlarmEvents as alarm}
              <div class="alarm-code-item">
                <strong>{alarm.alarm_code || "000"}</strong>
                <span>{alarm.source}</span>
                <p>{alarm.message}</p>
              </div>
            {/each}
          </div>
        {/if}
        {#if askerMotoruLiveStatus}
          <div class="asker-live-panel">
            <h3>Asker Motoru Canlı API</h3>
            <p>Durum: {askerMotoruLiveStatus.health} / Bağlı: {askerMotoruLiveStatus.connected ? "evet" : "hayır"}</p>
            {#if askerMotoruLiveStatus.last_error}
              <pre>{askerMotoruLiveStatus.last_error}</pre>
            {/if}
          </div>
        {/if}
        {#if askerMotoruStatus}
          <div class="asker-bridge-panel">
            <h3>Asker Motoru Durum Köprüsü</h3>
            <p>DB boyutu: {(dbSizeBytes / (1024 * 1024)).toFixed(2)} MB</p>
            {#each askerMotoruStatus.root_sources || [] as root}
              <div class="asker-file" class:missing={root.health !== "available"}>
                <strong>{root.kind}: {root.source_path || "bağlı değil"}</strong>
                <span>{root.source_kind} / {root.health}</span>
                <pre>{root.error || "Kaynak erişilebilir."}</pre>
              </div>
            {/each}
            {#each askerMotoruStatus.files as file}
              <div class="asker-file" class:missing={!file.exists}>
                <strong>{file.path}</strong>
                <span>{file.source_kind} / {file.health}</span>
                <pre>{file.preview}</pre>
              </div>
            {/each}
          </div>
        {/if}
      {:else if activeSection === 'skills'}
        <SkillLibraryExplorer />
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

  .critical-alarm-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    width: fit-content;
    min-height: 26px;
    padding: 6px 12px;
    border-radius: 999px;
    border: 1px solid #2d2d31;
    background: #1a1a1c;
    color: #d5d5db;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.4px;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  .critical-alarm-indicator.active {
    color: #ffd4d4;
    border-color: rgba(244, 71, 71, 0.9);
    background: rgba(244, 71, 71, 0.18);
  }

  .critical-alarm-indicator.idle {
    color: #b5b5bd;
    border-color: #2d2d31;
    background: #151517;
  }

  .critical-alarm-indicator.pulsing {
    animation: criticalIndicatorPulse 1.2s ease-in-out infinite;
  }

  .indicator-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #3dbf66;
    box-shadow: 0 0 8px rgba(61, 191, 102, 0.7);
    flex-shrink: 0;
  }

  .critical-alarm-indicator.active .indicator-dot {
    background: #ff4747;
    box-shadow: 0 0 12px rgba(255, 71, 71, 0.9);
  }

  @keyframes criticalIndicatorPulse {
    0% {
      box-shadow: 0 0 0px rgba(255, 71, 71, 0.0);
    }
    100% {
      box-shadow: 0 0 16px rgba(255, 71, 71, 0.6);
    }
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

  .operation-audit-mini-panel {
    margin-top: 10px;
    padding: 10px 12px;
    border: 1px solid #2c2c31;
    border-radius: 8px;
    background: #131316;
    color: #cfd3dc;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 11px;
  }

  .operation-audit-title {
    color: #8fdaff;
    font-weight: 800;
    letter-spacing: 0.5px;
    font-size: 11px;
  }

  .operation-audit-empty {
    color: #8f8f98;
    font-size: 10px;
  }

  .operation-audit-scroll {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 170px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .operation-audit-row {
    display: grid;
    grid-template-columns: 150px 130px 1fr 1fr 60px;
    gap: 8px;
    align-items: start;
    font-size: 10px;
    color: #dee3ec;
  }

  .operation-audit-row span:first-child {
    color: #a0acbe;
    font-family: ui-monospace, monospace;
  }

  .operation-audit-row span:nth-child(2) {
    color: #ffe78a;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .operation-audit-row span:nth-child(4) {
    color: #f5f7fb;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .operation-audit-row span:nth-child(5) {
    font-weight: 800;
    text-align: right;
  }

  .operation-audit-row .pass {
    color: #7ee79c;
  }

  .operation-audit-row .warn {
    color: #ffd27a;
  }

  .operation-audit-row .fail {
    color: #ff9aa6;
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
  .alarm-code-panel,
  .asker-live-panel,
  .asker-bridge-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 6px;
    margin-bottom: 16px;
    color: #f4f4f5;
  }
  .alarm-code-item {
    padding: 8px;
    margin-top: 8px;
    border: 1px solid #5a2020;
    border-radius: 6px;
    background: #1a1010;
  }
  .alarm-code-item strong {
    color: #ff8a8a;
    margin-right: 8px;
  }
  .asker-live-panel {
    border-color: #2f4a66;
    background: #101820;
  }
  .asker-file {
    padding: 10px;
    margin-top: 8px;
    border: 1px solid #2d2d31;
    border-radius: 6px;
    background: #111113;
  }
  .asker-file.missing {
    border-color: rgba(248, 193, 74, 0.35);
  }
  .asker-file strong,
  .asker-file span {
    display: block;
    color: #dfe4ec;
    overflow-wrap: anywhere;
  }
  .asker-file span {
    width: fit-content;
    margin-top: 4px;
    padding: 2px 6px;
    border: 1px solid #3b3b40;
    border-radius: 999px;
    color: #9fd3ff;
    font-size: 10px;
    font-weight: 800;
  }
  .asker-file pre {
    margin: 8px 0 0;
    color: #b8b8bf;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: 12px;
  }
  .progress-bar-container { display: flex; align-items: center; justify-content: center; padding: 12px 20px; background: #0c0c0d; border-bottom: 1px solid #1f1f21; }
  .progress-step { font-size: 11px; font-weight: bold; color: #555; padding: 4px 10px; border-radius: 12px; background: #18181a; border: 1px solid #2d2d31; }
  .progress-step.active { color: #15110a; background: #f59e0b; border-color: #f59e0b; }
  .progress-step.done { color: #47d18c; border-color: #47d18c; }
  .progress-line { flex: 1; height: 2px; background: #2d2d31; margin: 0 10px; max-width: 50px; }
</style>

