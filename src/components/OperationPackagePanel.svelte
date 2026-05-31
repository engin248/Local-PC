<script lang="ts">
  let { packages = [] } = $props<{
    packages: any[];
  }>();

  function parseList(value: string) {
    try {
      const parsed = JSON.parse(value || "[]");
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return value ? [value] : [];
    }
  }
</script>

<div class="package-panel-container">
  <h3>ALT BIRIM OPERASYON PAKETLERI</h3>

  {#if packages.length > 0}
    <div class="package-list">
      {#each packages as pkg}
        <section class="package-card">
          <div class="package-header">
            <div>
              <span class="package-type">{pkg.package_type}</span>
              <h4>{pkg.subject}</h4>
            </div>
            <span class="status">{pkg.status}</span>
          </div>

          <div class="detail-grid">
            <div><strong>Alt konu:</strong> {pkg.sub_topic}</div>
            <div><strong>Kriter:</strong> {pkg.criterion}</div>
            <div><strong>Alt kriter:</strong> {pkg.sub_criterion}</div>
            <div><strong>Teknoloji:</strong> {pkg.technology}</div>
            <div><strong>Etki alani:</strong> {pkg.impact_area}</div>
            <div><strong>Kontrol noktasi:</strong> {pkg.control_point}</div>
          </div>

          <div class="reason-box">
            <strong>Kabul edilen dogru:</strong>
            <p>{pkg.accepted_truth}</p>
          </div>

          <div class="reason-box selected">
            <strong>Secilen en iyi alternatif:</strong>
            <p>{pkg.selected_best_alternative}</p>
          </div>

          <div class="lists">
            <div>
              <strong>Islem sirasi</strong>
              <ol>
                {#each parseList(pkg.operation_sequence) as step}
                  <li>{step}</li>
                {/each}
              </ol>
            </div>
            <div>
              <strong>Kontrol kriterleri</strong>
              <ul>
                {#each parseList(pkg.control_criteria) as criterion}
                  <li>{criterion}</li>
                {/each}
              </ul>
            </div>
            <div>
              <strong>Test plani</strong>
              <ul>
                {#each parseList(pkg.test_plan) as test}
                  <li>{test}</li>
                {/each}
              </ul>
            </div>
          </div>

          <div class="role-grid">
            <span>Yapan: <b>{pkg.executor_role}</b></span>
            <span>Dogru yaptiran: <b>{pkg.correctness_guard_role}</b></span>
            <span>Kontrol eden: <b>{pkg.controller_role}</b></span>
            <span>Bagimsiz dogrulayan: <b>{pkg.independent_verifier_role}</b></span>
            <span>Son onay: <b>{pkg.final_approver_role}</b></span>
          </div>
        </section>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Plan kilidi tamamlaninca alt birim operasyon paketi burada gorunur.</p>
  {/if}
</div>

<style>
  .package-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .package-list { display: flex; flex-direction: column; gap: 12px; }
  .package-card {
    background: #111112;
    border: 1px solid #2d2d31;
    border-left: 4px solid #f59e0b;
    border-radius: 6px;
    padding: 14px;
  }
  .package-header {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
    border-bottom: 1px solid #2d2d31;
    padding-bottom: 10px;
    margin-bottom: 10px;
  }
  .package-type { font-size: 0.68rem; color: #f59e0b; text-transform: uppercase; font-weight: 700; }
  h4 { margin: 3px 0 0 0; color: #fff; font-size: 0.95rem; }
  .status { background: #1f352b; color: #47d18c; font-size: 0.68rem; padding: 4px 7px; border-radius: 4px; text-transform: uppercase; font-weight: 700; }
  .detail-grid, .role-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 8px;
    font-size: 0.76rem;
  }
  .reason-box {
    margin-top: 10px;
    padding: 8px;
    background: #18181a;
    border-radius: 4px;
    font-size: 0.76rem;
  }
  .reason-box.selected { border-left: 3px solid #47d18c; }
  .reason-box p { margin: 4px 0 0 0; color: #f4f4f5; line-height: 1.4; }
  .lists {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 10px;
    margin-top: 10px;
    font-size: 0.75rem;
  }
  ol, ul { margin: 6px 0 0 18px; padding: 0; }
  li { margin-bottom: 4px; }
  .role-grid {
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid #2d2d31;
  }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
