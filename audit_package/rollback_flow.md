# Geri Alma Akışı (Rollback Flow)

Hata anında en son kararlı duruma geri dönüşü (Rollback) yöneten mimari akış:

## 🔄 Rollback İşlem Adımları
1. **İşlem Öncesi Snapshot**: Riskli bir yazma işlemi başlamadan önce `rollback_manager` tetiklenerek hedef dosya veya veritabanının o anki verilerini yedekler ve bunu `snapshots` tablosuna yazar.
2. **State History Kaydı**: Alınan yedek verileri JSON formatında `state_history` tablosuna yazılır ve görev kaydı (`tasks.last_valid_state_id`) bu yedek noktasıyla ilişkilendirilir.
3. **Başarısızlık Durumu**: Eğer işlem sonrasındaki testler (`test_manager`) veya bütünlük kontrolü (`integrity_checker`) başarısız olursa, `rollback_manager` tetiklenir.
4. **Geri Yükleme**: `rollback_manager` veritabanından ilgili `state_json` içeriğini okuyarak dosyaları eski kararlı durumlarına geri döndürür.
