<script lang="ts">
  let { onCreate: _onCreate } = $props<{
    onCreate?: (title: string, userRequest: string) => void;
  }>();
</script>

<div class="intake-panel-container">
  <div class="intake-header">
    <h2>Görev Başlatma Yönlendirmesi</h2>
    <p>Operasyon akışı sıralıdır. Görev atama yalnızca Panel 1 üzerinden yapılır.</p>
  </div>

  <div class="redirect-card">
    <ol>
      <li>Üstteki <strong>Panel 1 · Kurucu</strong> bölümüne gidin.</li>
      <li>Görevi yazılı veya sesli olarak Albay Burhan'a atayın.</li>
      <li>Görev atandıktan sonra Panel 2, 3 ve 4 otomatik açılır.</li>
      <li>Planlama tamamlandıktan sonra operasyon başlatılabilir.</li>
    </ol>
    <p class="note">Intake paneli devre dışı bırakıldı. Akış dışı görev oluşturma engellendi.</p>
  </div>
</div>

<style>
  .intake-panel-container {
    padding: 18px;
    border: 1px solid #2f3a3a;
    border-radius: 10px;
    background: #14181a;
    color: #dce5ea;
    margin-bottom: 16px;
  }
  .intake-header h2 { margin: 0 0 6px; color: #f2f6f8; }
  .intake-header p { margin: 0 0 12px; color: #9eb0b8; font-size: 0.9rem; }
  .redirect-card {
    background: #101416;
    border: 1px solid #2a3438;
    border-radius: 8px;
    padding: 14px;
  }
  ol { margin: 0; padding-left: 1.2rem; }
  li { margin-bottom: 8px; line-height: 1.45; }
  .note {
    margin: 12px 0 0;
    font-size: 0.84rem;
    color: #8ec9c9;
    font-weight: 600;
  }
</style>
