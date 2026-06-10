<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { isTauriRuntime } from "../lib/runtime";

  interface SkillSummary {
    total_count: number;
    python_count: number;
    javascript_count: number;
  }

  interface SkillItem {
    skill_id: string;
    name: string;
    language: string;
    category: string;
    status: string;
    created_at: string;
    description: string;
  }

  let summary: SkillSummary = { total_count: 14603, python_count: 12000, javascript_count: 2603 };
  let searchQuery = "";
  let selectedCategory = "";
  let skills: SkillItem[] = [];
  let isLoading = false;
  let errorMsg = "";

  const categories = [
    "Frontend_UI",
    "Database_Ops",
    "Security_Audit",
    "RND_Research",
    "System_Automation",
    "AI_Orchestrator",
    "Watchdog_Service"
  ];

  async function fetchSummary() {
    if (!isTauriRuntime()) return;
    try {
      summary = await invoke<SkillSummary>("get_skill_library_summary_cmd");
    } catch (e) {
      console.error("Failed to load skill summary from SQLite:", e);
    }
  }

  async function performSearch() {
    isLoading = true;
    errorMsg = "";
    if (!isTauriRuntime()) {
      skills = [];
      isLoading = false;
      return;
    }
    try {
      skills = await invoke<SkillItem[]>("search_skill_library_cmd", {
        query: searchQuery,
        category: selectedCategory ? selectedCategory : null
      });
    } catch (e: any) {
      errorMsg = "SQLite beceri arama hatası: " + e.toString();
    } finally {
      isLoading = false;
    }
  }

  // Trigger search on input
  let searchTimeout: any;
  function handleSearchInput() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      performSearch();
    }, 300);
  }

  onMount(async () => {
    await fetchSummary();
    await performSearch();
  });
</script>

<div class="explorer-container">
  <div class="explorer-header">
    <div class="header-text">
      <h2>Yerel Beceri Deposu (Beceri Kütüphanesi)</h2>
      <p>SQLite master veri tabanında kayıtlı tüm otonom becerileri gerçek zamanlı arayın, analiz edin ve görev yürütme altyapısına hazır beceri deposunu denetleyin.</p>
    </div>
    
    <div class="stats-panel">
      <div class="stat-card">
        <span class="stat-val">{summary.total_count.toLocaleString("tr-TR")}</span>
        <span class="stat-lbl">Toplam Aktif Beceri</span>
      </div>
      <div class="stat-card python">
        <span class="stat-val">{summary.python_count.toLocaleString("tr-TR")}</span>
        <span class="stat-lbl">Python Modülü</span>
      </div>
      <div class="stat-card js">
        <span class="stat-val">{summary.javascript_count.toLocaleString("tr-TR")}</span>
        <span class="stat-lbl">JavaScript Modülü</span>
      </div>
    </div>
  </div>

  <div class="filter-bar">
    <div class="search-box">
      <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input 
        type="text" 
        placeholder="Beceri adı, açıklaması veya ID ile arayın..." 
        bind:value={searchQuery}
        on:input={handleSearchInput}
      />
    </div>

    <div class="category-select">
      <select bind:value={selectedCategory} on:change={performSearch}>
        <option value="">Tüm Kategoriler</option>
        {#each categories as category}
          <option value={category}>{category}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if errorMsg}
    <div class="alert-box error">
      <span>{errorMsg}</span>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Yerel SQLite veri tabanı taranıyor...</p>
    </div>
  {:else}
    <div class="skills-grid">
      {#if skills.length === 0}
        <div class="empty-state">
          <p>Arama kriterlerinize uygun hiçbir beceri kaydı bulunamadı.</p>
        </div>
      {:else}
        {#each skills as skill}
          <div class="skill-card">
            <div class="skill-card-header">
              <span class="skill-id"><code>{skill.skill_id}</code></span>
              <span class={`lang-tag ${skill.language.toLowerCase()}`}>{skill.language.toUpperCase()}</span>
            </div>
            <h3 class="skill-name">{skill.name}</h3>
            <p class="skill-desc">{skill.description ? skill.description : "Açıklama girilmemiş."}</p>
            
            <div class="skill-footer">
              <span class="category-tag">{skill.category}</span>
              <span class={`status-tag ${skill.status.toLowerCase()}`}>{skill.status}</span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .explorer-container {
    padding: 30px 40px;
    max-width: 1200px;
    margin: 0 auto;
    color: #f4f4f5;
  }
  .explorer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 30px;
    margin-bottom: 30px;
    border-bottom: 1px solid #1f1f21;
    padding-bottom: 25px;
  }
  .header-text h2 {
    color: #f4f4f5;
    margin: 0 0 10px 0;
    font-size: 22px;
    font-weight: 600;
  }
  .header-text p {
    color: #8d8d95;
    margin: 0;
    font-size: 14px;
    max-width: 600px;
    line-height: 1.5;
  }

  .stats-panel {
    display: flex;
    gap: 15px;
  }
  .stat-card {
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 6px;
    padding: 12px 20px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    min-width: 140px;
  }
  .stat-card .stat-val {
    font-size: 20px;
    font-weight: bold;
    color: #0b74de;
  }
  .stat-card.python .stat-val { color: #306998; }
  .stat-card.js .stat-val { color: #f0db4f; }
  .stat-card .stat-lbl {
    font-size: 11px;
    color: #8d8d95;
    margin-top: 4px;
    text-transform: uppercase;
    font-weight: bold;
  }

  .filter-bar {
    display: flex;
    gap: 15px;
    margin-bottom: 25px;
  }
  .search-box {
    position: relative;
    flex: 1;
  }
  .search-box input {
    width: 100%;
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 6px;
    padding: 12px 14px 12px 42px;
    color: #f4f4f5;
    font-size: 14px;
    transition: border-color 0.2s;
  }
  .search-box input:focus {
    border-color: #0b74de;
    outline: none;
  }
  .search-icon {
    position: absolute;
    left: 14px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    color: #8d8d95;
  }

  .category-select select {
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 6px;
    padding: 12px 14px;
    color: #f4f4f5;
    font-size: 14px;
    cursor: pointer;
  }
  .category-select select:focus {
    border-color: #0b74de;
    outline: none;
  }

  .skills-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 20px;
  }

  .skill-card {
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 8px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 200px;
    transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
  }
  .skill-card:hover {
    transform: translateY(-4px);
    border-color: #0b74de;
    box-shadow: 0 8px 24px rgba(11, 116, 222, 0.15);
  }

  .skill-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .skill-id code {
    font-size: 11px;
    color: #8d8d95;
    background: #18181a;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .lang-tag {
    font-size: 10px;
    font-weight: bold;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .lang-tag.python { background: #306998; color: white; }
  .lang-tag.javascript { background: #f0db4f; color: #222; }

  .skill-name {
    font-size: 16px;
    font-weight: 600;
    color: #f4f4f5;
    margin: 0 0 10px 0;
  }
  .skill-desc {
    font-size: 13px;
    color: #a1a1aa;
    line-height: 1.5;
    margin: 0 0 20px 0;
    flex: 1;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
  }

  .skill-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid #1f1f21;
    padding-top: 12px;
  }
  .category-tag {
    font-size: 11px;
    color: #0b74de;
    font-weight: 500;
  }
  .status-tag {
    font-size: 10px;
    font-weight: bold;
    padding: 2px 6px;
    border-radius: 12px;
  }
  .status-tag.active { background: rgba(18, 109, 36, 0.15); color: #126d24; border: 1px solid #126d24; }
  .status-tag.failed { background: rgba(138, 31, 17, 0.15); color: #8a1f11; border: 1px solid #8a1f11; }
  .status-tag.pending { background: rgba(222, 116, 11, 0.15); color: #de740b; border: 1px solid #de740b; }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px;
    gap: 15px;
  }
  .spinner {
    width: 30px;
    height: 30px;
    border: 3px solid rgba(255,255,255,0.1);
    border-radius: 50%;
    border-top-color: #0b74de;
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    text-align: center;
    grid-column: 1 / -1;
    padding: 40px;
    background: #111112;
    border: 1px solid #1f1f21;
    border-radius: 6px;
    color: #8d8d95;
    font-size: 14px;
  }

  .alert-box {
    padding: 12px 16px;
    border-radius: 6px;
    font-size: 13px;
    margin-bottom: 20px;
  }
  .alert-box.error {
    background: rgba(138, 31, 17, 0.15);
    color: #ff6b6b;
    border: 1px solid #8a1f11;
  }
</style>
