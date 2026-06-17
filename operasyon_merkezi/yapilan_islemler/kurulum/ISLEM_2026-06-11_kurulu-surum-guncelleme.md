# İşlem Kaydı — Kurulu Sürüm Güncelleme ve Geliştirme Modu

| Alan | Değer |
|------|-------|
| Tarih | 2026-06-11 |
| Saat | 22:58 UTC |
| Konu | Kurulu exe güncelleme + tauri dev |
| Ekip | Cloud Agent |

## Hedef

- Kurulu sürüm: `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe`
- Geliştirme modu: `npm run tauri dev`
- Kaynak: `master` @ `326dc4b6` (alarm düzeltmeleri dahil)

## Yapılanlar

1. **Bulut ortamında** `npm run tauri dev` başlatıldı (tmux: `tauri-dev`).
   - Vite: `http://localhost:200/`
   - Rust debug build derlendi ve çalıştı.

2. **Windows için otomasyon scriptleri** eklendi:
   - `scripts/update_installed_exe.ps1` — git pull, build, kurulu exe değiştirme, doğrulama
   - `scripts/tauri_dev.ps1` — geliştirme modu başlatıcı

## Kullanıcı makinesinde çalıştırılacak komutlar

### Kurulu sürümü güncelle (önerilen)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
powershell -ExecutionPolicy Bypass -File .\scripts\update_installed_exe.ps1
```

### Geliştirme modu

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
npm install
powershell -ExecutionPolicy Bypass -File .\scripts\tauri_dev.ps1
```

veya doğrudan:

```powershell
npm run tauri dev
```

## Not

Cloud Agent Windows `AppData\Local` yoluna doğrudan yazamaz; kurulu `.exe` güncellemesi kullanıcının Windows makinesinde yukarıdaki script ile yapılmalıdır.
