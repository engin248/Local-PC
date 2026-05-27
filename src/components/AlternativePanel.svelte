<script lang="ts">
  let { alternatives = [] } = $props<{
    alternatives: any[];
  }>();
</script>

<div class="alternative-panel-container">
  <h3>ALTERNATİF ANALİZİ (11 EKSENLİ PUANLAMA)</h3>

  {#if alternatives.length > 0}
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Alternatif Başlığı</th>
            <th>Açıklama</th>
            <th>Doğruluk</th>
            <th>Güvenlik</th>
            <th>Bağımlılık</th>
            <th>Durum</th>
          </tr>
        </thead>
        <tbody>
          {#each alternatives as alt}
            <tr class:selected={alt.selected === 1}>
              <td>
                <strong>{alt.title}</strong>
              </td>
              <td>{alt.description}</td>
              <td class="score">{alt.accuracy_score}/10</td>
              <td class="score">{alt.safety_score}/10</td>
              <td class="score">{alt.dependency_score}/10</td>
              <td>
                {#if alt.selected === 1}
                  <span class="badge selected-badge">SEÇİLDİ</span>
                {:else}
                  <span class="badge rejected-badge">ELENDİ</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
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
  .table-wrapper {
    overflow-x: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.75rem;
    text-align: left;
  }
  th, td {
    padding: 8px;
    border-bottom: 1px solid #2d2d2d;
  }
  th { background: #252526; color: #888; text-transform: uppercase; font-size: 0.65rem; }
  tr.selected {
    background: #1a2a1e;
  }
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
</style>
