# Lokal Bilgisayar Kontrol Paneli - Adli Denetim Notu

**Hedef proje:** `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli`

Bu not, eski denetim raporunda yer alan yanlış klasör adlarını, eski config örneklerini ve geçersiz test referanslarını temizlemek için güncellenmiştir.

## Güncel Proje Kimliği

- Görünen uygulama adı: `LOKAL BİLGİSAYAR KONTROL PANELİ`
- Ana klasör: `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli`
- Frontend: `src/`
- Backend: `src-tauri/src/`
- Config: `config/`
- Storage: `storage/`

## Güncel Config Durumu

`config/system_connectors.json` üretim config'inde geçersiz veya sahte connector tanımı bulunmaz.

Geçerli connector mantığı:

- `local_projects`: lokal proje kökü, dinamik `$PROJECT_ROOT` yolu ile.
- `local_app_db`: lokal SQLite veritabanı, dinamik `$PROJECT_ROOT/storage/app.db` yolu ile.
- `live_site_api`: canlı API bağlantısı, varsayılan olarak kapalı.

`config/authority_matrix.json`, `config/risk_rules.json` ve `config/approval_rules.json` içinde dosya, klasör, SQLite, API, terminal, AI provider ve rapor üretimi aksiyonları açıkça tanımlıdır.

## Denetim Bulgusu

Üretim kaynakları ve config dosyalarında eski sahte sağlayıcı, sahte connector, örnek dosya, sahte başarı testi, tek karar düğümü işleme ve varsayılan otomatik kullanıcı onayı kalıntıları bulunmamalıdır.

Bu rapor, geçmiş durumu değil güncel proje kimliğini ve güncel denetim hedefini temsil eder.
