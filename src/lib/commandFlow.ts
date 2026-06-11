export type CommandFlowStage =
  | "awaiting_task"
  | "assigned_to_burhan"
  | "operations_active"
  | "completed";

export type PanelTheme = "sakin" | "disiplin" | "minimal";

export const PANEL_THEMES: Record<
  PanelTheme,
  { label: string; description: string; accent: string; surface: string; text: string }
> = {
  sakin: {
    label: "Sakin Operasyon",
    description: "Yumuşak tonlar, düşük stres, net adımlar.",
    accent: "#5b8f8f",
    surface: "#f4f7f6",
    text: "#1e2a2a",
  },
  disiplin: {
    label: "Askeri Disiplin",
    description: "Koyu zemin, altın vurgu, komuta hissi.",
    accent: "#c9a24d",
    surface: "#1a1410",
    text: "#f6f0e8",
  },
  minimal: {
    label: "Minimal Net",
    description: "Az öğe, yüksek okunabilirlik, sade düzen.",
    accent: "#3d6fb8",
    surface: "#f8f9fb",
    text: "#1a1f2b",
  },
};

export function resolveFlowStage(task: any | null, burhanDispatched: boolean): CommandFlowStage {
  if (!task) return "awaiting_task";
  if (!isCommandCenterTask(task)) return "awaiting_task";
  if (task.status === "completed") return "completed";
  if (task.execution_status === "in_progress" || task.status === "in_progress") {
    return "operations_active";
  }
  if (burhanDispatched || task.current_gate) return "assigned_to_burhan";
  return "assigned_to_burhan";
}

export function isCommandCenterTask(task: any | null | undefined): boolean {
  return Boolean(task?.user_request?.includes("[KomutMerkezi:"));
}

export function getStoredPanelTheme(): PanelTheme {
  if (typeof localStorage === "undefined") return "sakin";
  const value = localStorage.getItem("commandPanelTheme");
  if (value === "disiplin" || value === "minimal" || value === "sakin") return value;
  return "sakin";
}

export function storePanelTheme(theme: PanelTheme) {
  localStorage.setItem("commandPanelTheme", theme);
}
