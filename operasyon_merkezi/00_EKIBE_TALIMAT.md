# Operasyon Merkezi — 5 Ekip Üyesine Talimat

**Tarih:** 2026-05-30  
**Proje:** Lokal Bilgisayar Kontrol Paneli  
**Klasör kökü:** `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\operasyon_merkezi`

---

## 1. Bu sistem ne?

Panel içinde anlık sohbet yok. **Beş ekip üyesi bu klasör üzerinden haberleşir.**

| Adım | Ne olur? |
|------|----------|
| 1 | Koordinatör (Auto / Antigravity) görev dosyasını `gorevler/` klasörüne koyar |
| 2 | Her ekip üyesi **kendi adına yazılmış dosyayı** okur, işi yapar |
| 3 | Bitince raporunu `raporlar/` klasörüne kaydeder |
| 4 | Koordinatör raporları okur, kontrol eder |
| 5 | Eksik veya hatalıysa yeni görev dosyası verir; tamamsa durumu `kontrol/` altında işaretler |

**Kural:** Görev alınmadan rapor yazılmaz. Rapor yazılmadan görev tamamlanmış sayılmaz.

---

## 2. Klasör yapısı

```
operasyon_merkezi/
├── 00_EKIBE_TALIMAT.md          ← Bu dosya (herkes okur)
├── 01_PROJE_PLANI_OZET.md        ← Kalan %30 işlerin özeti
├── gorevler/                     ← Görevler buradan alınır
│   ├── GOREV_[ISIM].md
│   └── ...
├── raporlar/                     ← Raporlar buraya konur
│   ├── RAPOR_[ISIM]_[TARIH].md
│   └── ...
├── kontrol/                      ← Koordinatör kontrol kayıtları
│   └── KONTROL_DURUMU.md
└── sablonlar/
    ├── GOREV_SABLONU.md
    └── RAPOR_SABLONU.md
```

---

## 3. Görev alma (her ekip üyesi)

1. `operasyon_merkezi/gorevler/` klasörünü aç.
2. **Kendi adına** yazılmış `GOREV_[ISIM].md` dosyasını bul.
3. Dosyada şunlar yazar:
   - Görev kimliği
   - Sorumlu ekip / rol
   - Yapılacak işler (madde madde)
   - Etki alanı (hangi dosyalar)
   - Kabul kriterleri
   - Teslim: rapor dosya adı
4. Anlamadığın madde varsa raporda **BLOCKER** olarak yaz; yine de raporu teslim et.

---

## 4. Rapor verme (her ekip üyesi)

1. İş bitince `operasyon_merkezi/raporlar/` klasörüne git.
2. Dosya adı: `RAPOR_[ISIM]_2026-05-30.md` (tarih değişebilir).
3. `sablonlar/RAPOR_SABLONU.md` formatını kullan.
4. Raporda mutlaka olsun:
   - Hangi görev maddeleri **TAMAMLANDI**
   - Hangileri **YAPILAMADI** ve neden
   - Değişen dosyalar (tam yol)
   - Test / doğrulama çıktısı (varsa)
   - Sonraki adım önerisi (varsa)

---

## 5. Koordinatör işi (Auto)

- `gorevler/` altına isim bazlı görev dosyalarını yazar.
- `raporlar/` altındaki raporları okur.
- `kontrol/KONTROL_DURUMU.md` dosyasını günceller:
  - VERILDI / ALINDI / RAPORLANDI / ONAYLANDI / TEKRAR_GOREVLENDIRILDI
- Eksik kalan işler için yeni veya güncellenmiş görev dosyası üretir.

---

## 6. Beş ekip ve genel roller

| Ekip | Platform / Rol | Ana sorumluluk |
|------|----------------|----------------|
| **Ekip 1** | Codex — Çekirdek İcra | Rust backend, connector icra, AI provider, sync |
| **Ekip 2** | Cursor — Arayüz & Swarm UI | Svelte UI, panel refactor, swarm görünürlüğü |
| **Ekip 3** | OAM — Güvenlik & Denetim | 8 kapı, config drift, policy, fail-closed |
| **Ekip 4** | Perplexity — Doğrulama & Test | Test, parite, satır denetimi |
| **Ekip 5** | Antigravity / Auto — Orkestrasyon | Koordinasyon, köprü, release, kayıt defteri |

> **Not:** Kurucu ekran görüntüsündeki gerçek isimler geldikten sonra `gorevler/GOREV_[ISIM].md` dosyaları kişi adına göre güncellenecektir.

---

## 7. Panel ile ilişki

Bu klasör, **Lokal Bilgisayar Kontrol Paneli** projesinin operasyonel haberleşme katmanıdır. Paneldeki Intake / görev / onay akışı ileride bu klasörle entegre edilebilir; şimdilik **dosya tabanlı koordinasyon resmi kanaldır**.

Proje kod kökü:  
`C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli`

---

## 8. İletişim özeti (tek cümle)

**Görevleri `gorevler/` klasöründen al → yap → raporu `raporlar/` klasörüne koy → koordinatör `kontrol/` üzerinden takip eder.**

---

*Kurucu Engin onayı ile yürütülür.*
