<script lang="ts">
  export let onCreate: (title: string, userRequest: string) => void;

  let newTitle = "";
  let newTaskType = "Analiz";
  let selectedAgents = { codex: true, oam: true, antigravity: false, cursor: false };
  let newRequest = "";
  let attachedImageBase64 = "";

  function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (item.type.indexOf("image") !== -1) {
        const file = item.getAsFile();
        if (file) {
          const reader = new FileReader();
          reader.onload = (event) => {
            attachedImageBase64 = event.target?.result as string;
          };
          reader.readAsDataURL(file);
          e.preventDefault();
        }
        break;
      }
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.type.startsWith("image/")) {
        const reader = new FileReader();
        reader.onload = (event) => {
          attachedImageBase64 = event.target?.result as string;
        };
        reader.readAsDataURL(file);
      }
    }
  }

  function clearAttachment() {
    attachedImageBase64 = "";
  }

  function handleCreate(e: Event) {
    e.preventDefault();
    if (!newTitle || !newRequest) return;
    let agentTags = Object.entries(selectedAgents).filter(([_, v]) => v).map(([k, _]) => k.toUpperCase()).join(",");
    let finalRequest = "[" + newTaskType + "] [Ajanlar: " + agentTags + "] " + newRequest;
    if (attachedImageBase64) {
      finalRequest += "\n\n[GÖRSEL EKLENTİSİ: " + attachedImageBase64.slice(0, 100) + "...]";
    }
    onCreate(newTitle, finalRequest);
    newTitle = "";
    newRequest = "";
    attachedImageBase64 = "";
  }
</script>

<div class="intake-panel-container">
  <div class="intake-header">
    <h2>Yeni Görev & Operasyon Başlat (Intake)</h2>
    <p>Ajanların gerçekleştirmesini istediğiniz görevi detaylandırın, kaynakları atayın ve süreci başlatın.</p>
  </div>

  <form class="intake-form" on:submit={handleCreate}>
    
    <div class="form-group">
      <label for="taskTitle">Görev Başlığı / Kısa Tanım</label>
      <input id="taskTitle" type="text" placeholder="Örn: Veritabanı optmizasyonu..." bind:value={newTitle} />
    </div>

    <div class="form-row">
      <div class="form-group half">
        <label for="taskType">Operasyon Tipi</label>
        <select id="taskType" bind:value={newTaskType} class="task-type-select">
          <option value="Analiz">Sadece Analiz & İnceleme</option>
          <option value="Kod Yazma">Kod Değişikliği / Yazma</option>
          <option value="Araştırma">Dış İnternet Araştırması</option>
          <option value="Sistem">Sistem Taraması</option>
        </select>
      </div>

      <fieldset class="form-group half agent-fieldset">
        <legend>Ajan Kaynak Tahsisi (Agent Allocation)</legend>
        <div class="agent-selectors">
          <label><input type="checkbox" bind:checked={selectedAgents.codex} /> Codex (Uygulayıcı)</label>
          <label><input type="checkbox" bind:checked={selectedAgents.oam} /> OAM (Denetçi)</label>
          <label><input type="checkbox" bind:checked={selectedAgents.antigravity} /> AntiGrav (Çapraz Kontrol)</label>
          <label><input type="checkbox" bind:checked={selectedAgents.cursor} /> Cursor (Dosya/Satır Okuma)</label>
        </div>
      </fieldset>
    </div>

    <div class="form-group">
      <label for="taskDesc">Kullanıcı Talebi / Detaylar (Prompt) (Ekran görüntüsü yapıştırabilirsiniz)</label>
      <textarea 
        id="taskDesc" 
        rows="6" 
        placeholder="Gerçekleştirilecek işlemi adım adım detaylandırın... (Ctrl+V ile resim yapıştırabilir veya resmi bu kutuya sürükleyip bırakabilirsiniz)" 
        bind:value={newRequest}
        on:paste={handlePaste}
        on:dragover={handleDragOver}
        on:drop={handleDrop}
      ></textarea>
    </div>

    {#if attachedImageBase64}
      <div class="image-preview-container">
        <span class="preview-label">Ekran Görüntüsü Eklendi:</span>
        <div class="preview-box">
          <img src={attachedImageBase64} alt="Screenshot" class="screenshot-preview" />
          <button type="button" class="clear-attachment-btn" on:click={clearAttachment}>Görseli Kaldır</button>
        </div>
      </div>
    {/if}

    <button type="submit" class="submit-btn" disabled={!newTitle || !newRequest}>
      GÖREVİ BAŞLAT (Intake Gate 1)
    </button>
  </form>
</div>

<style>
  .intake-panel-container {
    padding: 30px 40px;
    max-width: 900px;
    margin: 20px auto;
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }
  .intake-header { margin-bottom: 30px; border-bottom: 1px solid #1f1f21; padding-bottom: 15px; }
  .intake-header h2 { color: #f4f4f5; margin: 0 0 10px 0; font-size: 22px; font-weight: 600; }
  .intake-header p { color: #8d8d95; margin: 0; font-size: 14px; }
  
  .intake-form { display: flex; flex-direction: column; gap: 20px; }
  .form-group { display: flex; flex-direction: column; gap: 8px; }
  .form-row { display: flex; gap: 20px; }
  .half { flex: 1; }

  label, legend { color: #a1a1aa; font-size: 13px; font-weight: bold; text-transform: uppercase; letter-spacing: 0.5px; }
  .agent-fieldset {
    margin: 0;
    padding: 0;
    border: 0;
  }
  input[type="text"], select, textarea {
    background: #18181a;
    border: 1px solid #2d2d31;
    border-radius: 6px;
    padding: 12px 14px;
    color: #f4f4f5;
    font-size: 14px;
    font-family: inherit;
    transition: border-color 0.2s;
  }
  input[type="text"]:focus, select:focus, textarea:focus { border-color: #0b74de; outline: none; }
  
  .agent-selectors {
    display: flex; flex-wrap: wrap; gap: 15px; padding: 12px 14px;
    background: #18181a; border: 1px solid #2d2d31; border-radius: 6px;
  }
  .agent-selectors label {
    display: flex; align-items: center; gap: 6px; color: #f4f4f5; font-size: 13px; font-weight: normal; cursor: pointer; text-transform: none; letter-spacing: 0;
  }

  .submit-btn {
    margin-top: 15px;
    background: #0b74de;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 16px;
    font-size: 15px;
    font-weight: bold;
    cursor: pointer;
    text-transform: uppercase;
    transition: background 0.2s, transform 0.1s;
  }
  .submit-btn:hover:not(:disabled) { background: #005bb5; transform: scale(1.01); }
  .submit-btn:disabled { background: #1f1f21; color: #555; cursor: not-allowed; }

  /* Image attachment preview */
  .image-preview-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 10px;
    padding: 15px;
    background: #18181a;
    border: 1px solid #2d2d31;
    border-radius: 6px;
  }
  .preview-label {
    font-size: 12px;
    color: #0b74de;
    font-weight: bold;
  }
  .preview-box {
    display: flex;
    align-items: center;
    gap: 15px;
  }
  .screenshot-preview {
    max-width: 250px;
    max-height: 150px;
    border-radius: 4px;
    border: 1px solid #2d2d31;
    box-shadow: 0 4px 10px rgba(0,0,0,0.3);
  }
  .clear-attachment-btn {
    background: #8a1f11;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 12px;
    cursor: pointer;
    font-weight: bold;
    text-transform: uppercase;
    transition: background 0.2s;
  }
  .clear-attachment-btn:hover {
    background: #b52c1a;
  }
</style>
