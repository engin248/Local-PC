# Operasyon Merkezi — Kitap İndeksi (doğru kaynak)

**Güncelleme:** 2026-06-11  
**Mod:** gcloud kapalı — yerel Windows + yerel Cursor Agent

---

## Önce okuyun

| Sıra | Dosya | Ne için |
|------|-------|---------|
| 1 | `kurulum/BASLANGIC_SIFIR.md` | Sıfırdan kurulum (komutan) |
| 2 | `raporlar/GIT_VE_KURULUM_ZORUNLU_MU.md` | Git push/pull gerekir mi? |
| 3 | `raporlar/GITHUB_DURUM.md` | GitHub vs yerel depot |
| 4 | `kontrol/KONTROL_DURUMU.md` | KN maddeleri — **güncel durum** |
| 5 | `raporlar/EKSIKLER_TABLOSU.md` | Komutan işaretleme tablosu |

---

## Kurulum (Windows)

| Dosya | Konu |
|-------|------|
| `kurulum/BASLANGIC_SIFIR.md` | İlk adımlar |
| `kurulum/DOGRU_YEREL_ADRESLER.md` | UZMAN, skill_library yolları |
| `kurulum/KURULU_EXE_NEREDEN_GUNCELLENIR.md` | .exe güncelleme |
| `kurulum/CURSOR_YEREL_AGENT.md` | Yerel Agent (cloud yok) |
| `kurulum/ASKER_CANLI_API_NEDIR.md` | Canlı API açık/kapalı |

---

## Raporlar

| Dosya | Not |
|-------|-----|
| `raporlar/RAPOR_AUTO_TUM_KN.md` | KN kanıt özeti |
| `raporlar/RAPOR_URETIM_01.md` | ApprovedExecution |
| `raporlar/RAPOR_URETIM_FAZ1.md` | Üretim faz 1 |
| `raporlar/RAPOR_SISTEM_MUFETTIS_2026-06-11.md` | Tarihsel denetim (Cloud dönemi) |

---

## Eski / dikkat

| Dosya | Uyarı |
|-------|--------|
| `02_CALISMAYAN_SOMUT_LISTE.md` | Özet tablo güncellendi; alt KN metinleri tarihsel olabilir |
| `raporlar/RAPOR_CLOUD_AGENT_*` | Cloud Agent dönemi — artık kullanılmıyor |
| `kurulum/KOPRU_KURULUM.md` | Tünel kapalı — `config/yerel_calisma_modu.json` |

---

## Config (kök `config/`)

| Dosya | |
|-------|--|
| `yerel_calisma_modu.json` | gcloud kapalı, yerel mod |
| `yerel_veri_yollari.json` | PC tam yolları |
| `skill_library.json` | SQLite arama sırası |
| `asker_motoru_bridge.json` | Asker kök |

---

## Test (son doğrulama)

```
cd src-tauri && cargo test   → 54 unit + 1 e2e = 55 geçti
npm run check                → 0 hata
```
