/** Kritik alarmın tekrar tetiklenmesini engellemek için pencere (ms). */
export const ALARM_DEDUPE_MS = 20_000;

const speechNoisePatterns = [
  /interrupted/i,
  /canceled/i,
  /cancelled/i,
  /not-allowed/i,
];

/** Ses sentezi kesintileri kritik alarm sayılmaz. */
export function isSpeechSynthesisNoise(error: unknown): boolean {
  const text = String(error);
  return speechNoisePatterns.some((pattern) => pattern.test(text));
}

/** Salt okunur / rutin yükleme hataları siren tetiklemez. */
export function isOperationalReadError(source: string): boolean {
  return (
    source.includes("Görev detayları") ||
    source.includes("audit logu") ||
    source.includes("health-check") ||
    source.includes("Sistem sağlık") ||
    source.includes("Seslendirme motoru")
  );
}

export function shouldSuppressCriticalAlarm(source: string, err: unknown): boolean {
  if (isSpeechSynthesisNoise(err)) return true;
  if (source.includes("Seslendirme motoru") && isSpeechSynthesisNoise(err)) return true;
  return false;
}

/** Dekoratif görseller (logo) yüklenemezse siren tetiklenmez. */
export function isDecorativeResourceLoadFailure(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLImageElement)) return false;
  const src = target.src || "";
  return (
    src.includes("brain_logo") ||
    src.includes("tauri.svg") ||
    src.includes("favicon") ||
    target.classList.contains("brand-logo")
  );
}
