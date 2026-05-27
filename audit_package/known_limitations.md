# Lokal Bilgisayar Kontrol Paneli - Güncel Sınır Notları

Bu dosya eski Sprint 1 sınırlama metninin yerine güncellenmiştir. Üretim kaynak yolunda geçersiz sağlayıcı, geçersiz connector, örnek akış veya otomatik onay modeli kullanılmamalıdır.

## Güncel Sınırlar

- Canlı API connector varsayılan olarak kapalıdır; gerçek `base_url` ve `LIVE_SITE_API_KEY` tanımlanmadan etkinleştirilmez.
- Yazma, silme, API yazma ve terminal komutu işlemleri görev/düğüm/aksiyon/risk bağlamına bağlı yetkili onay kaydı olmadan çalıştırılmaz.
- Eski build çıktıları ve eski snapshot dosyaları üretim kaynağı kabul edilmez; doğrulama kaynak dosyaları ve güncel build komutları üzerinden yapılır.
- Test ve rollback kanıtları gerçek test kriterleri, snapshot metadata alanları ve audit kayıtlarıyla izlenir.
