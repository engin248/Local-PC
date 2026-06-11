export type VoicePersona = {
  lang: string;
  rate: number;
  pitch: number;
  volume: number;
  preferFemaleVoice: boolean;
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

export function setVoicePersona(next: Partial<VoicePersona>) {
  persona = { ...persona, ...next };
}

function pickTurkishVoice(synth: SpeechSynthesis): SpeechSynthesisVoice | undefined {
  const voices = synth.getVoices();
  const turkish = voices.filter((voice) => voice.lang.toLowerCase().startsWith("tr"));
  if (!turkish.length) return undefined;
  if (persona.preferFemaleVoice) {
    const female = turkish.find((voice) => /female|kadın|woman|zira|yelda/i.test(`${voice.name} ${voice.voiceURI}`));
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
  const utterance = new SpeechSynthesisUtterance(current.text);
  const voice = pickTurkishVoice(synth);
  utterance.lang = persona.lang;
  utterance.rate = persona.rate;
  utterance.pitch = persona.pitch;
  utterance.volume = persona.volume;
  if (voice) utterance.voice = voice;
  utterance.onend = () => processQueue();
  utterance.onerror = () => processQueue();
  synth.speak(utterance);
}

export function speakText(text: string, key = text, force = true) {
  if (typeof window === "undefined" || !("speechSynthesis" in window)) return false;
  if (!force && key === lastKey) return false;
  lastKey = key;
  if (force || key.startsWith("critical") || key.startsWith("alarm")) {
    queue.length = 0;
    window.speechSynthesis.cancel();
  }
  queue.push({ text, key });
  if (!speaking) processQueue();
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
