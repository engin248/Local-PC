# İŞLEM KAYDI — Yanlış kritik alarm kök çözümü

**Tarih:** 11.06.2026  
**Saat:** (oturum)  
**Yapan ekip:** Engin (Kurucu / talep), Cursor Cloud Agent (Uygulama)  
**Konu klasörü:** `operasyon_merkezi/yapilan_islemler/kontrol-departmani/`  
**Durum:** TAMAMLANDI

---

## Özet

Panelde "Görev detayları yüklenirken hata" ve "Acil sistem alarmi" sesli uyarıları kök nedenler giderilerek susturuldu. Sorun Tauri argüman uyumsuzluğu (`taskId` vs `task_id`, audit `input` sarmalayıcısı), salt okunur komutlarda `critical-error` yayını ve audit hatalarında gereksiz siren tetiklemesinden kaynaklanıyordu.

---

## Yapılan adımlar

1. `src/lib/tauriInvoke.ts` — camelCase → snake_case ve `append_operation_audit_cmd` için `{ input }` sarmalayıcısı.
2. `+page.svelte` — `refreshTaskDetails` `Promise.allSettled` ile kısmi yükleme; kritik alarm kaldırıldı.
3. Audit kayıt hatası artık `console.warn` (siren yok).
4. `lib.rs` — `get_*` komutları `critical-error` eventi yayınlamaz.
5. `critical-error` dinleyicisi `get_` komutlarını filtreler.

---

## Değişen dosyalar

| Dosya | Değişiklik |
|-------|------------|
| `src/lib/tauriInvoke.ts` | Yeni — argüman normalizasyonu |
| `src/routes/+page.svelte` | Alarm politikası + invokePanel |
| `src/components/FounderAssignmentPanel.svelte` | invokePanel |
| `src-tauri/src/lib.rs` | Salt okunur komutlarda sessiz hata |
| `AGENTS.md` | Eski audit alarm notu kaldırıldı |

---

## Doğrulama

- `npm run check` ve `cargo test` çalıştırılacak (commit yok — Kurucu talimatı).

---

## Sonraki adım

- Masaüstünde Tauri uygulaması yeniden derlenip alarm tekrarı gözlemlenecek.

---

## Koordinatör notu

- Commit/push yapılmadı (Kurucu: "tek harf push edilmeyecek").
