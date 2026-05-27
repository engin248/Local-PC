# Mimari Özet (Architecture Summary)

Bu doküman, ikinci göz denetçi AI platformunun LOKAL BİLGİSAYAR KONTROL PANELİ yapısal mimarisini doğrulaması için hazırlanmıştır.

## 🏗️ 3 Katmanlı Sistem Mimarisi
1. **Frontend (Kullanıcı Arayüzü)**: Svelte + TypeScript + HTML/CSS ile tasarlanmış, sade ve görsel süslemeden uzak, doğrudan operasyon verisine odaklanan arayüz.
2. **Backend Çekirdek (Rust Tauri)**: Yetki matrisini yöneten, 8 kapılı (Execution Gates) güvenlik kontrolü uygulayan ve tüm logları SQLite veritabanına basan çekirdek.
3. **Depolama Katmanı (SQLite app.db)**: Tüm durum geçişlerini, snapshot yedeklerini ve yetkilendirmeleri yerel veritabanında saklayan sunucusuz veri deposu.

## 🛡️ Güvenlik İlkeleri
- **Plansız İşlem Yapılamaz**: Planlama kilidi (Planning Gate) 17 zorunlu alan doğrulanmadan hiçbir işlemi başlatmaz.
- **Dağıtık Karar Yetkisi**: Tek bir AI kendi ürettiği çıktıyı tek başına onaylayamaz; yetki matrisi (`authority_matrix.json`) üzerinden yetkilendirme doğrulanır.
- **Rollback Güvencesi**: Yazma/değiştirme/silme içeren her işlem öncesinde snapshot alınması zorunludur.
- **Kullanıcı Kontrolü**: High veya Critical risk içeren tüm işlemler kullanıcı onayı (`approval_manager`) olmadan uygulanamaz.
