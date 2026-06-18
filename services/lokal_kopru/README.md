# Lokal Komuta Köprüsü

Windows masaüstünüzü Cloud Agent’a (tünel ile) veya dosya kuyruğu ile bağlar.

## İlk kurulum (komutan / çocuk — 2 dakika)

1. **`KOPRU_KURULUM.cmd`** çift tık  
2. İsteğe bağlı Cloud erişimi: **`KOPRU_TUNEL_BASLAT.cmd`** → çıkan `https://....trycloudflare.com` adresini  
   `config/kopru_bridge.json` → `tunnel_public_url` alanına yapıştırın  
3. Token otomatik oluşur: `config/kopru_bridge.json` → `token` (kimseyle paylaşmayın)

## Ne yapabilir?

| İşlem | Yerel | Cloud (tünel açıkken) |
|-------|-------|------------------------|
| Yol kontrolü | `YOLLARI_KONTROL.cmd` | `scripts/kopru_cloud_call.sh yol_kontrol` |
| Kurulu exe güncelle | `TEK_TIK_GUNCELLE.cmd` | `kopru_cloud_call.sh kurulu_guncelle` |
| Panel kapat | `ACIL_PANEL_KAPAT.cmd` | `kopru_cloud_call.sh panel_kapat` |

## Güvenlik

- Sadece **izinli** işlemler (`allowed_operations`)
- **Token** zorunlu (`X-Kopru-Token` başlığı)
- Sunucu varsayılan **127.0.0.1** — dış dünya yalnızca tünel ile

## Dosya kuyruğu (tünel yoksa)

`storage/kopru/inbox/*.json` dosyası yazın → köprü işler → sonuç `storage/kopru/outbox/`

Örnek: `storage/kopru/inbox/ORNEK_gorev.json.example`

## Port

Varsayılan: **19200** (`config/kopru_bridge.json`)
