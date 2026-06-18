# 20 SORU — SİSTEMDEN CEVAPLANMIŞ TABLO

**Kural:** Önce kod ve config okundu. Sizden yalnızca bulut/uzak erişimle öğrenilemeyenler sorulur.

---

## UZMAN HAVUZU ve skill_library — NEDEN DİSKTE?

**GitHub'da olmak zorunda DEĞİL.**

Panel Rust kodu dosyayı **yerel diskten okur** (`fs::read_to_string`, SQLite `open`):

| Dosya | Kod nerede bakar | Neden disk |
|-------|------------------|------------|
| `UZMAN_HAVUZU.json` | `asker_module_registry.rs` → Asker kök klasörleri | 314 modül envanteri; büyük harici veri; repoya gömülmez |
| `skill_library.sqlite` | `skill_library.json` → `SKILL_LIBRARY_DB_PATH` veya `C:\Users\Esisya\Desktop\Lokal Kütüphane\database\skill_library.sqlite` | Beceri tablosu SQLite; veritabanı dosyası olarak durur |

**Özet:** DIX = disk. Git değil. Panel çalışırken o yoldan **okuma** yapar. Sizin Lokal Kütüphane / Asker Motoru klasörünüz veri deposudur.

`asker_motoru_bridge.json` içinde `windows_root` şu an **boş** — env `ASKER_MOTORU_WINDOWS_ROOT` veya config'e yol yazılmazsa UZMAN dosyası bulunamaz.

---

## 20 SORU CEVAPLARI (koddan / bulut testinden)

| # | Madde | Cevap (sistemden) |
|---|--------|-------------------|
| 1 | UNO sistemi | **YOK** — panel kodunda UNO diye bir şey yok |
| 2 | Kurulu Windows .exe güncel mi | **Buluttan bilinmez** → sizin PC |
| 3 | Yarbay Emel sesi kodda | **VAR** — bootstrap düğmesi gerekir |
| 4 | Alarm düzeltmeleri kodda | **VAR** — master'da |
| 5 | ACİL SES KES kodda | **VAR** |
| 6 | SQLite app.db | **VAR** — `storage/app.db` |
| 7 | UZMAN_HAVUZU.json | **Bu VM'de YOK** — config Asker kökü/env bekliyor; sizin diskte aranır |
| 8 | skill_library.sqlite | **Bu VM'de YOK** — varsayılan: `Desktop\Lokal Kütüphane\database\` |
| 9 | Pinokio | **KAPALI** — :42000 yanıt yok |
| 10 | Asker Motoru canlı API | **KAPALI** — config `enabled: false` |
| 11 | AI API | **KAPALI** — tüm provider `enabled: false` |
| 12 | Supabase | **KAPALI** — env yok |
| 13 | Albay Burhan servisi | **Dosya var** — `services/burhan_orchestrator/`; süreç çalışıyor mu bilinmez |
| 14 | Tam üretim yazma | **KISMI** — varsayılan ReadOnly |
| 15 | Kontrol Departmanı | **VAR** |
| 16 | Sizden git commit/push | **GEREKMEZ** |
| 17 | Sizden git pull | **GEREKMEZ** (zorunlu haller dışında) |
| 18 | Cloud Agent | **ÇALIŞIYOR** |
| 19 | Testler | **GEÇİYOR** — 51 test |
| 20 | Cursor sohbet sesi | **YOK** — Joshua sesi panelde yok; OpenClaw'da başka kişi |

---

## Joshua / sohbet sesi

Panel ve Cursor sohbetinde **"Joshua" sesi tanımlı değil.**  
OpenClaw CHANGELOG'da JoshuaLelon = katkıcı isim; sizin Emel/TTS ile ilgisi yok.  
**Cursor bu sohbeti sesli okuyamaz.** Ses: panel `Emel'i Başlat` veya `SESLI_OZET_OKU.cmd` (yerel).

---

## SİZDEN SORULAN (yalnızca 3 madde)

1. Kurulu `lokal_bilgisayar_kontrol_paneli.exe` güncellediniz mi? (evet/hayır)
2. `UZMAN_HAVUZU.json` tam yolu sizin PC'de nerede? (Asker Motoru / Lokal Kütüphane?)
3. UNO ile kastettiğiniz tam isim ne? (panelde yok)

GitHub push/pull size **zorunlu değil**. İş yerel depoda.
