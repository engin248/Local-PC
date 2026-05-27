# Lokal Bilgisayar Kontrol Paneli - Düzeltme Planı

Bu plan, eski klasör adları ve geçersiz connector örnekleri temizlendikten sonra güncel production hardening işlerini izlemek için kullanılır.

## Kapatılan Maddeler

- Production config içinden mock connector/provider kalıntıları kaldırıldı.
- `system_connectors.json` dinamik proje yollarına bağlandı.
- `write_folder` aksiyonu authority, approval ve risk configlerinde tanımlandı.
- Execution context varsayılanı read-only olacak şekilde sertleştirildi.
- Otomatik kullanıcı onayı ve tek karar düğümü işleme yaklaşımı kaldırıldı.
- Approval kayıtları approver id, rol ve kaynak bilgisiyle doğrulanacak hale getirildi.
- Rollback snapshot metadata alanları gerçek hedefe bağlandı.
- Test Gate tanımlı kriterlerle çalışacak şekilde sertleştirildi.

## Takip Edilecek Maddeler

- Her yeni write/delete/API/terminal aksiyonu config ve DB approval policy ile birlikte eklenmelidir.
- Eski denetim raporları üretim gerçekliğini yansıtmıyorsa arşiv notu olarak güncellenmelidir.
- `target/` klasörü build çıktısıdır; eski kalıntılar için kaynak kabul edilmez ve gerektiğinde `cargo clean` ile yenilenir.

## Doğrulama Standardı

Her düzeltme sonrası şu komutlar çalıştırılır:

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\src-tauri"
C:\Users\Esisya\.cargo\bin\cargo.exe check
C:\Users\Esisya\.cargo\bin\cargo.exe test
C:\Users\Esisya\.cargo\bin\cargo.exe clippy --all-targets -- -D warnings
C:\Users\Esisya\.cargo\bin\cargo.exe build

cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
npm run check
npm run build
npm run tauri -- build
```
