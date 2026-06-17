# İŞLEM KAYDI — Yanlış alarm ve kesilmeyen ses kök analizi

**Tarih:** 11.06.2026  
**Saat:** (oturum)  
**Yapan ekip:** Engin (Kurucu / test bildirimi), Cursor Cloud Agent (Kök analiz ve düzeltme)  
**Konu klasörü:** `operasyon_merkezi/yapilan_islemler/kontrol-departmani/`  
**Durum:** TAMAMLANDI

---

## Özet

İlk alarm düzeltmesi diskte kaldı, commit/push edilmediği için masaüstü kurulumu eski kodu çalıştırmaya devam etti. Ek olarak TTS `onerror` → kritik alarm döngüsü, açılışta sistem doğrulama sirenı ve sayfa yenilemede ses kuyruğunun durmaması tespit edildi. Hepsi giderildi.

---

## Kök neden analizi (neden doğru olmadı)

| # | Sorun | Sonuç |
|---|--------|--------|
| 1 | Önceki düzeltme **commit edilmedi** | Kurulu `.exe` hâlâ `raiseCriticalAlarm("Görev detayları...")` çalıştırıyor |
| 2 | `taskId` / `input` Tauri uyumsuzluğu | `get_*` komutları hata döndürüyordu |
| 3 | `utterance.onerror` → `raiseCriticalAlarm` | Ses kesilince yeni alarm → ses tekrar → döngü |
| 4 | `checkSystemHealth` her açılışta siren | `asker_motoru_live_api` config validator hatası |
| 5 | Sayfa yenileme sesi durdurmuyordu | `speechSynthesis` + bekleyen `setTimeout` bip'leri temizlenmiyordu |

---

## Yapılan düzeltmeler

1. `src/lib/tauriInvoke.ts` — argüman normalizasyonu (tüm panel invoke)
2. `src/lib/alarmPolicy.ts` — alarm bastırma, dedupe, operasyonel okuma sınıfı
3. `+page.svelte` — açılışta `silenceAllAudio`, TTS hata döngüsü kırıldı, health-check siren kaldırıldı
4. `config/system_connectors.json` — `asker_motoru_live_api` rollback/test listesi
5. `lib.rs` — `get_*` komutları `critical-error` yayınlamaz
6. Bileşenler `invokePanel` kullanıyor

---

## Doğrulama

- `npm run check` — 0 hata
- `cargo test` — 51 geçti
- `system_validator` — artık CONNECTOR hatası yok

---

## Masaüstünde uygulama

Sayfa yenileme yetmez. **Yeni derleme şart:**

```bash
cd "Lokal Bilgisayar Kontrol Paneli"
npm run tauri build
```

Kurulu sürüm: `AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\` — yeni `.exe` ile değiştirin.

---

## Sonraki adım

- Kurucu onayı ile commit/push ve installer güncellemesi.
