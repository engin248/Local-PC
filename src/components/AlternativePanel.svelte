<script lang="ts">
  let { alternatives = [] } = $props<{
    alternatives: any[];
  }>();
</script>

<div class="alternative-panel-container">
  <h3>ALTERNATİF ve KARAR MATRİSİ (Ajan Önerileri)</h3>

  {#if alternatives.length > 0}
    <div class="matrix-grid">
      {#each alternatives as alt}
        <div class="matrix-card" class:selected={alt.selected === 1}>
          <div class="matrix-header">
            <h4>{alt.title}</h4>
            {#if alt.selected === 1}
              <span class="badge selected-badge">SEÇİLDİ</span>
            {:else}
              <span class="badge rejected-badge">ELENDİ</span>
            {/if}
          </div>
          <p class="desc">{alt.description}</p>
          <div class="scores">
            <div class="score-item"><span>Doğruluk:</span> <b>{alt.accuracy_score}/10</b></div>
            <div class="score-item"><span>Güvenlik:</span> <b>{alt.safety_score}/10</b></div>
            <div class="score-item"><span>Risk:</span> <b>{alt.dependency_score}/10</b></div>
          </div>
          <div class="pros-cons">
            <div class="pros"><strong>Avantajlar:</strong><br/>- Hızlı çözüm<br/>- İzole müdahale</div>
            <div class="cons"><strong>Dezavantajlar:</strong><br/>- Ekstra test gerektirir<br/>- Mimari dışı yama</div>
          </div>
          {#if alt.selected !== 1}
            <button class="override-btn">Bunu Zorla Seç (Override)</button>
          {/if}
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Alternatif analizi henüz çalıştırılmadı.</p>
  {/if}
</div>

<style>
  .alternative-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  
  
  
  
  
  .score { font-family: monospace; font-weight: bold; }
  .badge {
    font-size: 0.65rem;
    padding: 2px 5px;
    border-radius: 3px;
    font-weight: bold;
  }
  .selected-badge { background: #4ec9b0; color: #1e1e1e; }
  .rejected-badge { background: #3c3c3c; color: #888; }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
  .matrix-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px; margin-top: 12px; }
  .matrix-card { background: #18181a; border: 1px solid #2d2d31; border-radius: 6px; padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  .matrix-card.selected { border-color: #47d18c; box-shadow: 0 0 10px rgba(71, 209, 140, 0.1); }
  .matrix-header { display: flex; justify-content: space-between; align-items: flex-start; }
  .matrix-header h4 { margin: 0; color: #f4f4f5; font-size: 15px; }
  .desc { font-size: 12px; color: #a1a1aa; margin: 0; line-height: 1.4; }
  .scores { display: flex; gap: 12px; background: #111112; padding: 8px; border-radius: 4px; }
  .score-item { font-size: 11px; color: #8d8d95; }
  .score-item b { color: #f4f4f5; margin-left: 4px; }
  .pros-cons { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11px; }
  .pros { color: #47d18c; background: rgba(71, 209, 140, 0.05); padding: 6px; border-radius: 4px; }
  .cons { color: #e03131; background: rgba(224, 49, 49, 0.05); padding: 6px; border-radius: 4px; }
  .override-btn { background: transparent; border: 1px solid #0b74de; color: #0b74de; padding: 6px; border-radius: 4px; cursor: pointer; font-size: 11px; margin-top: auto; }
  .override-btn:hover { background: #0b74de; color: #fff; }
</style>
