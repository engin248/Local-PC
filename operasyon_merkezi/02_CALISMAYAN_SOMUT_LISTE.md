# Çalışmayan Maddeler — Somut Liste ve Kontrol Noktaları

**Mod:** Kurucu Engin + Auto (tek ekip)  
**Klasör:** `operasyon_merkezi`  
**Tarih:** 2026-05-30  
**Kural:** Her madde bitmeden sonrakine geçilmez. Her maddede **KP** = kontrol noktası, **KK** = kontrol kriteri.

---

## Özet tablo

**Not (2026-06-11):** Aşağıdaki özet `kontrol/KONTROL_DURUMU.md` ile eşitlendi. Alt bölümlerdeki eski “BAŞLANACAK” cümleleri tarihseldir; kod durumu için **KONTROL_DURUMU** esas alınır.

| Sıra | ID | Çalışmayan ne? | Durum |
|------|-----|----------------|-------|
| 1 | KN-01 | Gate sonrası connector icrası | **ONAYLANDI** (URETIM-01 ApprovedExecution) |
| 2 | KN-02 | AI provider API | **ONAYLANDI** (enabled=false varsayılan) |
| 3 | KN-03 | AI failover | **ONAYLANDI** |
| 4 | KN-04 | Swarm inbox/outbox | **ONAYLANDI** (+ sync_outbox_reports) |
| 5 | KN-05 | Intake tahsis | **ONAYLANDI** |
| 6 | KN-06 | Supabase sync | **ONAYLANDI** (env yok=kapalı) |
| 7 | KN-07 | Log rotasyonu | **ONAYLANDI** |
| 8 | KN-08 | Asker köprü | **ONAYLANDI** (dosya; canlı API kapalı) |
| 9 | KN-09 | Tarayıcı sahte veri | **ONAYLANDI** |
| 10 | KN-10 | gorev_defteri | **ONAYLANDI** |
| 11 | KN-11 | Büyük dosyalar | **KISMI** |
| 12 | KN-12 | E2E test | **ONAYLANDI** |
| 13 | KN-13 | StructuredReportPanel | **ONAYLANDI** |
| 14 | KN-14 | Canlı API connector | **ONAYLANDI** (bilinçli kapalı) |

---

## KN-01 — Connector icra katmanı (KRİTİK)

**Sorun:** 8 kapı geçiliyor; `file_connector`, `terminal_connector` vb. pipeline’a bağlı değil. `execute_task_pipeline` her zaman `RunMode::ReadOnly`.

**Yapılacak iş:**
1. `action_executor` modülü (gate onayı sonrası dispatch)
2. Onaylı context’te `ApprovedExecution` modu
3. En az `read_file` + onaylı `write_file` tek düğümde uçtan uca

**KP-01:** `execution_engine.rs` içinde gate zinciri bittikten sonra `ActionExecutor::dispatch` çağrılıyor mu?  
**KK-01:** `execution_logs` tablosunda `event_type = action_execute` kaydı var.

**KP-02:** Onaysız write denemesi.  
**KK-02:** İşlem fail-closed; dosya değişmiyor; logda `read_only_blocked` veya approval hatası.

**KP-03:** 2 approver onaylı write.  
**KK-03:** Hedef dosya yazılıyor; öncesinde snapshot kaydı var.

**Test:** `cargo test --lib` → 39+ test yeşil.

**Dosyalar:** `src-tauri/src/core/execution_engine.rs`, yeni `src-tauri/src/core/action_executor.rs`, `system_connectors/*`

---

## KN-02 — AI provider gerçek çağrı

**Sorun:** Adapter dosyaları var; pipeline Statement/Execution aşamasında API çağrısı yok.

**KP-01:** `enabled: true` + env key ile health `available`.  
**KK-01:** TCP ping + (opsiyonel) minimal chat completion yanıtı loga düşer.

**KP-02:** Statement Gate sonrası provider çağrısı.  
**KK-02:** `statements` tablosuna provider yanıtı veya hata kanıtı yazılır.

**Dosyalar:** `ai_providers/openai_compatible_provider.rs`, `gemini_provider.rs`, `execution_engine.rs`

---

## KN-03 — AI failover

**Sorun:** `failover_policy.json` yok; health sadece raporlar.

**KP-01:** Primary `connection_failed` olunca backup seçilir.  
**KK-01:** `execution_logs` veya audit’te `failover_route` kaydı; süre < 5 sn (ilk sürüm).

**Dosyalar:** `config/failover_policy.json`, `ai_provider_manager.rs`

---

## KN-04 — Swarm workflow

**Sorun:** `ai_workflow/` boş; `ai_tasks` tabloları UI/komut yok.

**KP-01:** Görev oluşturulunca platform inbox’a JSON düşer.  
**KK-01:** `ai_task_allocations` satırı + diskte dosya var.

**KP-02:** Outbox raporu toplanır.  
**KK-02:** `ai_collected_reports` + `verified` alanı güncellenir.

**Dosyalar:** yeni `core/ai_workflow_manager.rs`, `lib.rs` komutları, `SwarmMonitorPanel.svelte` (opsiyonel)

---

## KN-05 — Intake → gerçek tahsis

**Sorun:** `IntakePanel` sadece `[Ajanlar: CODEX,OAM]` metni ekliyor.

**KP-01:** Checkbox → allocation engine.  
**KK-01:** Seçilen platformlar için KN-04 inbox dosyası oluşur.

**Dosya:** `src/components/IntakePanel.svelte`, `task_intake.rs`

---

## KN-06 — Supabase sync

**Sorun:** Kod yok; `gorev_defteri` SUCCESS yazıyor (yanlış).

**KP-01:** Env: `SUPABASE_URL` + key tanımlı.  
**KK-01:** `tasks` veya `execution_logs` push sonrası bulutta satır görünür.

**Dosya:** yeni `storage/supabase_sync.rs`

---

## KN-07 — Log rotasyonu

**Sorun:** SQLite sınırsız büyüyebilir.

**KP-01:** `app.db` > 50 MB veya log satır > eşik.  
**KK-01:** Eski loglar arşiv klasörüne; DB küçülür veya sabit kalır.

**Dosya:** `storage/db.rs`

---

## KN-08 — Asker Motoru köprü

**Sorun:** Parite script var; panel okumuyor.

**KP-01:** Panel “Bağlantılar” veya yeni sekmede JSON status.  
**KK-01:** `PLANLAMA_DURUMU.json` benzeri dosya okunup UI’da gösterilir.

**Dosyalar:** `scripts/verify_asker_motoru_parity.ps1`, yeni Tauri komutu

---

## KN-09 — Tarayıcı önizleme

**Sorun:** `npm run dev` mock veri; kapılar gerçek değil.

**KP-01:** Production build Tauri zorunlu uyarısı UI’da.  
**KK-01:** DEV modda banner: “Önizleme — gerçek DB değil”.

**Dosya:** `src/routes/+page.svelte`

---

## KN-10 — Dokümantasyon düzeltme

**Sorun:** `gorev_defteri.md` FAZ 1–2 SUCCESS ama kod yok.

**KP-01:** Her SUCCESS maddesinde dosya + test referansı.  
**KK-01:** KN-06/03 tamamlanmadan SUCCESS yazılmaz.

**Dosya:** `gorev_defteri.md`

---

## KN-11 — Mimari refactor (600 satır)

**Sorun:** `execution_engine.rs` ~1020, `+page.svelte` ~1116 satır.

**KP-01:** Bölme sonrası satır sayısı.  
**KK-01:** Hiçbir yeni dosya > 600 satır; `cargo test` yeşil.

**Sıra:** UI-01 → VAL → DEP → EXE (geçmiş sprint planı)

---

## KN-12 — E2E test

**KP-01:** Tauri driver veya script ile görev oluştur → plan → execute.  
**KK-01:** Otomatik senaryo CI’da yeşil.

---

## KN-13 — StructuredReportPanel

**KP-01:** Rapor sekmesinde 3 bölüm zorunlu.  
**KK-01:** Çözümleme / plan / izleme boş bırakılamaz (integrity).

---

## KN-14 — Canlı API (düşük öncelik)

**KP-01:** `LIVE_SITE_API_KEY` + `base_url` set.  
**KK-01:** Connector health `available`.

---

## Çalışma sırası (sadece sen + Auto)

```
KN-01 → KN-02 → KN-03 → KN-05 → KN-04 → KN-07 → KN-06 → KN-08 → KN-09 → KN-10 → KN-11 → KN-12 → KN-13 → KN-14
```

**Şu an aktif:** KN-01

**Rapor dosyası (Auto):** `raporlar/RAPOR_AUTO_KN01.md`  
**Kontrol güncelleme:** `kontrol/KONTROL_DURUMU.md`
