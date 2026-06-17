# İşlem Kaydı — Acil Ses Kes ve Panel Kapatma

| Alan | Değer |
|------|-------|
| Tarih | 2026-06-11 |
| Konu | Eski panel süreci + ses döngüsü |
| Commit | `463b0ab0` |

## Neden cloud agent sesi kesemedi

Cloud ortamı Linux VM; kullanıcının Windows `AppData\Local\...` yoluna ve çalışan masaüstü sürecine erişemez. F5/yenileme Tauri `speechSynthesis` kuyruğunu durdurmaz.

## Yapılan düzeltmeler (master)

1. `ACIL_PANEL_KAPAT.cmd` — tüm panel süreçlerini öldürür
2. `ACIL_SES_KES_VE_GUNCELLE.cmd` — kapat → build → kur → yeni panel aç
3. Çift `speakReply` hatası (alarm-code) giderildi
4. Güncelleme öncesi süreç kapatma zorunlu

## Kullanıcı komutu

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
git pull origin master
.\ACIL_SES_KES_VE_GUNCELLE.cmd
```
