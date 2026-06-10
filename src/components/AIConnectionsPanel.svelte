<script lang="ts">
  let {
    providers = [],
    onRefresh,
    onTestProvider
  }: {
    providers: any[];
    onRefresh: () => void;
    onTestProvider: (providerId: string, endpoint?: string) => void;
  } = $props();

  let endpointOverrides = $state<Record<string, string>>({});

  function providerBadge(provider: any) {
    if (provider.access_type === "local") return "local / ücretsiz";
    if (provider.api_key_required) return "free-tier / API key gerekli";
    return provider.optional_provider ? "opsiyonel" : "ücretsiz";
  }

  function visibleModels(provider: any) {
    const models = Array.isArray(provider.model_list) ? provider.model_list : [];
    return models.length > 0 ? models.slice(0, 4).join(", ") : provider.model || "model yok";
  }

  function endpointValue(provider: any) {
    return endpointOverrides[provider.id] ?? provider.endpoint ?? "";
  }
</script>

<section class="connection-panel">
  <div class="panel-header">
    <div>
      <span class="eyebrow">YAPAY ZEKA BAĞLANTILARI</span>
      <h3>AI Sağlayıcıları</h3>
    </div>
    <button type="button" onclick={onRefresh}>Tümünü tara</button>
  </div>

  <div class="connection-grid">
    {#each providers as provider (provider.id)}
      <article class="connection-row">
        <div class="provider-title">
          <strong>{provider.name}</strong>
          <span>{provider.provider_type} / {providerBadge(provider)}</span>
        </div>
        <div>
          <span class="label">Durum</span>
          <b
            class:ok={provider.status === "available"}
            class:warn={provider.status !== "available"}
          >
            {provider.status}
          </b>
        </div>
        <div>
          <span class="label">API Key</span>
          <b>{provider.api_key_required ? provider.api_key_status : "gerekmez"}</b>
        </div>
        <div>
          <span class="label">Enabled</span>
          <b>{provider.enabled ? "açık" : "kapalı"}</b>
        </div>
        <div>
          <span class="label">Bağımlılık</span>
          <b>{provider.dependency_level}</b>
        </div>
        <div class="wide">
          <span class="label">Endpoint</span>
          <div class="endpoint-row">
            <input
              aria-label={`${provider.name} endpoint`}
              value={endpointValue(provider)}
              oninput={(event) => endpointOverrides[provider.id] = event.currentTarget.value}
            />
            <button
              type="button"
              class="test-button"
              onclick={() => onTestProvider(provider.id, endpointValue(provider))}
            >
              Test
            </button>
          </div>
        </div>
        <div class="wide">
          <span class="label">Model listesi</span>
          <code>{visibleModels(provider)}</code>
        </div>
        <div class="wide">
          <span class="label">Bağlantı sonucu</span>
          <span>{provider.connection_result || "Henüz test sonucu yok."}</span>
        </div>
        <p>{provider.error_message || provider.last_error || "Son hata yok."}</p>
      </article>
    {/each}
  </div>
</section>

<style>
  .connection-panel {
    padding: 18px;
    border: 1px solid #2a2a2d;
    background: #18181a;
    border-radius: 6px;
    margin-bottom: 16px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: center;
    margin-bottom: 14px;
  }

  .eyebrow,
  .label {
    display: block;
    color: #8d8d95;
    font-size: 11px;
    text-transform: uppercase;
  }

  h3 {
    margin: 4px 0 0;
    color: #f2f2f4;
    font-size: 18px;
  }

  button {
    background: #0b74de;
    color: white;
    border: 0;
    border-radius: 6px;
    padding: 9px 12px;
    cursor: pointer;
  }

  .connection-grid {
    display: grid;
    gap: 10px;
  }

  .connection-row {
    display: grid;
    grid-template-columns: minmax(180px, 1.6fr) repeat(4, minmax(90px, 0.8fr));
    gap: 12px;
    align-items: center;
    padding: 12px;
    background: #111113;
    border: 1px solid #2d2d31;
    border-radius: 6px;
  }

  .provider-title span {
    display: block;
    margin-top: 4px;
  }

  .wide {
    grid-column: 1 / -1;
  }

  .endpoint-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    background: #0f0f11;
    border: 1px solid #39393f;
    border-radius: 5px;
    color: #e9e9ec;
    padding: 8px;
  }

  .test-button {
    background: #2c8f5b;
  }

  strong,
  b {
    color: #f4f4f5;
  }

  span,
  p,
  code {
    color: #b8b8bf;
    margin: 0;
  }

  code {
    overflow-wrap: anywhere;
  }

  p {
    grid-column: 1 / -1;
    font-size: 12px;
  }

  .ok {
    color: #47d18c;
  }

  .warn {
    color: #f8c14a;
  }
</style>
