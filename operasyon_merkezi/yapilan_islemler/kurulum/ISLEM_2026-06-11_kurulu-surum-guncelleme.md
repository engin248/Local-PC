# İşlem Kaydı — Kurulu Sürüm Güncelleme ve Geliştirme Modu

| Alan | Değer |
|------|-------|
| Tarih | 2026-06-11 |
| Saat | 23:15 UTC |
| Konu | Kurulu exe güncelleme + tauri dev düzeltmeleri |
| Ekip | Cloud Agent |

## Hedef

- Kurulu sürüm: `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe`
- Geliştirme modu: `npm run tauri dev`
- Alarm düzeltmeleri: `master` @ `326dc4b6` ve sonrası

## Yapılan düzeltmeler

1. `update_installed_exe.ps1` — NSIS sessiz kurulum varsayılan; proje kökü otomatik; `-SkipVerify`, `-DirectCopy` seçenekleri
2. `KURULU_SURUMU_GUNCELLE.cmd` — tek tık güncelleme
3. `TAURI_DEV.cmd` — tek tık geliştirme modu
4. `README.md` — kurulum adımları eklendi
5. Bulutta `npm run tauri dev` doğrulandı (Vite :200)

## Windows komutları

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
.\KURULU_SURUMU_GUNCELLE.cmd
```

Geliştirme:

```powershell
npm run tauri dev
```
