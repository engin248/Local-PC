# Başlangıç — Sıfır durum (gcloud kapalı, yerel only)

**Durum:** Google Cloud / uzak agent yok. Her şey **sizin Windows PC** ve **yerel klasör**.

---

## 1. Proje klasörü

```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli
```

Cursor **masaüstü uygulaması** → File → Open Folder → bu klasör.

**Cloud Agent değil** — normal Cursor Agent (yerel).

---

## 2. Çocuk / yanınızdaki — sırayla (çift tık)

| Sıra | Dosya | Ne yapar |
|------|-------|----------|
| 1 | `npm install` (terminal, bir kez) | Bağımlılık |
| 2 | `ONAYLI_TUM_ISLEMLER.cmd` | **Önerilen** — pull + yol + güncelle + panel |
| 3 | `YEREL_HAZIR_BASLAT.cmd` | Sadece hazırlık |
| 4 | `KURULU_SURUMU_GUNCELLE.cmd` | Sadece .exe güncelle |
| 5 | `YOLLARI_KONTROL.cmd` | Sadece yol kontrolü |

Tam yollar:
```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\YEREL_HAZIR_BASLAT.cmd
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\KURULU_SURUMU_GUNCELLE.cmd
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\YOLLARI_KONTROL.cmd
```

---

## 3. Veri dosyaları (diskte olmalı)

| Dosya | Adres |
|-------|--------|
| UZMAN havuzu | `C:\Users\Esisya\Desktop\asker motoru\runtime\indexes\UZMAN_HAVUZU.json` |
| Skill library | `C:\Users\Esisya\Desktop\Lokal Kütüphane\database\skill_library.sqlite` |

`YOLLARI_KONTROL.cmd` **[VAR]** / **[YOK]** yazar.

---

## 4. Panel açılınca

1. Kurulu program veya `TAURI_DEV.cmd`
2. **YARBAY EMEL** sekmesi → **Emel'i Başlat** (bir kez tık)
3. Ses yoksa: `SESLI_OZET_OKU.cmd`

---

## 5. Git / GitHub

| | |
|--|--|
| Sizden git push | **Hayır** |
| Sizden git pull | Sadece kod güncellemek isterseniz |
| gcloud | **Kapalı** |

---

## 6. Kullanmayın (eski cloud akışı)

- Cursor Cloud Agent görevleri
- `KOPRU_TUNEL_BASLAT.cmd`
- Uzak sunucu / tünel / GCP

---

## Sorun giderme

| Belirti | Çözüm |
|---------|--------|
| .cmd yok | Klasör içinde arayın; masaüstü simgesi değil |
| Eski alarm/ses | `ACIL_SES_KES_VE_GUNCELLE.cmd` |
| Emel sessiz | Emel'i Başlat + `SESLI_OZET_OKU.cmd` |

---

## Agent'a yazın (yerel Cursor)

- `yol kontrol et`
- `kurulu exe güncelle`
- `cargo test çalıştır`

Agent **aynı PC'de** terminal açar — cloud gerekmez.
