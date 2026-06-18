import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type MockUtterance = {
  text: string;
  lang: string;
  rate: number;
  pitch: number;
  volume: number;
  voice?: SpeechSynthesisVoice;
  onstart: (() => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
};

function createSpeechMock(options: { speak?: "start" | "fail" | "hang" }) {
  const utterances: MockUtterance[] = [];
  const synth = {
    speaking: false,
    pending: false,
    getVoices: () =>
      [{ name: "Microsoft Zira", lang: "tr-TR", voiceURI: "zira" }] as SpeechSynthesisVoice[],
    cancel: vi.fn(() => {
      synth.speaking = false;
      synth.pending = false;
    }),
    resume: vi.fn(),
    speak: vi.fn((utterance: MockUtterance) => {
      utterances.push(utterance);
      if (options.speak === "hang") {
        return;
      }
      if (options.speak === "fail") {
        queueMicrotask(() => utterance.onerror?.({ error: "network" }));
        return;
      }
      queueMicrotask(() => {
        synth.speaking = true;
        utterance.onstart?.();
        synth.speaking = false;
        utterance.onend?.();
      });
    }),
  };
  return { synth, utterances };
}

describe("bootstrapOperatorVoice", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
    class MockSpeechSynthesisUtterance {
      text: string;
      lang = "";
      rate = 1;
      pitch = 1;
      volume = 1;
      voice?: SpeechSynthesisVoice;
      onstart: (() => void) | null = null;
      onerror: ((event: { error: string }) => void) | null = null;
      onend: (() => void) | null = null;
      constructor(text: string) {
        this.text = text;
      }
    }
    vi.stubGlobal("SpeechSynthesisUtterance", MockSpeechSynthesisUtterance);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    Reflect.deleteProperty(window, "speechSynthesis");
  });

  it("returns ok when speech starts", async () => {
    const { synth } = createSpeechMock({ speak: "start" });
    Object.defineProperty(window, "speechSynthesis", { value: synth, configurable: true });

    const { bootstrapOperatorVoice, isOperatorVoiceBootstrapped } = await import("./voiceService");
    const result = await bootstrapOperatorVoice("test");
    expect(result.ok).toBe(true);
    expect(result.voiceName).toContain("Zira");
    expect(isOperatorVoiceBootstrapped()).toBe(true);
  });

  it("returns error when speech fails", async () => {
    const { synth } = createSpeechMock({ speak: "fail" });
    Object.defineProperty(window, "speechSynthesis", { value: synth, configurable: true });

    const { bootstrapOperatorVoice, isOperatorVoiceBootstrapped } = await import("./voiceService");
    const result = await bootstrapOperatorVoice("test");
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(/Ses hatası/);
    expect(isOperatorVoiceBootstrapped()).toBe(false);
  });

  it("times out when speech never starts", async () => {
    const { synth } = createSpeechMock({ speak: "hang" });
    Object.defineProperty(window, "speechSynthesis", { value: synth, configurable: true });

    const { bootstrapOperatorVoice } = await import("./voiceService");
    const pending = bootstrapOperatorVoice("test");
    await vi.advanceTimersByTimeAsync(2600);
    const result = await pending;
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(/Ses başlamadı/);
  });
});
