const SILENCE_UNTIL_KEY = "localControlPanel.alarmSilenceUntil";
const CIRCUIT_OPEN_KEY = "localControlPanel.alarmCircuitOpen";
const ALARM_BURST_KEY = "localControlPanel.alarmBurstTimestamps";

/** Kullanıcı veya devre kesici sesi kapattıysa true. */
export function isAlarmSilenced(): boolean {
  if (typeof window === "undefined") return false;
  try {
    if (localStorage.getItem(CIRCUIT_OPEN_KEY) === "1") return true;
    const until = Number(localStorage.getItem(SILENCE_UNTIL_KEY) || 0);
    return Date.now() < until;
  } catch {
    return false;
  }
}

/** Tüm alarm seslerini belirtilen süre kapatır (ms). */
export function silenceAlarmsForMs(ms: number) {
  if (typeof window === "undefined") return;
  localStorage.setItem(SILENCE_UNTIL_KEY, String(Date.now() + ms));
}

/** Acil durum: ses tamamen kapalı, devre kesici açık. */
export function activateAlarmCircuitBreaker(durationMs = 30 * 60 * 1000) {
  if (typeof window === "undefined") return;
  localStorage.setItem(CIRCUIT_OPEN_KEY, "1");
  silenceAlarmsForMs(durationMs);
}

export function resetAlarmCircuit() {
  if (typeof window === "undefined") return;
  localStorage.removeItem(CIRCUIT_OPEN_KEY);
  localStorage.removeItem(SILENCE_UNTIL_KEY);
  localStorage.removeItem(ALARM_BURST_KEY);
}

/** 60 sn içinde 3+ alarm → otomatik devre kesici. */
export function recordAlarmBurstAndTripBreaker(): boolean {
  if (typeof window === "undefined") return false;
  const now = Date.now();
  let stamps: number[] = [];
  try {
    const raw = localStorage.getItem(ALARM_BURST_KEY);
    stamps = raw ? (JSON.parse(raw) as number[]) : [];
  } catch {
    stamps = [];
  }
  stamps = stamps.filter((t) => now - t < 60_000);
  stamps.push(now);
  localStorage.setItem(ALARM_BURST_KEY, JSON.stringify(stamps));
  if (stamps.length >= 3) {
    activateAlarmCircuitBreaker();
    return true;
  }
  return false;
}

export function remainingSilenceLabel(): string | null {
  if (!isAlarmSilenced()) return null;
  try {
    const until = Number(localStorage.getItem(SILENCE_UNTIL_KEY) || 0);
    if (until > Date.now()) {
      const min = Math.ceil((until - Date.now()) / 60_000);
      return `Ses kapalı (~${min} dk)`;
    }
    if (localStorage.getItem(CIRCUIT_OPEN_KEY) === "1") {
      return "Alarm devre kesici açık — ses kapalı";
    }
  } catch {
    /* ignore */
  }
  return "Ses kapalı";
}
