# Proje Planı Özeti — Kalan %30

**Referans:** Tam analiz bu oturumda üretildi.  
**Genel durum:** ~%70 tamamlandı, ~%30 kaldı.

---

## Kritik kalan işler (öncelik sırası)

1. **FAZ-6** — Connector gerçek icra katmanı (gate sonrası read/write)
2. **FAZ-7** — AI provider çağrısı + failover
3. **FAZ-8** — AI Swarm workflow (inbox/outbox + DB + UI)
4. **FAZ-9** — Asker Motoru köprüsü (parite, süreç/port görünürlüğü)
5. **FAZ-10** — Supabase sync + log rotasyonu
6. **FAZ-11** — UI refactor (+page.svelte bölme)
7. **FAZ-12** — Backend refactor (execution_engine, dependency_analyzer)
8. **FAZ-13** — E2E test + CI
9. **FAZ-14** — Production release
10. **FAZ-15** — Dokümantasyon ve kayıt defteri

---

## Doğrulama standardı (her raporda belirtilmeli)

```powershell
cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\src-tauri"
cargo check
cargo test
cargo clippy --all-targets -- -D warnings

cd "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
npm run check
npm run build
```

---

## Bilinen uyarı

`gorev_defteri.md` içinde Supabase ve failover "SUCCESS" yazıyor; kod tabanında henüz yok. Raporlarda **kod kanıtı** zorunludur.
