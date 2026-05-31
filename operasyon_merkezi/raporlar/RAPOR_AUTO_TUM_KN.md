# RAPOR — Auto (KN-01 … KN-14)

**Tarih:** 2026-05-30  
**Durum:** TAMAMLANDI (KN-11 kısmi refactor)

---

## Özet

Kalan %30 maddelerin tamamı sırayla uygulandı. Her madde öncesi kod tabanı kontrol edildi; sonrasında `cargo test` ile doğrulama yapıldı.

---

## Madde madde

| ID | Sonuç | Kanıt dosyası |
|----|-------|----------------|
| KN-01 | OK | `src-tauri/src/core/action_executor.rs` |
| KN-02 | OK | `src-tauri/src/ai_providers/ai_provider_invoke.rs` |
| KN-03 | OK | `config/failover_policy.json` |
| KN-04 | OK | `src-tauri/src/core/ai_workflow_manager.rs` |
| KN-05 | OK | `task_intake.rs` allocate çağrısı |
| KN-06 | OK | `storage/supabase_sync.rs` |
| KN-07 | OK | `storage/log_rotation.rs` |
| KN-08 | OK | `asker_motoru_bridge.rs` + UI |
| KN-09 | OK | runtime banner |
| KN-10 | OK | `gorev_defteri.md` |
| KN-11 | KISMI | `src/lib/runtime.ts` |
| KN-12 | OK | `tests/e2e_workflow_test.rs` |
| KN-13 | OK | `StructuredReportPanel.svelte` |
| KN-14 | OK | `LIVE_SITE_API_KURULUM.md` |

---

## Kontrol noktaları (KN-01)

- KP-01: `action_execute` / `action_execute_skipped` event types — **OK**
- KP-02: read-only write skip — **OK** (unit test)
- KP-03: onaylı write fail-closed log — **OK**

---

## Notlar

- Supabase: `SUPABASE_URL` + key yoksa sync bilinçli atlanır.
- AI HTTP: provider `enabled: true` + env key gerekir; aksi halde yerel stub.
- KN-11 tam `+page.svelte` bölme sonraki sprintte sürdürülebilir.
