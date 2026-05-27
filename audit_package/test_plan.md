# Test Planı (Test Plan)

Sistem bütünlüğünü ve kapıların doğruluğunu kanıtlayan test senaryoları:

## 🧪 Sprint 1 Test Senaryoları

### 1. Planlama Kilidi Testi
- **Koşul**: Görev planı oluşturulurken 17 alandan biri eksik bırakılır.
- **Beklenen Çıktı**: `execution_engine` işlemi başlatmayı engellemeli ve `planning_incomplete` durumunda kalmalıdır.

### 2. Yetkisiz Karar Kilidi Testi
- **Koşul**: Karar düğümü yetki matrisinde tanımlanmamış bir provider ile çalıştırılmak istenir.
- **Beklenen Çıktı**: `authority_router` işlemi durdurmalı ve yetkisiz karar düğümü hatası loglanmalıdır.

### 3. Kullanıcı Onay Kilidi Testi
- **Koşul**: High riskli dosya yazma işlemi başlatılır, ancak kullanıcı onayı verilmez.
- **Beklenen Çıktı**: İşlem `Approval Gate` kapısında bloke edilmeli ve yazma işlemi engellenmelidir.

### 4. Rollback Geri Yükleme Testi
- **Koşul**: Dosya yazma işlemi başarılı olur fakat ardından çalıştırılan test başarısız döner.
- **Beklenen Çıktı**: `rollback_manager` tetiklenmeli, dosya eski haline dönmeli ve rollback raporu oluşturulmalıdır.
