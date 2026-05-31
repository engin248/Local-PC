<script lang="ts">
  let { task = null, onSavePlan } = $props<{
    task: any;
    onSavePlan: (plan: any) => void;
  }>();

  let task_definition = $state("");
  let purpose = $state("Lokal bilgisayar kontrol paneli içinde seçilen hedefi güvenli kapılarla analiz etmek ve kullanıcı onayı olmadan yazma işlemi yapmamak.");
  let scope = $state("Lokal Bilgisayar Kontrol Paneli proje kökü ve storage/app.db üzerindeki salt okunur analiz kapsamı.");
  let topic = $state("Dosya Analizi ve Yazımı");
  let sub_topic = $state("Yetkili Onay Kapılı Yazma");
  let criterion = $state("Tam Geri Alınabilirlik");
  let sub_criterion = $state("Snapshot-Bazlı Rollback");
  let alternatives = $state([
    "Sadece oku ve raporla",
    "Uygulama yapma, manuel plan üret",
    "Onaylı, kontrollü ve rollback destekli uygula",
    "Onaysız ve rollback'siz doğrudan uygula - elenen alternatif"
  ]);
  let risk_analysis = $state("high");
  let impact_area = $state("storage/app.db");
  let technology_selection = $state("Tauri, Rust, SQLite, Svelte");
  let dependency_analysis = $state("Düşük bağımlılık (İnternetsiz lokal çalışma)");
  let checkpoints = $state(["Planlama kapısı kontrolü", "Yetki eşleştirme kontrolü", "Risk analiz kontrolü"]);
  let test_criteria = $state(["file_exists:storage/app.db"]);
  let rollback_plan = $state("Değişiklikten önce gerçek hedef snapshot'ı alınır; hata halinde kayıtlı snapshot hedefe geri yüklenir.");
  let operation_plan = $state("action:code_analysis, action:approval_check, action:snapshot_create, action:test_run, action:report_generate");
  let authorized_deciders = $state(["local_projects", "local_app_db", "user"]);
  let accepted_correct_approach_reason = $state("Genel doğru yaklaşım kullanıcı iradesini, veri gizliliğini, rollback ve test edilebilirliği korur.");
  let selected_best_option_reason = $state("Seçilen en iyi seçenek mevcut sistemle uyumlu, düşük riskli, rollback destekli ve test edilebilirdir.");
  let operation_sequence = $state([
    "Çözümleme yap",
    "Kabul edilmiş doğruyu seç",
    "Her kriter için en iyi alternatifi seç",
    "Uygulama paketini alt birime ver",
    "Kontrol et",
    "Bağımsız doğrula",
    "Son onay ver"
  ]);
  let control_criteria = $state(["Plan var", "Etki alanı var", "Teknoloji var", "Test var", "Rollback var"]);
  let executor_role = $state("executor");
  let correctness_guard_role = $state("correctness_guard");
  let controller_role = $state("controller");
  let independent_verifier_role = $state("independent_verifier");
  let final_approver_role = $state("final_approver");
  let per_part_alternative_policy = $state("Her atomik parça için gerçek hayattaki tüm makul alternatifler aynı kriterlerle değerlendirilir ve veritabanına kaydedilir.");

  function parseCommaList(value: string) {
    return value
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }

  $effect(() => {
    task_definition = task?.title || "";
    risk_analysis = task?.risk_level || "high";
  });

  function handleSubmit(e: Event) {
    e.preventDefault();
    onSavePlan({
      task_definition,
      purpose,
      scope,
      topic,
      sub_topic,
      criterion,
      sub_criterion,
      alternatives,
      risk_analysis,
      impact_area,
      technology_selection,
      dependency_analysis,
      checkpoints,
      test_criteria,
      rollback_plan,
      operation_plan,
      authorized_deciders,
      accepted_correct_approach_reason,
      selected_best_option_reason,
      operation_sequence,
      control_criteria,
      executor_role,
      correctness_guard_role,
      controller_role,
      independent_verifier_role,
      final_approver_role,
      per_part_alternative_policy
    });
  }
</script>

<div class="planning-container">
  <h3>PLANLAMA STANDARDI & KİLİT AÇMA FORMU (MİMARİ ZORUNLU ALANLAR)</h3>
  
  {#if task?.planning_status === 'planning_complete'}
    <div class="success-alert">
      <strong>PLAN ONAYLANDI:</strong> Mimari zorunlu alanlar, rol ayrımı, test ve rollback doğrulandı; operasyon paketi veritabanına kaydedildi.
    </div>
  {:else}
    <div class="warning-alert">
      <strong>PLANLAMA KİLİDİ AKTİF:</strong> Plan, işlem sırası, teknoloji, etki alanı, kontrol kriterleri, test ve rollback olmadan Execution Engine çalışmaz.
    </div>
  {/if}

  <form onsubmit={handleSubmit} class="plan-form">
    <div class="form-grid">
      <div class="field">
        <label for="task-definition">1. Görev Tanımı</label>
        <input id="task-definition" bind:value={task_definition} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-purpose">2. Amaç</label>
        <input id="plan-purpose" bind:value={purpose} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-scope">3. Kapsam</label>
        <input id="plan-scope" bind:value={scope} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-topic">4. Konu</label>
        <input id="plan-topic" bind:value={topic} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-subtopic">5. Alt Konu</label>
        <input id="plan-subtopic" bind:value={sub_topic} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-criterion">6. Kriter</label>
        <input id="plan-criterion" bind:value={criterion} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-subcriterion">7. Alt Kriter</label>
        <input id="plan-subcriterion" bind:value={sub_criterion} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-alternatives">8. Alternatifler (Virgülle Ayırın)</label>
        <input id="plan-alternatives" value={alternatives.join(', ')} oninput={(event) => alternatives = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-risk">9. Risk Analizi Seviyesi</label>
        <select id="plan-risk" bind:value={risk_analysis} disabled={task?.planning_status === 'planning_complete'}>
          <option value="low">Low (Düşük Risk)</option>
          <option value="medium">Medium (Orta Risk)</option>
          <option value="high">High (Yüksek Risk)</option>
          <option value="critical">Critical (Kritik Risk)</option>
        </select>
      </div>
      <div class="field">
        <label for="plan-impact">10. Etki Alanı</label>
        <input id="plan-impact" bind:value={impact_area} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-tech">11. Teknoloji Seçimi</label>
        <input id="plan-tech" bind:value={technology_selection} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-dep">12. Bağımlılık Analizi</label>
        <input id="plan-dep" bind:value={dependency_analysis} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-checkpoints">13. Kontrol Noktaları (Virgülle Ayırın)</label>
        <input id="plan-checkpoints" value={checkpoints.join(', ')} oninput={(event) => checkpoints = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-tests">14. Test Kriterleri (Virgülle Ayırın)</label>
        <input id="plan-tests" value={test_criteria.join(', ')} oninput={(event) => test_criteria = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-rollback">15. Geri Alma Planı</label>
        <input id="plan-rollback" bind:value={rollback_plan} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-operation">16. Operasyon Planı</label>
        <input id="plan-operation" bind:value={operation_plan} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="plan-deciders">17. Yetkili Karar Noktaları (Virgülle Ayırın)</label>
        <input id="plan-deciders" value={authorized_deciders.join(', ')} oninput={(event) => authorized_deciders = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field wide">
        <label for="correct-approach-reason">Genel Doğru Yaklaşım Gerekçesi</label>
        <input id="correct-approach-reason" bind:value={accepted_correct_approach_reason} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field wide">
        <label for="best-option-reason">Seçilen En İyi Seçenek Gerekçesi</label>
        <input id="best-option-reason" bind:value={selected_best_option_reason} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field wide">
        <label for="operation-sequence">İşlem Sırası (Virgülle Ayırın)</label>
        <input id="operation-sequence" value={operation_sequence.join(', ')} oninput={(event) => operation_sequence = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field wide">
        <label for="control-criteria">Kontrol Kriterleri (Virgülle Ayırın)</label>
        <input id="control-criteria" value={control_criteria.join(', ')} oninput={(event) => control_criteria = parseCommaList(event.currentTarget.value)} disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="executor-role">İşlemi Yapan Rol</label>
        <input id="executor-role" bind:value={executor_role} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="guard-role">Doğru Yapılmasını Sağlayan Rol</label>
        <input id="guard-role" bind:value={correctness_guard_role} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="controller-role">Kontrol Eden Rol</label>
        <input id="controller-role" bind:value={controller_role} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="verifier-role">Bağımsız Doğrulayan Rol</label>
        <input id="verifier-role" bind:value={independent_verifier_role} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field">
        <label for="approver-role">Son Onay Veren Rol</label>
        <input id="approver-role" bind:value={final_approver_role} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
      <div class="field wide">
        <label for="alternative-policy">Her Parça İçin Alternatif Politikası</label>
        <input id="alternative-policy" bind:value={per_part_alternative_policy} required disabled={task?.planning_status === 'planning_complete'} />
      </div>
    </div>

    {#if task?.planning_status !== 'planning_complete'}
      <button type="submit" class="submit-plan-btn">Mimari Planı Gönder & Kilidi Kaldır</button>
    {/if}
  </form>
</div>

<style>
  .planning-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .success-alert {
    background: #1a332d;
    border-left: 4px solid #4ec9b0;
    color: #4ec9b0;
    padding: 10px;
    margin-bottom: 15px;
    font-size: 0.8rem;
    font-weight: bold;
  }
  .warning-alert {
    background: #3c2419;
    border-left: 4px solid #ce9178;
    color: #ce9178;
    padding: 10px;
    margin-bottom: 15px;
    font-size: 0.8rem;
    font-weight: bold;
  }
  .plan-form {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field.wide { grid-column: 1 / -1; }
  label { font-size: 0.7rem; color: #888; text-transform: uppercase; }
  input, select {
    padding: 6px;
    background: #252526;
    border: 1px solid #2d2d2d;
    color: white;
    font-size: 0.8rem;
    border-radius: 4px;
  }
  input:disabled, select:disabled {
    background: #1a1a1a;
    color: #888;
    border-color: #222;
  }
  .submit-plan-btn {
    padding: 10px;
    background: #ce9178;
    color: #1e1e1e;
    font-weight: bold;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .submit-plan-btn:hover { background: #dfa289; }
</style>
