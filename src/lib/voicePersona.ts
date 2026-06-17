import { invokePanel } from "./tauriInvoke";
import { setVoicePersona, type VoicePersona } from "./voiceService";

export type VoicePersonaConfig = {
  schema_version: number;
  locale: string;
  persona: {
    name: string;
    rank: string;
    role: string;
    tone?: string;
  };
  synthesis: {
    lang: string;
    rate: number;
    pitch: number;
    volume: number;
    prefer_female_voice: boolean;
    voice_hints?: string[];
  };
  templates: Record<string, string>;
};

const fallbackConfig: VoicePersonaConfig = {
  schema_version: 2,
  locale: "tr-TR",
  persona: {
    name: "Yarbay Emel Hanım",
    rank: "Yarbay",
    role: "Komuta Operatörü",
  },
  synthesis: {
    lang: "tr-TR",
    rate: 0.88,
    pitch: 1.08,
    volume: 1,
    prefer_female_voice: true,
    voice_hints: ["zira", "yelda", "emel", "kadın", "female"],
  },
  templates: {
    greeting: "Yarbay Emel Hanım görevde. Mesajlarınızı sesli okuyacağım. Komutanım, hazırım.",
    command_accepted: "Emir alındı. {summary}",
    report_ready: "Rapor hazır. {summary}",
    burhan_dispatch: "Albay Burhan emir dağıttı. {summary}",
    assistant_message: "{summary}",
    chat_readout: "{summary}",
  },
};

let loadedConfig: VoicePersonaConfig = { ...fallbackConfig };
let voiceHints: string[] = [...(fallbackConfig.synthesis.voice_hints || [])];

export function getVoicePersonaConfig(): VoicePersonaConfig {
  return loadedConfig;
}

export function getOperatorName(): string {
  return loadedConfig.persona.name || "Yarbay Emel Hanım";
}

export function getVoiceHints(): string[] {
  return voiceHints;
}

export function formatVoiceTemplate(
  key: string,
  vars: Record<string, string> = {},
): string {
  const template = loadedConfig.templates[key] || "{summary}";
  return Object.entries(vars).reduce(
    (text, [name, value]) => text.replaceAll(`{${name}}`, value),
    template,
  );
}

export function applyVoicePersonaToService() {
  const s = loadedConfig.synthesis;
  const next: Partial<VoicePersona> = {
    lang: s.lang,
    rate: s.rate,
    pitch: s.pitch,
    volume: s.volume,
    preferFemaleVoice: s.prefer_female_voice,
    voiceHints: s.voice_hints,
  };
  setVoicePersona(next);
  voiceHints = s.voice_hints || voiceHints;
}

export async function loadVoicePersonaConfig(): Promise<VoicePersonaConfig> {
  try {
    const remote = await invokePanel<VoicePersonaConfig>("get_voice_persona_cmd");
    if (remote?.persona?.name) {
      loadedConfig = remote;
      applyVoicePersonaToService();
      return loadedConfig;
    }
  } catch {
  }

  try {
    const res = await fetch("/config/voice_persona.json");
    if (res.ok) {
      loadedConfig = (await res.json()) as VoicePersonaConfig;
      applyVoicePersonaToService();
      return loadedConfig;
    }
  } catch {
    /* preview modu */
  }

  loadedConfig = { ...fallbackConfig };
  applyVoicePersonaToService();
  return loadedConfig;
}
