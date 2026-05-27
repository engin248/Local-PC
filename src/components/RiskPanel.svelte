<script lang="ts">
  let { task } = $props<{
    task: any;
  }>();
</script>

<div class="risk-panel-container">
  <h3>GÜVENLİK DUVARI & RİSK ANALİZİ</h3>

  {#if task && task.risk_level}
    <div class="risk-summary">
      <div class="risk-level-display {task.risk_level}">
        <span class="label">Maksimum Risk Düzeyi</span>
        <span class="value">{task.risk_level.toUpperCase()}</span>
      </div>
      <div class="risk-rules">
        <h4>AKTİF GÜVENLİK BARİYERLERİ VE KURALLARI:</h4>
        <ul>
          {#if task.risk_level === 'low'}
            <li>Log ve Denetim: Kayıt yeterli.</li>
          {:else if task.risk_level === 'medium'}
            <li>Orta Risk Bariyeri: Değişiklik öncesi en az 1 Checkpoint doğrulaması zorunludur.</li>
          {:else if task.risk_level === 'high'}
            <li>Yüksek Risk Bariyeri: Çift checkpoint doğrulaması zorunludur.</li>
            <li>Geri Alma Koruması: snapshot yedeği olmadan dosya yazma engellenmiştir.</li>
            <li>Kullanıcı Onay Kapısı: Kullanıcının explicit (açık) onayı olmadan işlem tamamlanamaz.</li>
          {:else if task.risk_level === 'critical'}
            <li>KRİTİK RİSK BARİYERLERİ:</li>
            <li>Alternatif Analiz Kapısı: 11 kriterli analiz zorunludur.</li>
            <li>Yedekleme Kapısı: Snapshot ve hash kontrolü zorunludur.</li>
            <li>Çift Kontrol: AI kararları ve connector testleri çift kontrol edilmektedir.</li>
            <li>Kullanıcı Onayı: Manuel onay alınmadan hiçbir işlem yürütülemez.</li>
          {/if}
        </ul>
      </div>
    </div>
  {:else}
    <p class="empty-msg">Risk analizi henüz yapılmadı.</p>
  {/if}
</div>

<style>
  .risk-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .risk-summary {
    display: flex;
    gap: 20px;
    align-items: flex-start;
  }
  .risk-level-display {
    padding: 15px;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 150px;
    color: #1e1e1e;
    font-weight: bold;
    text-align: center;
  }
  .risk-level-display.low { background: #4ec9b0; }
  .risk-level-display.medium { background: #dcdcaa; }
  .risk-level-display.high { background: #ce9178; color: white; }
  .risk-level-display.critical { background: #f44747; color: white; }

  .risk-level-display .label { font-size: 0.6rem; text-transform: uppercase; opacity: 0.8; margin-bottom: 5px; }
  .risk-level-display .value { font-size: 1.3rem; letter-spacing: 1px; }

  .risk-rules {
    flex: 1;
    background: #252526;
    border: 1px solid #3c3c3c;
    padding: 10px;
    border-radius: 4px;
  }
  .risk-rules h4 { margin: 0 0 8px 0; font-size: 0.75rem; color: #eee; }
  ul { margin: 0; padding-left: 15px; }
  li { font-size: 0.75rem; color: #aaa; margin: 4px 0; }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
