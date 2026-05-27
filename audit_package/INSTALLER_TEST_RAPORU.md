# LOKAL BİLGİSAYAR KONTROL PANELİ - Gerçek Installer Test Raporu

Test tarihi: 27 Mayıs 2026

## A. Test edilen paket dosyası

NSIS setup dosyası test edildi:

`C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\src-tauri\target\release\bundle\nsis\LOKAL BILGISAYAR KONTROL PANELI_0.1.0_x64-setup.exe`

MSI paketi ayrıca kurulum, açılış ve uninstall zincirinden geçirildi:

`C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\src-tauri\target\release\bundle\msi\LOKAL BILGISAYAR KONTROL PANELI_0.1.0_x64_en-US.msi`

## B. Kurulum ortamı

- Windows kullanıcı ortamı: `Esisya`
- Windows sürüm bilgisi: Windows 10 Pro, 64-bit, NT `10.0.26200.0`
- Kurulum tipi: NSIS sessiz kullanıcı kurulumu ve MSI sessiz kullanıcı kurulumu
- Temizlik yöntemi: Önceki kurulum kaldırıldı. Runtime app data klasörü test öncesinde yedeklenerek ayrıldı:
  `C:\Users\Esisya\AppData\Local\com.engin.lokal-bilgisayar-kontrol-paneli.pre_installer_test_20260527104704`

Not: Ayrı yeni Windows kullanıcı hesabı oluşturulmadı. Test mevcut kullanıcı altında temiz runtime veri klasörüyle yapıldı.

## C. Kurulum sonucu

NSIS kurulum komutu başarıyla tamamlandı:

- `install_exit=0`
- Kurulum klasörü oluştu:
  `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI`
- Kurulan dosyalar:
  - `lokal_bilgisayar_kontrol_paneli.exe`
  - `uninstall.exe`

MSI kurulum komutu başarıyla tamamlandı:

- Kurulum klasörü oluştu:
  `C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI`
- Registry uninstall kaydı oluştu:
  `MsiExec.exe /X{FF623646-58AC-452A-B693-6D01ABF24A25}`
- Kurulan dosya:
  - `lokal_bilgisayar_kontrol_paneli.exe`

## D. Uygulama açılış sonucu

İlk installer testinde uygulama açılışta kapanıyordu. Kök sebep:

- Kurulum ortamında proje kökü bulunamıyordu.
- `config` ve `storage` klasörleri kurulu runtime ortamında oluşmuyordu.
- `SystemValidator` daha sonra config tutarsızlıklarını yakalayıp açılışı durduruyordu.

Uygulanan düzeltmelerden sonra tekrar kurulum yapıldı. Son NSIS açılış sonucu:

- Uygulama 8 saniyelik gözlem sonunda çalışır durumdaydı.
- Process sonucu: `RUNNING`
- Pencere başlığı: `Lokal Bilgisayar Kontrol Paneli`
- Test sonunda process elle durduruldu.

MSI açılış sonucu:

- Uygulama 8 saniyelik gözlem sonunda çalışır durumdaydı.
- Process sonucu: `MSI_RUNNING`
- Pencere başlığı: `Lokal Bilgisayar Kontrol Paneli`
- Test sonunda process elle durduruldu.

## E. Panel adı kontrolü

Panel içi görünen başlık kaynakta doğrulandı:

```text
LOKAL BİLGİSAYAR
KONTROL PANELİ
```

Kurulu Windows pencere başlığı:

```text
Lokal Bilgisayar Kontrol Paneli
```

Installer ürün adı teknik WiX/NSIS uyumluluğu için ASCII karakterlerle kalmıştır:

```text
LOKAL BILGISAYAR KONTROL PANELI
```

## F. SystemValidator sonucu

İlk kurulum açılışında SystemValidator şu hataları yakaladı ve uygulamayı durdurdu:

- `AUTHORITY_UNKNOWN_ACTION: code_modification_control`
- `AUTHORITY_UNKNOWN_ACTION: file_read`
- `WRITE_ROLLBACK_MISSING: terminal_command`
- `WRITE_ROLLBACK_MISSING: write_folder`
- `POLICY_UNKNOWN_ACTION: file_delete`

Kök düzeltmeler:

- `risk_rules.json` içine `file_read` ve `code_modification_control` gerçek risk aksiyonları eklendi.
- `approval_rules.json` içindeki eski `file_delete` kalıntısı kaldırıldı.
- `rollback_rules.json` içindeki eski `file_delete` kalıntısı kaldırıldı.
- `rollback_rules.json` içine `write_folder` ve `terminal_command` eklendi.

Son installer testinde SystemValidator açılışı bloke etmedi; uygulama çalışır durumda kaldı.

## G. Config/DB erişim sonucu

Kurulu runtime ortamında otomatik oluşan kök:

`C:\Users\Esisya\AppData\Local\com.engin.lokal-bilgisayar-kontrol-paneli`

Kontrol sonucu:

- `runtime_root_exists=True`
- `config_exists=True`
- `storage_exists=True`
- `db_exists=True`
- `snapshots_exists=True`
- `backups_exists=True`
- `reports_exists=True`

SQLite şema kontrolü:

- `app.db` oluştu.
- Tablolar oluştu: `tasks`, `approvals`, `decision_nodes`, `state_history`, `snapshots`, `tests`, `reports`, `execution_logs` ve diğer çekirdek tablolar.

## H. Read-only güvenlik kontrolü

Kod yolu doğrulandı:

- Varsayılan execution context `read_only=true`.
- Varsayılan approval source `DatabaseOnly`.
- Write-like action tespitinde read-only mod `read_only_blocked` ile işlemi durdurur.
- Connector yazma istekleri `approval_context` olmadan kabul edilmez.

Ek otomatik güvenlik testi eklendi:

- `system_connectors::file_connector::tests::blocks_file_write_without_valid_approval`
- `approval_context` olmadan dosya yazma engellendi.
- `approval_context` olsa bile DB'de geçerli/yetkili onay yoksa fiziksel dosya yazılmadı.
- `cargo test` sonucu: `5 passed; 0 failed`.

## I. Onay sistemi kontrolü

Onay kontrol yolu doğrulandı:

- `approved_at IS NOT NULL`
- `approver_id` boş olamaz.
- Yetkili roller: `admin`, `owner`, `security_officer`
- Yüksek/kritik riskte `COUNT(DISTINCT approver_id)` kontrolü vardır.
- Connector yazma aksiyonları veritabanındaki geçerli approval kaydına bağlanmıştır.

Kurulu temiz DB üzerinde başlangıç onay sayısı `0` olarak doğrulandı.

## J. Rollback dizin kontrolü

Kurulu runtime storage altında şu dizinler oluştu:

- `storage\snapshots`
- `storage\backups`
- `storage\reports`

Rollback policy tutarlılığı SystemValidator tarafından doğrulanacak hale getirildi.

## K. Uninstall sonucu

NSIS uninstall test edildi:

- `uninstall_exit=0`
- Kurulum klasörü kaldırıldı: `install_exists_after=False`
- Uninstall registry kaydı kaldırıldı: `uninstall_entry_exists=False`
- Runtime kullanıcı verisi korundu: `appdata_preserved=True`

Runtime kullanıcı verisinin uninstall sonrası korunması standart kullanıcı-verisi davranışıdır.

MSI uninstall test edildi:

- Kurulum klasörü kaldırıldı: `msi_install_exists_after=False`
- Uninstall registry kaydı kaldırıldı: `msi_uninstall_entry_exists=False`
- Runtime kullanıcı verisi korundu: `runtime_data_preserved=True`

## L. Kalan hata/risk

1. Test ayrı yeni Windows kullanıcı hesabında değil, mevcut kullanıcıda temiz runtime app data klasörüyle yapıldı.
2. Installer ürün adı teknik paket uyumluluğu nedeniyle ASCII: `LOKAL BILGISAYAR KONTROL PANELI`. Panel içi görünen ad Türkçe karakterli doğru addır.
3. UI üzerinden butonla yazma denemesi yapılmadı; onaysız yazma engeli otomatik connector testiyle doğrulandı.

Ek tarama:

- Production/source/config/audit kapsamında eski ad, sahte sağlayıcı/connector, demo dosya, sahte test, otomatik onay ve tek düğüm işleme kalıpları yeniden tarandı.
- Tarama sonucu: bulgu yok.

Son doğrulama komutları:

- `cargo check`: geçti
- `cargo test`: `5 passed; 0 failed`
- `cargo clippy --all-targets -- -D warnings`: geçti
- `npm run check`: `0 errors and 0 warnings`
- `npm run build`: geçti

## M. Son karar önerisi

Şartlı teknik kabul, kurulum testi açısından kabul edilebilir seviyeye gelmiştir. NSIS ve MSI için açılış, runtime config üretimi, SQLite oluşturma, SystemValidator geçişi, rollback dizinleri ve uninstall zinciri doğrulandı.

Manuel görsel alarm testi otomatikleştirildi ve kanıt ekran görüntüsü alındı:

`audit_package/alarm_visual_test.png`

Yönetim kurulu sunum raporu:

`audit_package/YONETIM_KURULU_KABUL_RAPORU.md`

Tam operasyon kabulü için kalan tek öneri, ayrı bir Windows kullanıcı hesabında kısa manuel görsel açılış tekrar testidir.
