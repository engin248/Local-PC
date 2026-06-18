# Silinenler Denetimi — 2026-06-11

**Amaç:** Son temizlikte ne silindi, ne geri konmalı, ne kurulmalı?

---

## A. Bilinçli silinenler (geri konmaz)

| Ne silindi | Neden | Karar |
|------------|-------|-------|
| UNO satırları `EKSIKLER_TABLOSU.md` | Geçersiz soru | **Doğru** — geri konmaz |
| UNO maddesi `emel_sesli_ozet.txt` | Yanlış yönlendirme | **Doğru** |
| "Sizden sorulan 3 madde" (UNO dahil) | Gereksiz | **Doğru** |

`UNO_ACIKLAMA.txt` ve `RAPOR_UNO_CANLI_INCELEME.md` **arşiv** olarak duruyor — kod değil, rapor.

---

## B. Yanlışlıkla kısalan (geri kondu)

| Ne eksilmişti | Durum |
|---------------|-------|
| UZMAN/skill_library disk tablosu | **Geri yazıldı** `EKSIKLER_TABLOSU_CEVAPLI.md` |
| 20 soru tam listesi | **Geri yazıldı** (UNO yok) |
| Tablo satır numaraları (2→4 atlaması) | **Düzeltildi** `EKSIKLER_TABLOSU.md` |

---

## C. Eski git silmeleri (sorun yok)

| Dosya | Ne oldu |
|-------|---------|
| `CommandCenterPanel.svelte` | `KontrolDepartmaniPanel.svelte` ile değişti — import OK |
| `audit_package/.../system_connectors.json` | Eski yedek — gerek yok |

---

## D. Henüz kurulması gereken (silinmedi — sizin PC)

| Madde | Durum | Ne yapılır |
|-------|-------|------------|
| Kurulu `.exe` güncel mi | Bilinmiyor | `KURULU_SURUMU_GUNCELLE.cmd` |
| `UZMAN_HAVUZU.json` | Config yolu var, dosya sizin diskte | Asker Motoru açıkken panel okur |
| `skill_library.sqlite` | Lokal Kütüphane yolunda | Env veya varsayılan yol |
| Asker canlı API | `enabled: false` | İsterseniz `asker_motoru_bridge.json` → `true` |
| AI provider | `enabled: false` | `config/ai_providers.json` + env key |

**GitHub push gerekmez.**

---

## Test — Analiz — Onay

| Adım | Sonuç |
|------|-------|
| Kod silmesi | Yok — yalnızca rapor satırları |
| Import kırığı | Yok |
| Geri yükleme | Tablolar düzeltildi |
| Onay | Silinenler denetlendi; kurulum listesi net |
