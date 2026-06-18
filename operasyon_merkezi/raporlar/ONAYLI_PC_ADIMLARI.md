# Onaylı işlemler — PC'de yapılacaklar (tek liste)

**Onay:** Komutan — tüm işlemler etkin  
**GitHub:** Push tamam (`master` = `cbd36b7f`)  
**Cloud Agent:** PC'nize dokunamaz — aşağıdaki adımlar **Windows'ta sizin/çocuğun** elinde

---

## Yerel “cin” var mı?

Evet — üç katman:

| Katman | Ne yapar | Nasıl |
|--------|----------|-------|
| **1. Cursor Yerel Agent** | Kod yazar, dosya oluşturur, script çalıştırır | Cursor Desktop → proje klasörünü aç → Agent modu |
| **2. YEREL_HAZIR_BASLAT.cmd** | Köprü + yol kontrolü (tünel yok) | Çift tık |
| **3. YEREL_ISLEM.cmd** | Tek komutla iş | `YEREL_ISLEM.cmd kurulu_guncelle` vb. |

Cloud Agent / gcloud / tünel **kapalı** — bunlar yerine geçer.

---

## Sıra (tek tık — önerilen)

### Tek dosya (hepsi)

`ONAYLI_TUM_ISLEMLER.cmd` çift tık — sırayla:

1. `git pull origin master`
2. Yol kontrolü (UZMAN + skill_library)
3. Kurulu `.exe` derleme ve güncelleme
4. Panel açma + sesli Emel hatırlatması

Tam yol:
```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\ONAYLI_TUM_ISLEMLER.cmd
```

Panel açılınca → **Emel'i Başlat** (bir tık — tarayıcı kuralı).

---

## Sıra (adım adım — alternatif)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
```

### 2 — Hazırlık

`YEREL_HAZIR_BASLAT.cmd` çift tık  
veya `YOLLARI_KONTROL.cmd` — UZMAN + skill_library yolları

### 3 — Kurulu .exe güncelle

`TEK_TIK_GUNCELLE.cmd` çift tık  
(ses takılıysa önce `ACIL_PANEL_KAPAT.cmd`)

### 4 — Emel sesi

Panel açılınca → **YARBAY EMEL** → **Emel'i Başlat** (tıklama şart)

### 5 — Cursor Agent (isteğe bağlı)

Klasör açıkken Agent'a yazın: *"yol kontrol"* veya *"kurulu güncelle"*

---

## Tam yollar

| Dosya | Yol |
|-------|-----|
| Proje | `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\` |
| Kurulu exe | `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe` |
| UZMAN | `C:\Users\Esisya\Desktop\asker motoru\runtime\indexes\UZMAN_HAVUZU.json` |
| skill_library | `C:\Users\Esisya\Desktop\Lokal Kütüphane\database\skill_library.sqlite` |

---

## Ben (Cloud) ne yaptım?

- GitHub `master` push — **tamam**
- Test 55/55 — **geçti**
- Kitap çelişkileri — **düzeltildi**
- PC .exe / hoparlör — **sizin tıklamanız gerekir**
