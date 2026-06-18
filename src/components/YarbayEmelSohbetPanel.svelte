<script lang="ts">
  import { getOperatorName } from "../lib/voicePersona";

  type EmelMessage = {
    id: string;
    text: string;
    at: string;
    source?: string;
  };

  let {
    messages = [],
    voiceRepliesEnabled = true,
    voiceBootstrapped = false,
    voiceStatus = null as string | null,
    onBootstrap,
    onSpeak,
    onAddMessage,
  } = $props<{
    messages?: EmelMessage[];
    voiceRepliesEnabled?: boolean;
    voiceBootstrapped?: boolean;
    voiceStatus?: string | null;
    onBootstrap?: () => void | Promise<void>;
    onSpeak?: (text: string, key?: string) => void;
    onAddMessage?: (text: string, source?: string) => void;
  }>();

  let draft = $state("");
  let autoRead = $state(true);
  const operatorName = getOperatorName();

  function readDraft() {
    const text = draft.trim();
    if (!text) return;
    onAddMessage?.(text, "Komutan");
    draft = "";
  }

  async function pasteAndRead() {
    try {
      const clip = await navigator.clipboard.readText();
      if (!clip?.trim()) return;
      onAddMessage?.(clip.trim(), "Pano");
    } catch {
      onAddMessage?.("Pano okunamadı. Metni kutuya yapıştırıp Oku düğmesine basın.", "Sistem");
    }
  }

  function repeatLast() {
    const last = messages[0];
    if (!last) return;
    onSpeak?.(last.text, `emel-repeat:${last.id}`);
  }
</script>

<section class="emel-panel" aria-label="Yarbay Emel Hanım sesli sohbet">
  <header>
    <span class="eyebrow">SESİLİ KOMUTA HATTI</span>
    <h2>{operatorName}</h2>
    <p>
      Göz bandajı ve erişilebilirlik modu. Tüm mesajlar büyük yazı ve sesli okunur.
      Albay Burhan komutayı yürütür; {operatorName} size seslendirir.
    </p>
  </header>

  {#if !voiceBootstrapped}
    <div class="bootstrap-box" role="alert">
      <p><strong>Ses gelmiyorsa:</strong> Windows ve Tauri ilk ses için bir tıklama ister.</p>
      <button type="button" class="btn bootstrap" onclick={() => onBootstrap?.()}>
        Emel'i Başlat — Ses Hattını Aç
      </button>
    </div>
  {:else}
    <p class="status-ok">Ses hattı açık. {voiceStatus || "Hazır."}</p>
    <p class="hint">Ses duymadıysanız: Windows ses seviyesi + Türkçe konuşma paketi (Zira).</p>
  {/if}

  {#if voiceStatus && !voiceBootstrapped}
    <p class="warn">{voiceStatus}</p>
  {/if}

  <div class="controls">
    <label class="toggle">
      <input type="checkbox" bind:checked={autoRead} />
      Yeni mesajları otomatik oku
    </label>
    <button type="button" class="btn primary" onclick={pasteAndRead}>Panodan Oku</button>
    <button type="button" class="btn" onclick={repeatLast} disabled={!messages.length}>Son Mesajı Tekrarla</button>
  </div>

  <div class="composer">
    <label for="emel-draft">Mesaj yazın veya yapıştırın</label>
    <textarea
      id="emel-draft"
      bind:value={draft}
      rows="4"
      placeholder="Buraya yazın veya yapıştırın. Oku düğmesiyle Yarbay Emel Hanım seslendirir."
    ></textarea>
    <button type="button" class="btn primary large" onclick={readDraft}>Oku</button>
  </div>

  <div class="feed" aria-live="polite">
    {#if !messages.length}
      <p class="empty">Henüz mesaj yok. İlk mesaj sesli okunacak.</p>
    {:else}
      {#each messages as msg (msg.id)}
        <article class="msg">
          <time>{msg.at}</time>
          {#if msg.source}
            <span class="source">{msg.source}</span>
          {/if}
          <p>{msg.text}</p>
          <button type="button" class="btn small" onclick={() => onSpeak?.(msg.text, `emel:${msg.id}`)}>
            Tekrar Oku
          </button>
        </article>
      {/each}
    {/if}
  </div>

  {#if !voiceRepliesEnabled}
    <p class="warn">Sesli cevap kapalı. Üst bardan "Yarbay Emel — Sesli Okuma" ile açın.</p>
  {/if}
</section>

<style>
  .emel-panel {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.25rem;
    background: #0f1419;
    border: 2px solid #3d7a46;
    border-radius: 12px;
    color: #e8f0e8;
  }

  .eyebrow {
    font-size: 0.75rem;
    letter-spacing: 0.12em;
    color: #7fdc8a;
  }

  h2 {
    margin: 0.25rem 0;
    font-size: 2rem;
    color: #fff;
  }

  header p {
    margin: 0;
    font-size: 1.15rem;
    line-height: 1.5;
    max-width: 52rem;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
  }

  .toggle {
    font-size: 1.1rem;
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .composer label {
    display: block;
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
  }

  textarea {
    width: 100%;
    font-size: 1.35rem;
    line-height: 1.5;
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid #4a6a4f;
    background: #1a2420;
    color: #fff;
    resize: vertical;
  }

  .btn {
    border: 1px solid #4a6a4f;
    background: #1e2e22;
    color: #e8f0e8;
    padding: 0.65rem 1rem;
    border-radius: 8px;
    font-size: 1.05rem;
    cursor: pointer;
  }

  .btn.primary {
    background: #2d6a3e;
    border-color: #4caf66;
  }

  .btn.large {
    margin-top: 0.75rem;
    font-size: 1.25rem;
    padding: 0.85rem 1.5rem;
  }

  .btn.small {
    font-size: 0.95rem;
    margin-top: 0.5rem;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .feed {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-height: 28rem;
    overflow-y: auto;
  }

  .msg {
    background: #162018;
    border: 1px solid #335a3a;
    border-radius: 10px;
    padding: 1rem 1.1rem;
  }

  .msg p {
    margin: 0.5rem 0 0;
    font-size: 1.45rem;
    line-height: 1.55;
  }

  .msg time,
  .source {
    font-size: 0.9rem;
    color: #9bc4a3;
  }

  .source {
    margin-left: 0.75rem;
  }

  .empty,
  .warn {
    font-size: 1.15rem;
    color: #c8dcc8;
  }

  .warn {
    color: #ffb74d;
  }

  .bootstrap-box {
    background: #1a3020;
    border: 2px solid #ffb74d;
    border-radius: 10px;
    padding: 1rem 1.1rem;
  }

  .bootstrap-box p {
    margin: 0 0 0.75rem;
    font-size: 1.2rem;
  }

  .btn.bootstrap {
    background: #c77800;
    border-color: #ffb74d;
    font-size: 1.35rem;
    padding: 0.9rem 1.4rem;
    width: 100%;
  }

  .status-ok {
    font-size: 1.1rem;
    color: #7fdc8a;
    margin: 0;
  }

  .hint {
    font-size: 1rem;
    color: #9bc4a3;
    margin: 0.35rem 0 0;
  }
</style>
