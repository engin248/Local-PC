# Lokal Komuta Köprüsü — Kurulum

> **DURUM: KAPALI (2026-06-11)** — `config/yerel_calisma_modu.json` → gcloud/Cloud Agent kapalı.  
> **Normal kullanım:** `YEREL_HAZIR_BASLAT.cmd` veya `KURULU_SURUMU_GUNCELLE.cmd` — tünel gerekmez.  
> Bu dosya yalnızca **gelecekte isteğe bağlı** köprü kurulumu için arşivlenmiştir.

---

## Ne zaman gerekir?

Köprü, uzak bir agent'ın Windows PC'nize erişmesi içindi. Yerel Cursor Agent kullanıyorsanız **atlayın**.

| İş | Köprü olmadan |
|----|----------------|
| .exe güncelle | `KURULU_SURUMU_GUNCELLE.cmd` veya `TEK_TIK_GUNCELLE.cmd` |
| Yol kontrolü | `YOLLARI_KONTROL.cmd` |
| Panel kapat | `ACIL_PANEL_KAPAT.cmd` |

Başlangıç: `BASLANGIC_SIFIR.md`

---

## (Arşiv) Köprü kurulumu — kullanmayın

`KOPRU_TUNEL_BASLAT.cmd` ekranda **KAPALI** yazar — tünel devre dışı.

Eski adımlar (referans):

1. `KOPRU_KURULUM.cmd` — yerel HTTP sunucu (127.0.0.1:19200)
2. ~~Tünel~~ — **kapalı**
3. Token: `config/kopru_bridge.json` (gizli tutun)

Dosya kuyruğu (tünel olmadan, yerel): `storage/kopru/inbox/*.json` → sonuç `outbox/`

Detay: `services/lokal_kopru/README.md`
