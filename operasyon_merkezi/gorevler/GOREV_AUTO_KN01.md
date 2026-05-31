# GÖREV — Auto (KN-01)

**Görev ID:** GOREV-2026-05-30-KN01  
**Sorumlu:** Auto  
**Birlikte:** Kurucu Engin (onay / test)  
**Durum:** BAŞLADI

---

## Amaç

8 kapı geçildikten sonra connector’ların gerçekten çağrılması. Şu an pipeline sadece DB/log güncelliyor; fiziksel icra yok.

---

## Yapılacaklar

- [ ] `action_executor.rs` modülü oluştur
- [ ] `execution_engine` gate zinciri sonunda dispatch ekle
- [ ] Onaylı `write_file` için tek uçtan uca senaryo
- [ ] `execution_logs` → `action_execute` event
- [ ] `cargo test --lib` yeşil

---

## Kontrol noktaları (bitince işaretle)

| KP | Kontrol | KK | Sonuç |
|----|---------|-----|-------|
| KP-01 | dispatch çağrısı var mı? | `action_execute` log | ☐ |
| KP-02 | onaysız write | fail + dosya yok | ☐ |
| KP-03 | 2 approver + snapshot | yazma başarılı | ☐ |

---

## Teslim

`raporlar/RAPOR_AUTO_KN01.md`
