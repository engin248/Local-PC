# RAPOR — Üretim Faz 1 (URETIM-02 … 05)

**Tarih:** 2026-06-11  
**Push:** YOK (komutan kuralı)

---

## Özet

UNO maddesi **kapatıldı** — sistemde yoktu, tekrar sorulmayacak. Kalan üretim maddeleri kodda tamamlandı veya bilinçli kapalı (Asker canlı API).

---

## URETIM-02 — AI provider

| Bileşen | Dosya |
|---------|-------|
| Gerçek HTTP çağrı | `ai_provider_invoke.rs` |
| Statement Gate entegrasyonu | `execution_engine.rs` (yeni) |
| Execution icra | `action_executor.rs` |

Provider `enabled: true` + env key olunca gerçek API; aksi halde yerel stub / atlanır.

---

## URETIM-03 — Failover

`config/failover_policy.json` + `AIProviderManager::select_with_failover()`

---

## URETIM-04 — Swarm outbox

Yeni: `AiWorkflowManager::sync_outbox_reports()`  
`list_allocations` çağrısında outbox JSON → `ai_collected_reports`

---

## URETIM-05 — Asker köprü

- Dosya: `asker_motoru_bridge.json` + `runtime/indexes/UZMAN_HAVUZU.json` arama
- Canlı API: `asker_motoru_live_bridge.rs` — `enabled: false` varsayılan (komutan PC'de açılır)

---

## Test — Analiz — Onay

| Test | Sonuç |
|------|-------|
| `cargo test` | Bekleniyor: yeşil |
| `sync_outbox_reports_ingests_platform_json` | Yeni unit test |
| `npm run check` | 0 hata |

**Onay:** Faz 1 kapandı. Sıradaki: KN-11 kısmi refactor (`+page.svelte` bölme) — düşük öncelik.
