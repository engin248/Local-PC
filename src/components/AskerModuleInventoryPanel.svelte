<script lang="ts">
  import { onMount } from "svelte";
  import { invokePanel } from "../lib/tauriInvoke";

  interface AskerModuleSummary {
    expected_module_total: number;
    registered_module_count: number;
    active_in_panel_count: number;
    skill_count: number;
    source_kind: string;
    source_path: string | null;
    inventory_match: boolean;
    last_error: string | null;
  }

  interface AskerModuleRecord {
    module_id: string;
    name: string;
    duty: string | null;
    skills: string[];
    system_skills: string[];
    status: string;
    source: string;
  }

  let summary: AskerModuleSummary | null = $state(null);
  let modules: AskerModuleRecord[] = $state([]);
  let selectedModuleId = $state<string | null>(null);
  let moduleSkills: string[] = $state([]);
  let searchQuery = $state("");
  let isLoading = $state(false);
  let errorMsg = $state("");

  const filteredModules = $derived(
    modules.filter((row) => {
      const q = searchQuery.trim().toLowerCase();
      if (!q) return true;
      return (
        row.module_id.toLowerCase().includes(q) ||
        row.name.toLowerCase().includes(q) ||
        (row.duty || "").toLowerCase().includes(q) ||
        row.skills.some((s) => s.toLowerCase().includes(q)) ||
        row.system_skills.some((s) => s.toLowerCase().includes(q))
      );
    })
  );

  async function refreshInventory() {
    isLoading = true;
    errorMsg = "";
    try {
      summary = await invokePanel("get_asker_module_summary_cmd");
      modules = await invokePanel("get_asker_module_inventory_cmd", {
        limit: 500,
      });
      if (selectedModuleId) {
        await loadModuleSkills(selectedModuleId);
      }
    } catch (e: unknown) {
      errorMsg = "Modül envanteri okunamadı: " + String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadModuleSkills(moduleId: string) {
    selectedModuleId = moduleId;
    try {
      moduleSkills = await invokePanel("get_module_skills_cmd", { moduleId });
    } catch {
      moduleSkills = [];
    }
  }

  onMount(() => {
    refreshInventory();
  });
</script>

<div class="module-inventory-panel">
  <div class="panel-head">
    <div>
      <h3>Asker Motoru Modül Envanteri</h3>
      <p>
        314 modül kayıtlıdır; görev, beceri ve sistem becerileri tablo/envanter dosyalarından okunur.
        Komuta panelinde aktif platform sayısı ayrı gösterilir.
      </p>
    </div>
    <button class="refresh-btn" onclick={refreshInventory} disabled={isLoading}>
      {isLoading ? "Taranıyor..." : "Yenile"}
    </button>
  </div>

  {#if summary}
    <div class="stats-row">
      <div class="stat">
        <span class="val">{summary.expected_module_total}</span>
        <span class="lbl">Beklenen modül</span>
      </div>
      <div class="stat" class:warn={!summary.inventory_match}>
        <span class="val">{summary.registered_module_count}</span>
        <span class="lbl">Tablodan okunan</span>
      </div>
      <div class="stat">
        <span class="val">{summary.active_in_panel_count}</span>
        <span class="lbl">Panelde aktif platform</span>
      </div>
      <div class="stat">
        <span class="val">{summary.skill_count.toLocaleString("tr-TR")}</span>
        <span class="lbl">Beceri kaydı</span>
      </div>
    </div>

    <div class="source-line">
      <span>Kaynak: <strong>{summary.source_kind}</strong></span>
      {#if summary.source_path}
        <span class="path">{summary.source_path}</span>
      {/if}
      {#if summary.inventory_match}
        <span class="match ok">Envanter 314 ile eşleşiyor</span>
      {:else if summary.registered_module_count > 0}
        <span class="match warn">
          Tablo sayısı ({summary.registered_module_count}) beklenen 314 ile eşleşmiyor
        </span>
      {:else}
        <span class="match warn">Envanter tablosu bağlı değil — beklenen 314 modül korunuyor</span>
      {/if}
    </div>

    {#if summary.last_error}
      <div class="alert">{summary.last_error}</div>
    {/if}
  {/if}

  {#if errorMsg}
    <div class="alert">{errorMsg}</div>
  {/if}

  <div class="search-row">
    <input
      type="text"
      placeholder="Modül no, ad, görev veya beceri ile ara..."
      bind:value={searchQuery}
    />
  </div>

  {#if isLoading}
    <p class="loading">Modül tabloları taranıyor...</p>
  {:else if filteredModules.length === 0}
    <p class="empty">
      Henüz tablo kaydı yok. UZMAN_HAVUZU.json veya skill_library.sqlite bağlandığında
      314 modülün görev ve becerileri burada listelenir.
    </p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Modül</th>
            <th>Ad</th>
            <th>Görev</th>
            <th>Beceriler</th>
            <th>Sistem becerileri</th>
            <th>Durum</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredModules as row}
            <tr
              class:selected={selectedModuleId === row.module_id}
              onclick={() => loadModuleSkills(row.module_id)}
            >
              <td><code>{row.module_id}</code></td>
              <td>{row.name}</td>
              <td>{row.duty || "—"}</td>
              <td>
                {#if row.skills.length > 0}
                  {row.skills.join(", ")}
                {:else if selectedModuleId === row.module_id && moduleSkills.length > 0}
                  {moduleSkills.join(", ")}
                {:else}
                  —
                {/if}
              </td>
              <td>{row.system_skills.length > 0 ? row.system_skills.join(", ") : "—"}</td>
              <td><span class="status">{row.status}</span></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .module-inventory-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 6px;
    margin-bottom: 16px;
    color: #f4f4f5;
  }
  .panel-head {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
    margin-bottom: 14px;
  }
  h3 {
    margin: 0 0 6px;
    color: #f2f2f4;
  }
  p {
    margin: 0;
    color: #9a9aa3;
    font-size: 0.85rem;
    line-height: 1.45;
    max-width: 720px;
  }
  .refresh-btn {
    background: #0b74de;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 8px 14px;
    cursor: pointer;
    font-weight: 600;
    white-space: nowrap;
  }
  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }
  .stats-row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 12px;
  }
  .stat {
    background: #111112;
    border: 1px solid #2a2a2d;
    border-radius: 6px;
    padding: 10px 14px;
    min-width: 120px;
  }
  .stat.warn {
    border-color: #8a5a12;
  }
  .val {
    display: block;
    font-size: 1.25rem;
    font-weight: 700;
    color: #0b74de;
  }
  .stat.warn .val {
    color: #f8c14a;
  }
  .lbl {
    font-size: 0.72rem;
    color: #8d8d95;
    text-transform: uppercase;
    font-weight: 700;
  }
  .source-line {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    font-size: 0.8rem;
    color: #b8b8c0;
    margin-bottom: 12px;
  }
  .path {
    word-break: break-all;
    color: #7eb8ff;
  }
  .match.ok {
    color: #6ee7a0;
    font-weight: 700;
  }
  .match.warn {
    color: #f8c14a;
    font-weight: 700;
  }
  .alert {
    background: rgba(138, 90, 18, 0.15);
    border: 1px solid #8a5a12;
    color: #f8c14a;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 0.82rem;
    margin-bottom: 12px;
  }
  .search-row input {
    width: 100%;
    background: #111112;
    border: 1px solid #2a2a2d;
    border-radius: 6px;
    padding: 10px 12px;
    color: #f4f4f5;
    margin-bottom: 12px;
  }
  .loading,
  .empty {
    color: #888;
    font-size: 0.88rem;
  }
  .table-wrap {
    overflow-x: auto;
    max-height: 420px;
    overflow-y: auto;
    border: 1px solid #2a2a2d;
    border-radius: 6px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
  }
  th,
  td {
    text-align: left;
    padding: 8px 10px;
    border-bottom: 1px solid #2a2a2d;
    vertical-align: top;
  }
  th {
    position: sticky;
    top: 0;
    background: #141416;
    color: #c8c8cc;
    z-index: 1;
  }
  tr {
    cursor: pointer;
  }
  tr:hover,
  tr.selected {
    background: rgba(11, 116, 222, 0.08);
  }
  code {
    font-size: 0.78rem;
    color: #9fd3ff;
  }
  .status {
    font-size: 0.72rem;
    font-weight: 700;
    color: #f8c14a;
  }
</style>
