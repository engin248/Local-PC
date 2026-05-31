<script lang="ts">
  const decompositionRules = [
    "Her görev önce parçalanır.",
    "Konu, alt konu, kriter ve alt kriter çıkarılır.",
    "En küçük uygulanabilir / doğrulanabilir parçaya kadar ayrıştırılır.",
    "Her konu ve alt konu için gerçek hayattaki bütün makul alternatif çözümler çıkarılır.",
    "Alternatifler de aynı mantıkla parça, kriter ve alt kriter bazında değerlendirilir."
  ];

  const decisionRules = [
    "Önce kabul edilmiş doğrular seçilir.",
    "Sonra her parçanın ve her kriterin en iyi alternatifi seçilir.",
    "Doğru kabul edilen yaklaşım ile en iyi alternatif en küçük parçadan konu seviyesine birleştirilir.",
    "Uygulanabilir seçenek bundan sonra seçilir.",
    "Çözümleme ayrı yapılır; uygulama ayrı yapılır."
  ];

  const roleRules = [
    "İşlemi yapan ayrı olur.",
    "İşlem sırasında işlemin doğru yapılmasını sağlayan ayrı olur.",
    "Kontrol eden ayrı olur.",
    "Bağımsız doğrulayan ayrı olur.",
    "Son onayı veren ayrı olur."
  ];

  const packageRules = [
    "Proje planı",
    "İş",
    "İşlem",
    "İşlem sırası",
    "Kullanılacak teknoloji",
    "İşlem etki alanı",
    "Kontrol noktası",
    "Kontrol kriterleri",
    "Test operasyon planı",
    "Rollback planı",
    "Alt birim paketi",
    "Veritabanı kaydı"
  ];
</script>

<section class="doctrine-panel" aria-label="Operasyon doktrini">
  <div class="doctrine-header">
    <div>
      <span class="eyebrow">Yerel Bilgisayar Paneli Zorunlu Mimari Standardı</span>
      <h3>Parçala → doğruyu seç → en iyi alternatifi seç → paketle → kaydet → doğrula</h3>
    </div>
    <div class="gate-lock">Plan Yoksa İşlem Yok</div>
  </div>

  <div class="doctrine-grid">
    <div class="doctrine-card">
      <div class="card-title">1. Parçalama</div>
      {#each decompositionRules as rule, index}
        <div class="rule-row">
          <span>{index + 1}</span>
          <p>{rule}</p>
        </div>
      {/each}
    </div>

    <div class="doctrine-card">
      <div class="card-title">2. Doğru & Alternatif Seçimi</div>
      {#each decisionRules as rule, index}
        <div class="rule-row">
          <span>{index + 6}</span>
          <p>{rule}</p>
        </div>
      {/each}
    </div>

    <div class="doctrine-card">
      <div class="card-title">3. Ayrı Roller</div>
      {#each roleRules as rule, index}
        <div class="rule-row">
          <span>{index + 11}</span>
          <p>{rule}</p>
        </div>
      {/each}
    </div>
  </div>

  <div class="mandatory-strip">
    <strong>Alt birime verilmeden ve veritabanına kaydedilmeden işlem başlayamaz:</strong>
    {#each packageRules as gate}
      <span>{gate}</span>
    {/each}
  </div>
</section>

<style>
  .doctrine-panel {
    margin: 15px;
    padding: 16px;
    background: #111112;
    border: 1px solid #2d2d31;
    border-left: 4px solid #f59e0b;
    border-radius: 6px;
    color: #f4f4f5;
  }

  .doctrine-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #2d2d31;
    margin-bottom: 14px;
  }

  .eyebrow {
    display: block;
    color: #f59e0b;
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    margin-bottom: 5px;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
    color: #fff;
    letter-spacing: 0;
  }

  .gate-lock {
    flex-shrink: 0;
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.45);
    color: #fbbf24;
    border-radius: 4px;
    padding: 7px 10px;
    font-size: 0.72rem;
    font-weight: 900;
    text-transform: uppercase;
  }

  .doctrine-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }

  .doctrine-card {
    background: #18181a;
    border: 1px solid #2d2d31;
    border-radius: 6px;
    padding: 12px;
  }

  .card-title {
    color: #f4f4f5;
    font-size: 0.78rem;
    font-weight: 900;
    margin-bottom: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .rule-row {
    display: grid;
    grid-template-columns: 24px 1fr;
    gap: 8px;
    align-items: start;
    padding: 7px 0;
    border-top: 1px solid #242428;
  }

  .rule-row span {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: #242428;
    color: #f59e0b;
    border-radius: 50%;
    font-size: 0.68rem;
    font-weight: 900;
  }

  .rule-row p {
    margin: 0;
    color: #d4d4d8;
    font-size: 0.78rem;
    line-height: 1.35;
  }

  .mandatory-strip {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    padding: 10px;
    background: #141416;
    border: 1px solid #2d2d31;
    border-radius: 4px;
    color: #c8c8cc;
    font-size: 0.78rem;
  }

  .mandatory-strip strong { color: #fff; }

  .mandatory-strip span {
    background: rgba(245, 158, 11, 0.12);
    color: #fbbf24;
    border: 1px solid rgba(245, 158, 11, 0.28);
    border-radius: 4px;
    padding: 3px 7px;
    font-weight: 800;
  }

  @media (max-width: 1100px) {
    .doctrine-grid { grid-template-columns: 1fr; }
    .doctrine-header { flex-direction: column; }
  }
</style>
