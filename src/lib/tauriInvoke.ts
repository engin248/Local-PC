import { invoke } from "@tauri-apps/api/core";

/** Tauri Rust komutları snake_case bekler; UI camelCase kullanır. */
export function normalizeTauriArgs(
  cmd: string,
  args?: Record<string, unknown>,
): Record<string, unknown> | undefined {
  if (!args) return args;

  const out: Record<string, unknown> = { ...args };

  if (out.taskId !== undefined && out.task_id === undefined) {
    out.task_id = out.taskId;
    delete out.taskId;
  }
  if (out.operatorId !== undefined && out.operator_id === undefined) {
    out.operator_id = out.operatorId;
    delete out.operatorId;
  }
  if (out.approvalId !== undefined && out.approval_id === undefined) {
    out.approval_id = out.approvalId;
    delete out.approvalId;
  }
  if (out.alarmId !== undefined && out.alarm_id === undefined) {
    out.alarm_id = out.alarmId;
    delete out.alarmId;
  }
  if (out.moduleId !== undefined && out.module_id === undefined) {
    out.module_id = out.moduleId;
    delete out.moduleId;
  }
  if (out.runtimeAlarms !== undefined && out.runtime_alarms === undefined) {
    out.runtime_alarms = out.runtimeAlarms;
    delete out.runtimeAlarms;
  }
  if (out.writeAudit !== undefined && out.write_audit === undefined) {
    out.write_audit = out.writeAudit;
    delete out.writeAudit;
  }

  if (cmd === "append_operation_audit_cmd" && out.input === undefined && out.actor !== undefined) {
    return { input: out };
  }

  return out;
}

export async function invokePanel<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const normalized = normalizeTauriArgs(cmd, args);
  return invoke<T>(cmd, normalized);
}
