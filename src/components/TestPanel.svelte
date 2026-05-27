<script lang="ts">
  let { tests = [] } = $props<{
    tests: any[];
  }>();
</script>

<div class="test-panel-container">
  <h3>ÇIKTI VE KOD DOĞRULAMA TESTLERİ</h3>

  {#if tests.length > 0}
    <div class="tests-list">
      {#each tests as test}
        <div class="test-card" class:passed={test.status === 'passed'}>
          <div class="test-header">
            <span class="test-name">Test: {test.test_name}</span>
            <span class="status-badge {test.status}">{test.status.toUpperCase()}</span>
          </div>
          <div class="test-body">
            <div class="test-row">
              <span class="label">Beklenen Çıktı</span>
              <pre class="value">{test.expected_result}</pre>
            </div>
            <div class="test-row">
              <span class="label">Alınan Çıktı</span>
              <pre class="value">{test.actual_result}</pre>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="empty-msg">Doğrulama testleri henüz çalıştırılmadı.</p>
  {/if}
</div>

<style>
  .test-panel-container {
    padding: 15px;
    background: #181818;
    color: #ccc;
    border-bottom: 1px solid #333;
  }
  h3 { margin: 0 0 12px 0; font-size: 0.85rem; letter-spacing: 1px; color: #e0e0e0; }
  .tests-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .test-card {
    background: #252526;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 10px;
  }
  .test-card.passed {
    border-color: #2e4d3a;
  }
  .test-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #333;
    padding-bottom: 5px;
    margin-bottom: 8px;
  }
  .test-name { font-weight: bold; font-size: 0.75rem; color: #eee; }
  .status-badge {
    font-size: 0.65rem;
    font-weight: bold;
    padding: 2px 6px;
    border-radius: 3px;
  }
  .status-badge.passed { background: #4ec9b0; color: #1e1e1e; }
  .status-badge.failed { background: #f44747; color: white; }

  .test-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .test-row {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .test-row .label { font-size: 0.65rem; color: #888; text-transform: uppercase; }
  .test-row pre {
    margin: 0;
    padding: 6px;
    background: #1e1e1e;
    border: 1px solid #2d2d2d;
    color: #ce9178;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.75rem;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .empty-msg { text-align: center; color: #666; font-size: 0.8rem; padding: 20px 0; }
</style>
