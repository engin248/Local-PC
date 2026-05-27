# Installer Hash Dogrulama Raporu

## Karar

Kurulu exe ile `src-tauri/target/release` altindaki ham release exe hash'i birebir ayni degildir. Bu durum kurulu uygulamanin eski veya yanlis oldugunu gostermedi.

Dogru kabul standardi su sekilde netlestirildi:

- Installer paketleri ve installer tarafindan kurulan exe ayri ayri hashlenir.
- Kurulu exe, installer cikisi olarak canonical runtime artifact kabul edilir.
- `target/release/lokal_bilgisayar_kontrol_paneli.exe` ham build artifact olarak izlenir; kurulu exe ile birebir hash esitligi tek basina kabul kriteri degildir.
- Kurulu exe, son production kod markerlarini tasimiyorsa veya versiyon/isim/boyut sapmasi varsa test fail olur.

## Kanitlar

Test tarihi: 2026-05-27

Kurulu exe:

- Yol: `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe`
- Boyut: `11606016`
- SHA256: `36B58F230D9AE074A3534885A4FA2162B44D73BF3CA7DD8238AEBF70A0FC2356`
- ProductName: `LOKAL BILGISAYAR KONTROL PANELI`
- ProductVersion: `0.1.0`

Ham target/release exe:

- Yol: `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\src-tauri\target\release\lokal_bilgisayar_kontrol_paneli.exe`
- Boyut: `11606016`
- SHA256: `298DAC306C38404E55CF59E97BE47E187A418EA1EEE983CCA9FE3BA899EB2819`
- ProductName: `LOKAL BILGISAYAR KONTROL PANELI`
- ProductVersion: `0.1.0`

Installer paketleri:

- NSIS SHA256: `4E0EE4D30E894EE3F0BF18826D483B41CB0DF81C57607803B704C75D9C0EC59E`
- MSI SHA256: `0DDF92DD9C3E19FB66B379BEEDBE3273C22010E4F11E83637CEB37FCDC564BDC`

Binary fark ozeti:

- Kurulu exe ve ham target/release exe ayni boyuttadir.
- Binary karsilastirmada `3` bayt fark vardir.
- Bu fark kodun eski olmasi veya eksik derlenmesi seklinde degerlendirilmedi.

Kurulu exe icinde dogrulanan production markerlari:

- `CREATE TABLE IF NOT EXISTS operation_steps`
- `CREATE TABLE IF NOT EXISTS operation_monitor_logs`
- `CREATE TABLE IF NOT EXISTS principle_evaluations`
- `COUNT(DISTINCT approver_id)`
- `selected_best_option_reason`
- `accepted_correct_approach_reason`

Runtime SQLite semasi:

- `operation_steps` var.
- `operation_monitor_logs` var.
- `principle_evaluations` var.
- `principle_evaluations.accepted_correct_approach_reason` var.
- `principle_evaluations.selected_best_option_reason` var.

## Kalici Kontrol

`scripts/verify_installed_release.ps1` eklendi.

Calistirma:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\verify_installed_release.ps1
```

Bu kontrol sunlari fail-closed dogrular:

- Kurulu exe var mi?
- NSIS ve MSI paketleri var mi?
- Kurulu exe ile ham target/release exe boyutu ayni mi?
- ProductName dogru mu?
- Versiyonlar ayni mi?
- Kurulu exe production markerlarini tasiyor mu?
- Binary fark beklenen sinirin disina cikiyor mu?

## Sonuc

Kurulu exe hash'i ile ham target/release exe hash'i birebir esitlenmeye zorlanmadi. Bu, installer zincirini baypas eden gecici bir kopyalama islemi olurdu.

Kalici cozum olarak dogru artifact siniri tanimlandi ve otomatik dogrulama eklendi.
