export type VoicePersona = {
  lang: string;
  rate: number;
  pitch: number;
  volume: number;
  preferFemaleVoice: boolean;
  voiceHints?: string[];
};

const defaultPersona: VoicePersona = {
  lang: "tr-TR",
  rate: 0.92,
  pitch: 1.05,
  volume: 1,
  preferFemaleVoice: true,
};

let persona: VoicePersona = { ...defaultPersona };
const queue: { text: string; key: string }[] = [];
let speaking = false;
let lastKey = "";
let voicesHydrated = false;
let operatorVoiceBootstrapped = false;
let lastSpeakError: string | null = null;

export function setVoicePersona(next: Partial<VoicePersona>) {
  persona = { ...persona, ...next };
}

export function getVoicePersona(): VoicePersona {
  return { ...persona };
}

export function isOperatorVoiceBootstrapped(): boolean {
  return operatorVoiceBootstrapped;
}

export function getLastSpeakError(): string | null {
  return lastSpeakError;
}

export function hydrateSpeechVoices(): Promise<SpeechSynthesisVoice[]> {
  return new Promise((resolve) => {
    if (typeof window === "undefined" || !("speechSynthesis" in window)) {
      resolve([]);
      return;
    }
    const synth = window.speechSynthesis;
    const finish = () => {
      const voices = synth.getVoices();
      if (voices.length > 0) voicesHydrated = true;
      resolve(voices);
    };
    finish();
    if (!voicesHydrated) {
      synth.addEventListener("voiceschanged", () => finish(), { once: true });
      window.setTimeout(finish, 1000);
    }
  });
}

function isBootstrapSpeechNoise(error: string): boolean {
  return /interrupted|canceled|cancelled/i.test(error);
}

/** Tarayıcı/Tauri: ilk ses kullanıcı tıklaması gerektirir. */
export async function bootstrapOperatorVoice(testPhrase?: string): Promise<{
  ok: boolean;
  voiceName?: string;
  error?: string;
}> {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) {
    return { ok: false, error: "Bu ortamda ses sentezi (speechSynthesis) yok." };
  }

  await hydrateSpeechVoices();
  const synth = window.speechSynthesis;
  try {
    synth.cancel();
    synth.resume();
  } catch {
    /* ignore */
  }

  const voice = pickTurkishVoice(synth);
  const phrase =
    testPhrase || "Yarbay Emel Hanım görevde. Ses hattı açıldı. Komutanım, hazırım.";

  return new Promise((resolve) => {
    let settled = false;
    const finish = (result: { ok: boolean; voiceName?: string; error?: string }) => {
      if (settled) return;
      settled = true;
      if (result.ok) {
        operatorVoiceBootstrapped = true;
        lastSpeakError = null;
      } else {
        operatorVoiceBootstrapped = false;
      }
      resolve(result);
    };

    const utterance = new SpeechSynthesisUtterance(phrase);
    utterance.lang = persona.lang;
    utterance.rate = persona.rate;
    utterance.pitch = persona.pitch;
    utterance.volume = persona.volume;
    if (voice) utterance.voice = voice;

    utterance.onstart = () => {
      finish({ ok: true, voiceName: voice?.name || "varsayılan Türkçe" });
    };
    utterance.onerror = (event) => {
      const err = String((event as SpeechSynthesisErrorEvent)?.error || "speech-error");
      lastSpeakError = err;
      if (isBootstrapSpeechNoise(err)) {
        finish({ ok: true, voiceName: voice?.name || "varsayılan Türkçe" });
        return;
      }
      finish({ ok: false, error: `Ses hatası: ${err}` });
    };

    synth.speak(utterance);

    window.setTimeout(() => {
      try {
        synth.resume();
      } catch {
        /* ignore */
      }
      if (settled) return;
      if (synth.speaking || synth.pending) {
        finish({ ok: true, voiceName: voice?.name || "varsayılan Türkçe" });
        return;
      }
      finish({
        ok: false,
        error:
          "Ses başlamadı. Windows: Ayarlar → Zaman ve dil → Konuşma → Türkçe ses ekleyin (Microsoft Zira).",
      });
    }, 2500);
  });
}

function pickTurkishVoice(synth: SpeechSynthesis): SpeechSynthesisVoice | undefined {
  const voices = synth.getVoices();
  const turkish = voices.filter((voice) => voice.lang.toLowerCase().startsWith("tr"));
  if (!turkish.length) {
    const loose = voices.filter((voice) => /tr/i.test(voice.lang));
    if (!loose.length) return undefined;
    return loose[0];
  }
  if (persona.preferFemaleVoice) {
    const hints = persona.voiceHints?.length
      ? persona.voiceHints
      : ["female", "kadın", "woman", "zira", "yelda", "ayşe"];
    const pattern = new RegExp(hints.join("|"), "i");
    const female = turkish.find((voice) => pattern.test(`${voice.name} ${voice.voiceURI}`));
    if (female) return female;
  }
  return turkish[0];
}

function processQueue() {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) return;
  const synth = window.speechSynthesis;
  if (!queue.length) {
    speaking = false;
    return;
  }
  speaking = true;
  const current = queue.shift();
  if (!current) {
    speaking = false;
    return;
  }

  try {
    synth.resume();
  } catch {
    /* ignore */
  }

  const utterance = new SpeechSynthesisUtterance(current.text);
  const voice = pickTurkishVoice(synth);
  utterance.lang = persona.lang;
  utterance.rate = persona.rate;
  utterance.pitch = persona.pitch;
  utterance.volume = persona.volume;
  if (voice) utterance.voice = voice;

  utterance.onstart = () => {
    lastSpeakError = null;
  };
  utterance.onend = () => processQueue();
  utterance.onerror = (event) => {
    lastSpeakError = String((event as SpeechSynthesisErrorEvent)?.error || "speech-error");
    processQueue();
  };

  synth.speak(utterance);

  // Chrome/WebView2: konuşma takılırsa resume dene
  window.setTimeout(() => {
    if (speaking && synth.speaking) {
      try {
        synth.resume();
      } catch {
        /* ignore */
      }
    }
  }, 250);
}

export function speakText(text: string, key = text, force = true) {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) return false;
  if (!text?.trim()) return false;

  const isOperatorSpeech =
    key.startsWith("emel") || key === "voice-enabled" || key.startsWith("emel-user");

  if (!operatorVoiceBootstrapped && !force && !key.startsWith("critical") && !key.startsWith("alarm")) {
    if (isOperatorSpeech) {
      lastSpeakError = "Önce 'Emel'i Başlat' düğmesine tıklayın.";
      return false;
    }
  }

  if (!force && key === lastKey) return false;
  lastKey = key;

  if (force || key.startsWith("critical") || key.startsWith("alarm") || isOperatorSpeech) {
    queue.length = 0;
    window.speechSynthesis.cancel();
  }

  void hydrateSpeechVoices().then(() => {
    queue.push({ text, key });
    if (!speaking) processQueue();
  });

  return true;
}

export function stopSpeech() {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) return;
  queue.length = 0;
  speaking = false;
  window.speechSynthesis.cancel();
}

export function formatAlarmSpeech(code: string, title: string, message: string) {
  const codeSpeech = code.split("").join(" ");
  return `Alarm kodu ${codeSpeech}. ${title}. ${message}`;
}

type SpeechRecognitionCtor = new () => {
  lang: string;
  interimResults: boolean;
  continuous: boolean;
  onresult: ((event: { results: { [index: number]: { [index: number]: { transcript: string } } } }) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
};

export function startVoiceCommand(
  onResult: (transcript: string) => void,
  onError?: (message: string) => void,
) {
  if (typeof window === "undefined") return () => undefined;
  const ctor = (window as Window & { SpeechRecognition?: SpeechRecognitionCtor; webkitSpeechRecognition?: SpeechRecognitionCtor }).SpeechRecognition
    || (window as Window & { webkitSpeechRecognition?: SpeechRecognitionCtor }).webkitSpeechRecognition;
  if (!ctor) {
    onError?.("Sesli komut bu ortamda desteklenmiyor.");
    return () => undefined;
  }
  const recognition = new ctor();
  recognition.lang = "tr-TR";
  recognition.interimResults = false;
  recognition.continuous = false;
  recognition.onresult = (event) => {
    const transcript = event.results[0]?.[0]?.transcript?.trim();
    if (transcript) onResult(transcript);
  };
  recognition.onerror = (event) => onError?.(event.error);
  recognition.start();
  return () => recognition.stop();
}

if (typeof window !== "undefined" && "speechSynthesis" in window) {
  void hydrateSpeechVoices();
}
