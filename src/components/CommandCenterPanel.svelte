<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { startVoiceCommand, speakText } from "../lib/voiceService";
  import { isBrowserPreview } from "../lib/runtime";

  let {
    onCommandSubmitted,
    liveFeed = [],
  } = $props<{
    onCommandSubmitted?: (result: any) => void;
    liveFeed?: any[];
  }>();

  let sentence = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let stopListening: (() => void) | undefined;

  async function submitSentence() {
    const trimmed = sentence.trim();
    if (!trimmed || submitting) return;
    submitting = true;
    error = null;
    try {
      if (isBrowserPreview()) {
        const mock = {
          task: { id: `preview-${Date.now()}`, title: trimmed, user_request: trimmed },
          platforms: ["burhan_command", "codex", "open_agent_manager"],
          burhan_message: "Albay Burhan emri aldı.",
        };
        speakText(`Emir alındı. ${trimmed}`, `command:${mock.task.id}`, true);
        onCommandSubmitted?.(mock);
        sentence = "";
        return;
      }
      const result = await invoke("submit_command_sentence_cmd", {
        sentence: trimmed,
        operatorId: "kurucu",
      });
      speakText(`Emir alındı. ${trimmed}`, `command:${(result as any).task.id}`, true);
      onCommandSubmitted?.(result);
      sentence = "";
    } catch (err) {
      error = String(err);
      speakText(`Komut iletilemedi. ${error}`, `command-error:${Date.now()}`, true);
    } finally {
      submitting = false;
    }
  }

  function handleVoiceCommand() {
    stopListening?.();
    stopListening = startVoiceCommand(
      (transcript) => {
        sentence = transcript;
        submitSentence();
      },
      (message) => {
        error = message;
      },
    );
  }
</script>

<div class="command-center-panel">
  <header>
    <h3>Komuta Paneli</h3>
    <p>Tek cümle emir. Panel otomatik yürütür.</p>
  </header>
  <div class="command-input-row">
    <input
      bind:value={sentence}
      placeholder="Emrinizi tek cümleyle yazın..."
      onkeydown={(event) => event.key === "Enter" && submitSentence()}
      disabled={submitting}
    />
    <button onclick={submitSentence} disabled={submitting || !sentence.trim()}>
      {submitting ? "İletiliyor..." : "Emir Ver"}
    </button>
    <button class="voice" onclick={handleVoiceCommand} disabled={submitting}>Sesli Emir</button>
  </div>
  {#if error}
    <div class="error">{error}</div>
  {/if}
  <div class="live-feed">
    <h4>Canlı Komut Akışı</h4>
    {#if liveFeed.length === 0}
      <p class="empty">Henüz komut yok.</p>
    {:else}
      <ul>
        {#each liveFeed.slice(0, 12) as item}
          <li>
            <span class="time">{item.created_at || item.timestamp}</span>
            <strong>{item.source}</strong>
            <span>{item.message}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .command-center-panel {
    border: 1px solid #2f2f35;
    background: #141418;
    border-radius: 8px;
    padding: 14px;
    color: #f3f3f6;
  }
  header h3 { margin: 0 0 4px; }
  header p { margin: 0 0 12px; color: #9a9aa3; font-size: 0.85rem; }
  .command-input-row { display: flex; gap: 8px; flex-wrap: wrap; }
  input {
    flex: 1;
    min-width: 220px;
    background: #0d0d10;
    border: 1px solid #3a3a42;
    color: #fff;
    border-radius: 6px;
    padding: 10px 12px;
  }
  button {
    background: #2f5cff;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 10px 14px;
    font-weight: 700;
    cursor: pointer;
  }
  button.voice { background: #1f7a57; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .error { margin-top: 8px; color: #ff7f7f; font-size: 0.85rem; }
  .live-feed { margin-top: 14px; }
  .live-feed h4 { margin: 0 0 8px; font-size: 0.9rem; }
  ul { list-style: none; margin: 0; padding: 0; max-height: 180px; overflow: auto; }
  li {
    display: grid;
    grid-template-columns: 120px 120px 1fr;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid #26262c;
    font-size: 0.8rem;
  }
  .time { color: #8d8d97; }
  .empty { color: #888; font-size: 0.85rem; }
</style>
