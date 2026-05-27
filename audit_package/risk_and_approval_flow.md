# Risk ve Onay Akışı (Risk and Approval Flow)

Kritik işlemlerin kullanıcı onay kapısına düşürülmesini yöneten risk matrisi kuralları:

## 📊 Risk Derecesine Göre Zorunluluk Tablosu

| Risk Seviyesi | İzin Verilen Eylem / Connector Yapısı | Onay Gereksinimi | Rollback Snapshot |
| :--- | :--- | :--- | :--- |
| **Low** | Dosya okuma, listeleme, rapor üretme | Gerekmez (Sadece log yazar) | Gerekmez |
| **Medium** | Test çalıştırma, veritabanı okuma | Gerekmez (Kontrol noktası denetler) | Gerekmez |
| **High** | Dosya yazma, veritabanına kayıt ekleme | Kullanıcı Onayı Zorunlu | Değişiklik öncesi yedek zorunlu |
| **Critical** | Dosya silme, terminal çalıştırma, API yazma | Çift Kullanıcı Onayı Zorunlu | 11 parametreli alternatif analizi + snapshot zorunlu |

## 🚫 Onaysız Engellenenler
1. `approval_manager.rs` dosya yazma/silme, DB yazma ve terminal komutu işlemlerini onaysız olarak tamamen bloke eder.
2. İlk sprintte terminal işlemleri simüle edilerek onay akışlarının doğrulanması hedeflenmiştir.
