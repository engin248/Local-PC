import { describe, expect, it } from "vitest";
import {
  isDecorativeResourceLoadFailure,
  isOperationalReadError,
  isSpeechSynthesisNoise,
  shouldSuppressCriticalAlarm,
} from "./alarmPolicy";

describe("alarmPolicy", () => {
  it("speech synthesis noise is suppressed", () => {
    expect(isSpeechSynthesisNoise("interrupted")).toBe(true);
    expect(isSpeechSynthesisNoise("canceled")).toBe(true);
    expect(isSpeechSynthesisNoise("network-failure")).toBe(false);
    expect(shouldSuppressCriticalAlarm("Kaynak", "interrupted")).toBe(true);
  });

  it("operational read errors skip siren", () => {
    expect(isOperationalReadError("Bağlantı health-check uyarısı")).toBe(true);
    expect(isOperationalReadError("Yürütme sırasında hata")).toBe(false);
  });

  it("decorative logo images do not trigger critical alarm path", () => {
    const img = document.createElement("img");
    img.src = "http://127.0.0.1:200/brain_logo.png";
    img.className = "brand-logo";
    expect(isDecorativeResourceLoadFailure(img)).toBe(true);

    const brain = document.createElement("img");
    brain.src = "/brain_logo.png";
    expect(isDecorativeResourceLoadFailure(brain)).toBe(true);

    const other = document.createElement("img");
    other.src = "/api/chart.png";
    expect(isDecorativeResourceLoadFailure(other)).toBe(false);
  });
});
