<script lang="ts">
  import { invokePanel } from "../lib/tauriInvoke";
  import { startVoiceCommand, speakText } from "../lib/voiceService";
  import { isBrowserPreview } from "../lib/runtime";
  import {
    PANEL_THEMES,
    type PanelTheme,
    getStoredPanelTheme,
    storePanelTheme,
    type CommandFlowStage,
  } from "../lib/commandFlow";

  let {
    onCommandSubmitted,
    liveFeed = [],
    flowStage = "awaiting_task",
  } = $props<{
    onCommandSubmitted?: (result: any) => void;
    liveFeed?: any[];
    flowStage?: CommandFlowStage;
  }>();

  let sentence = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let inputMode = $state<"written" | "voice">("written");
  let theme = $state<PanelTheme>(getStoredPanelTheme());
  let stopListening: (() => void) | undefined;
  let isListening = $state(false);

  const themeConfig = $derived(PANEL_THEMES[theme]);

  const flowSteps = $derived([
    { id: 1, label: "Görev ver", active: flowStage === "awaiting_task", done: flowStage !== "awaiting_task" },
    { id: 2, label: "Albay Burhan'a ilet", active: flowStage === "assigned_to_burhan", done: ["operations_active", "completed"].includes(flowStage) },
    { id: 3, label: "Operasyon başlat", active: flowStage === "operations_active", done: flowStage === "completed" },
  ]);

  function selectTheme(next: PanelTheme) {
    theme = next;
    storePanelTheme(next);
  }

  async function submitSentence() {
    const trimmed = sentence.trim();
    if (!trimmed || submitting) return;
    submitting = true;
    error = null;
    try {
      if (isBrowserPreview()) {
        const mock = {
          task: { id: `preview-${Date.now()}`, title: trimmed, user_request: `[KomutMerkezi:kurucu] ${trimmed}` },
          platforms: ["burhan_command", "codex", "open_agent_manager"],
          burhan_message: "Albay Burhan görevi aldı.",
        };
        speakText(`Görev atandı. ${trimmed}`, `command:${mock.task.id}`, true);
        onCommandSubmitted?.(mock);
        sentence = "";
        return;
      }
      const result = await invokePanel("submit_command_sentence_cmd", {
        sentence: trimmed,
        operatorId: "kurucu",
      });
      speakText(`Görev atandı. ${trimmed}`, `command:${(result as any).task.id}`, true);
      onCommandSubmitted?.(result);
      sentence = "";
    } catch (err) {
      error = String(err);
      speakText(`Görev iletilemedi.`, `command-error:${Date.now()}`, true);
    } finally {
      submitting = false;
      isListening = false;
      stopListening?.();
    }
  }

  function handleVoiceCommand() {
    inputMode = "voice";
    isListening = true;
    error = null;
    stopListening?.();
    stopListening = startVoiceCommand(
      (transcript) => {
        sentence = transcript;
        isListening = false;
        submitSentence();
      },
      (message) => {
        error = message;
        isListening = false;
      },
    );
  }
</script>

<section
  class="founder-panel theme-{theme}"
  style={`--accent:${themeConfig.accent};--surface:${themeConfig.surface};--text:${themeConfig.text}`}
>
  <header class="panel-head">
    <div>
      <span class="panel-badge">Panel 1 · Kurucu</span>
      <h2>Albay Burhan'a Görev Atama</h2>
      <p>Operasyonlar yalnızca buradan verilen görevle başlar. Yazılı veya sesli atayın.</p>
    </div>
    <div class="theme-picker" aria-label="Panel tasarım alternatifleri">
      <span class="theme-label">Tasarım</span>
      {#each Object.entries(PANEL_THEMES) as [key, cfg]}
        <button
          class="theme-btn"
          class:selected={theme === key}
          title={cfg.description}
          onclick={() => selectTheme(key as PanelTheme)}
        >
          {cfg.label}
        </button>
      {/each}
    </div>
  </header>

  <ol class="flow-steps">
    {#each flowSteps as step}
      <li class:done={step.done} class:active={step.active}>
        <span class="step-no">{step.id}</span>
        <span>{step.label}</span>
      </li>
    {/each}
  </ol>

  <div class="mode-switch">
    <button class:selected={inputMode === "written"} onclick={() => (inputMode = "written")}>Yazılı Görev</button>
    <button class:selected={inputMode === "voice"} onclick={handleVoiceCommand}>Sesli Görev</button>
  </div>

  <div class="assignment-box">
    <label for="founder-task-input">Görev cümlesi</label>
    <textarea
      id="founder-task-input"
      bind:value={sentence}
      rows="3"
      placeholder="Örnek: Sistem bağlantılarını kontrol et ve raporla."
      disabled={submitting}
    ></textarea>
    <div class="actions">
      <button class="primary" onclick={submitSentence} disabled={submitting || !sentence.trim()}>
        {submitting ? "Albay Burhan'a iletiliyor..." : "Albay Burhan'a Görev Ata"}
      </button>
      {#if isListening}
        <span class="listening">Dinleniyor...</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error" role="alert">{error}</div>
  {/if}

  {#if flowStage === "awaiting_task"}
    <div class="flow-lock">Diğer paneller kilitli. Önce görev atayın.</div>
  {/if}

  <div class="live-feed">
    <h3>Son Görevler</h3>
    {#if liveFeed.length === 0}
      <p class="empty">Henüz görev atanmadı.</p>
    {:else}
      <ul>
        {#each liveFeed.slice(0, 8) as item}
          <li>
            <time>{item.created_at || item.timestamp}</time>
            <span>{item.message}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .founder-panel {
    border: 1px solid color-mix(in srgb, var(--accent) 35%, #d9e2e2);
    background: var(--surface);
    color: var(--text);
    border-radius: 14px;
    padding: 18px 20px;
    box-shadow: 0 8px 24px rgba(20, 30, 30, 0.08);
  }
  .theme-disiplin {
    border-color: #4a3828;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  }
  .panel-head {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 14px;
  }
  .panel-badge {
    display: inline-block;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--accent);
    margin-bottom: 6px;
  }
  h2 { margin: 0 0 6px; font-size: 1.35rem; font-weight: 700; }
  .panel-head p { margin: 0; opacity: 0.82; font-size: 0.92rem; max-width: 52ch; }
  .theme-picker { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
  .theme-label { font-size: 0.75rem; opacity: 0.7; margin-right: 4px; }
  .theme-btn {
    border: 1px solid color-mix(in srgb, var(--accent) 25%, #ccc);
    background: transparent;
    color: inherit;
    border-radius: 999px;
    padding: 5px 10px;
    font-size: 0.72rem;
    cursor: pointer;
  }
  .theme-btn.selected {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }
  .flow-steps {
    list-style: none;
    display: flex;
    gap: 10px;
    padding: 0;
    margin: 0 0 14px;
    flex-wrap: wrap;
  }
  .flow-steps li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--accent) 8%, #fff);
    font-size: 0.82rem;
    opacity: 0.65;
  }
  .flow-steps li.active { opacity: 1; border: 1px solid color-mix(in srgb, var(--accent) 40%, #fff); }
  .flow-steps li.done { opacity: 1; background: color-mix(in srgb, var(--accent) 18%, #fff); }
  .step-no {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
  }
  .mode-switch { display: flex; gap: 8px; margin-bottom: 12px; }
  .mode-switch button {
    border: 1px solid color-mix(in srgb, var(--accent) 30%, #ccc);
    background: transparent;
    color: inherit;
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
  }
  .mode-switch button.selected { background: color-mix(in srgb, var(--accent) 15%, #fff); border-color: var(--accent); }
  .assignment-box label { display: block; margin-bottom: 6px; font-size: 0.85rem; font-weight: 600; }
  textarea {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid color-mix(in srgb, var(--accent) 25%, #ccc);
    border-radius: 10px;
    padding: 12px;
    background: color-mix(in srgb, var(--surface) 90%, #fff);
    color: inherit;
    resize: vertical;
    font: inherit;
    line-height: 1.45;
  }
  .actions { display: flex; align-items: center; gap: 10px; margin-top: 10px; flex-wrap: wrap; }
  .primary {
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 10px;
    padding: 11px 16px;
    font-weight: 700;
    cursor: pointer;
  }
  .primary:disabled { opacity: 0.55; cursor: not-allowed; }
  .listening { font-size: 0.82rem; color: var(--accent); font-weight: 600; }
  .error {
    margin-top: 10px;
    padding: 10px;
    border-radius: 8px;
    background: #fdecec;
    color: #8a1f1f;
    font-size: 0.85rem;
  }
  .theme-disiplin .error { background: #3a1818; color: #ffb4b4; }
  .flow-lock {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--accent) 10%, #fff);
    font-size: 0.84rem;
  }
  .live-feed { margin-top: 14px; }
  .live-feed h3 { margin: 0 0 8px; font-size: 0.92rem; }
  .live-feed ul { list-style: none; margin: 0; padding: 0; max-height: 140px; overflow: auto; }
  .live-feed li {
    display: grid;
    grid-template-columns: 130px 1fr;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid color-mix(in srgb, var(--accent) 12%, #ddd);
    font-size: 0.8rem;
  }
  time { opacity: 0.65; }
  .empty { opacity: 0.7; font-size: 0.85rem; }
</style>
