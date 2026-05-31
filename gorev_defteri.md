# LOKAL BİLGİSAYAR KONTROL PANELİ - GÖREV DEFTERİ (GÜNCEL)

**Tarih:** 2026-05-30  
**Mod:** Kurucu Engin + Auto (tek ekip)

---

## Durum özeti

| Faz | Konu | Kod kanıtı | Durum |
|-----|------|------------|-------|
| KN-01 | Connector icra (`action_executor.rs`) | `execution_engine` dispatch | **TAMAMLANDI** |
| KN-02 | AI provider çağrısı | `ai_provider_invoke.rs` | **TAMAMLANDI** |
| KN-03 | Failover | `config/failover_policy.json` + `select_with_failover` | **TAMAMLANDI** |
| KN-04 | Swarm workflow | `ai_workflow_manager.rs` + inbox dosyaları | **TAMAMLANDI** |
| KN-05 | Intake tahsis | `task_intake.rs` → allocate | **TAMAMLANDI** |
| KN-06 | Supabase sync | `storage/supabase_sync.rs` (env gerekli) | **TAMAMLANDI** (env yoksa atlar) |
| KN-07 | Log rotasyon | `storage/log_rotation.rs` + startup | **TAMAMLANDI** |
| KN-08 | Asker köprü | `asker_motoru_bridge.rs` + UI | **TAMAMLANDI** |
| KN-09 | Önizleme banner | `+page.svelte` runtime banner | **ZATEN VARDI / GÜNCELLENDİ** |
| KN-10 | Bu dosya düzeltildi | — | **TAMAMLANDI** |
| KN-11 | `src/lib/runtime.ts` ayrıştırma | kısmi | **KISMI** |
| KN-12 | E2E test | `tests/e2e_workflow_test.rs` | **TAMAMLANDI** |
| KN-13 | StructuredReportPanel | `StructuredReportPanel.svelte` | **TAMAMLANDI** |
| KN-14 | Canlı API kurulum notu | `config/LIVE_SITE_API_KURULUM.md` | **TAMAMLANDI** |

> Eski FAZ 1–2 “SUCCESS (failover/supabase tam)” ifadeleri **kod olmadan** yazılmıştı; bu sürümde gerçek modüller eklendi.

---

## Doğrulama

```powershell
cd src-tauri
cargo test
```
