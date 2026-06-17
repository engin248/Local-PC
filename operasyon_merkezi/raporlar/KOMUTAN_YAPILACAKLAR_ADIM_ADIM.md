# Komutanın Yapması Gerekenler — Cloud Agent Yapamaz

**Tarih:** 2026-06-11  
**Neden:** Cloud Agent uzak Linux sunucusundadır; sizin Windows bilgisayarınıza, kurulu programa ve hoparlörünüze doğrudan erişemez.

---

## ADIM 0 — Raporu ŞİMDİ sesli dinle (çocuk veya siz)

PowerShell açın, **tek blok** yapıştırın:

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
powershell -NoProfile -ExecutionPolicy Bypass -File ".\scripts\GECE_DEVRI_RAPOR_OKU.ps1"
```

Veya dosyaya çift tık: `scripts\GECE_DEVRI_RAPOR_OKU.cmd`

Ses gelmezse ADIM 0B'ye geçin.

### ADIM 0B — İndirilen ses dosyasını çal

Repo içinde hazır ses: `scripts/audio/gece_devri_raporu_tr.wav`  
Dosyaya çift tıklayın — Windows Media Player veya telefonunuzla açın.

---

## ADIM 1 — Eski panel sesini durdur (F5 yetmez)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
.\ACIL_PANEL_KAPAT.cmd
```

**Ne yapar:** Çalışan eski `lokal_bilgisayar_kontrol_paneli.exe` sürecini öldürür.  
**Ben yapamam:** Sizin Windows süreçlerinize uzaktan dokunamam.

---

## ADIM 2 — Kurulu programı güncelle (yeni düzeltmeler)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
.\ACIL_SES_KES_VE_GUNCELLE.cmd
```

**Ne yapar:** Eski paneli kapatır → derler → `AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe` günceller → yeni paneli açar.  
**Süre:** İlk seferde 5–15 dakika (Rust derlemesi).  
**Ben yapamam:** `C:\Users\Esisya\AppData\Local\...` yoluna yazamam.

---

## ADIM 3 — Panelde Yarbay Emel sesli hattını aç

Panel açılınca:

1. İlk sekme: **YARBAY EMEL — SESLİ** (otomatik açılmalı)
2. Üstte: **Yarbay Emel — Sesli Okuma Açık** olmalı
3. Mesaj yapıştır → **Oku** veya **Panodan Oku**

**Ben yapamam:** Sizin masaüstü pencerenizi açamam veya tıklayamam.

---

## ADIM 4 — Cursor sohbet mesajlarını sesli duymak

Cursor sohbetini ben seslendiremem. Şunlardan biri:

```powershell
# Panodan: mesajı kopyala, sonra:
powershell -File "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\scripts\emel_panodan_oku.ps1"
```

Veya panelde **Panodan Oku**.

---

## ADIM 5 — Sabah (isteğe bağlı kontrol)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
npm run check
cd src-tauri
cargo test
```

Normal kullanıcı için ADIM 1–2 yeterli.

---

## Benim YAPABİLDİKLERİM (tamamlandı — tekrar gerekmez)

- [x] Kod yazma, test, GitHub'a push (`master` @ güncel)
- [x] Alarm düzeltmeleri, Emel paneli, scriptler
- [x] Gece devri raporu dosyası
- [x] Ses dosyası üretimi (repo içi `.wav`)

---

## Benim YAPAMADIKLARIM (sizin yapmanız şart)

| # | İş | Neden |
|---|-----|--------|
| 1 | Kurulu `.exe` değiştirmek | Windows AppData sizin PC'de |
| 2 | Eski panel sürecini kapatmak | Uzak süreç yönetimi yok |
| 3 | Hoparlörünüzden ses çalmak | Fiziksel cihazınızda değilim |
| 4 | Cursor sohbetine TTS eklemek | Cursor uygulamasını değiştiremem |
| 5 | Panel penceresini açmak/tıklamak | Masaüstü otomasyonu yok |
| 6 | `npm run tauri build` sizin Windows'ta | Sizin makinede derleme gerekir |

---

## Yardım — ses gelmiyorsa

1. Hoparlör açık mı, ses kısık mı?
2. `GECE_DEVRI_RAPOR_OKU.ps1` yönetici olmadan normal PowerShell'de çalıştırın
3. `scripts\audio\gece_devri_raporu_tr.wav` dosyasına çift tıklayın

İyi geceler, komutanım.
