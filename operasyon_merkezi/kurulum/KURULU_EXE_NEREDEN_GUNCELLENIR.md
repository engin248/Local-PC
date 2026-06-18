# Kurulu `.exe` — Nereden Güncellenir? (Taşıyıcı = Masaüstü Proje Klasörü)

**GitHub gerekmez.** İş yerel proje klasöründen yapılır.

---

## 1. Taşıyıcıyı aç

Windows Dosya Gezgini:

```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli
```

Bu klasör **taşıyıcı**dır (kaynak kod + scriptler).

---

## 2. Hangi durumda hangi dosya?

| Durum | Çift tık |
|-------|----------|
| Normal güncelleme (ses/alarm sorunu yok) | `KURULU_SURUMU_GUNCELLE.cmd` |
| Eski panel sesi durmuyor / sahte alarm | `ACIL_SES_KES_VE_GUNCELLE.cmd` |
| Sadece paneli kapat | `ACIL_PANEL_KAPAT.cmd` |
| Geliştirme (dev penceresi) | `TAURI_DEV.cmd` |

---

## 3. Git çekmeden güncelleme (sizin kuralınıza uygun)

PowerShell — proje klasöründe:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\update_installed_exe.ps1 -SkipPull
```

`-SkipPull` = GitHub'dan **hiçbir şey indirmez**; elinizdeki yerel kodu derler.

---

## 4. Kurulu program nereye yazılır?

```
C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe
```

Script sırası: eski süreçleri kapat → `npm run tauri build` → NSIS kurulum (veya `-DirectCopy`).

---

## 5. Emel sesi için (güncelleme sonrası)

1. Yeni `.exe` veya `TAURI_DEV.cmd` ile paneli aç  
2. **YARBAY EMEL** sekmesi  
3. **Emel'i Başlat — Ses Hattını Aç** (bir kez tıkla)

---

## Test — Analiz — Onay

| Adım | Not |
|------|-----|
| Test | Script yolları `update_installed_exe.ps1` ile doğrulandı |
| Analiz | Komutan PC'de çift tık yeterli; Cloud Agent Windows'a erişemez |
| Onay | Rehber yerel depoda; GitHub push gerekmez |
