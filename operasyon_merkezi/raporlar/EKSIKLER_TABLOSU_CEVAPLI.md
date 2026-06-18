# 20 SORU — SİSTEMDEN CEVAPLANMIŞ TABLO

**Kural:** Önce kod ve config okundu. Komutana yalnızca PC'den öğrenilemeyenler sorulur.

---

## UZMAN HAVUZU — neden diskte?

Panel dosyayı **yerel diskten** okur (`asker motoru\runtime\indexes\UZMAN_HAVUZU.json`). GitHub zorunlu değil.

---

## 20 SORU CEVAPLARI

| # | Madde | Cevap |
|---|--------|-------|
| 1 | Kurulu Windows .exe güncel mi | Buluttan bilinmez — sizin PC |
| 2 | Yarbay Emel sesi | VAR — Emel'i Başlat düğmesi |
| 3 | Alarm / ACİL SES KES | Kodda VAR |
| 4 | SQLite app.db | VAR |
| 5 | UZMAN_HAVUZU.json | Asker Motoru Planlama — diskte aranır |
| 6 | skill_library.sqlite | Lokal Kütüphane — diskte |
| 7 | Pinokio | Kapalı |
| 8 | Asker canlı API | Kapalı (dosya köprüsü aktif) |
| 9 | AI API | Kapalı (enabled: false) |
| 10 | Supabase | Kapalı (env yok) |
| 11 | Üretim yazma | URETIM-01 tamam — onay sonrası ApprovedExecution |
| 12 | Swarm outbox | URETIM-04 — sync_outbox_reports eklendi |
| 13 | Git commit/push sizden | GEREKMEZ |
| 14 | Testler | GEÇİYOR |

---

## Ses

Cursor sohbeti sesli okuyamaz. Panel: **Emel'i Başlat** veya `SESLI_OZET_OKU.cmd`.

GitHub push/pull komutan onayı olmadan yapılmaz.
