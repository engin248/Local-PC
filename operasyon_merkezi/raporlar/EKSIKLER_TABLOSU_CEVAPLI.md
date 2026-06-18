# 20 SORU — SİSTEMDEN CEVAPLANMIŞ TABLO

**Kural:** Önce kod ve config okundu. Komutana yalnızca PC'den öğrenilemeyenler sorulur.

---

## UZMAN HAVUZU ve skill_library — NEDEN DİSKTE?

**GitHub'da olmak zorunda değil.** Panel yerel diskten okur:

| Dosya | Kod nerede bakar | Neden disk |
|-------|------------------|------------|
| `UZMAN_HAVUZU.json` | `asker_module_registry.rs` → `runtime/indexes/` | 314 modül envanteri |
| `skill_library.sqlite` | `skill_library.json` → Lokal Kütüphane | Beceri SQLite |

Varsayılan Asker kökü: `config/asker_motoru_bridge.json` → `Desktop\asker motoru`

---

## 20 SORU CEVAPLARI

| # | Madde | Cevap |
|---|--------|-------|
| 1 | Kurulu Windows .exe güncel mi | Buluttan bilinmez — sizin PC |
| 2 | Yarbay Emel sesi | VAR — Emel'i Başlat |
| 3 | Alarm / ACİL SES KES | Kodda VAR |
| 4 | SQLite app.db | VAR |
| 5 | UZMAN_HAVUZU.json | Planlama / Asker Motoru diskte |
| 6 | skill_library.sqlite | Lokal Kütüphane diskte |
| 7 | Pinokio | Kapalı |
| 8 | Asker canlı API | Kapalı (`enabled: false`) |
| 9 | AI API | Kapalı (provider `enabled: false`) |
| 10 | Supabase | Kapalı (env yok) |
| 11 | Üretim yazma | URETIM-01 tamam |
| 12 | Swarm outbox | URETIM-04 `sync_outbox_reports` |
| 13 | Failover | `failover_policy.json` aktif |
| 14 | Kontrol Departmanı | VAR |
| 15 | Git commit/push sizden | GEREKMEZ |
| 16 | git pull sizden | GEREKMEZ |
| 17 | Cloud Agent | Çalışıyor |
| 18 | Testler | 55 geçiyor |
| 19 | Cursor sohbet sesi | YOK — Emel/panel/script |
| 20 | Kurulu exe rehberi | `kurulum/KURULU_EXE_NEREDEN_GUNCELLENIR.md` |

---

## Ses

Cursor sohbeti sesli okuyamaz. Panel: **Emel'i Başlat** veya `SESLI_OZET_OKU.cmd`.

GitHub push/pull komutan onayı olmadan yapılmaz.
