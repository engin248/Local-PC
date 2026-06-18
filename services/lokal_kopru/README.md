# Lokal Komuta Köprüsü

> **Varsayılan mod:** gcloud/Cloud Agent **kapalı** (`config/yerel_calisma_modu.json`).  
> Komutan iş akışı: yerel `.cmd` dosyaları — köprü **zorunlu değil**.

Windows'ta isteğe bağlı yerel HTTP sunucusu + dosya kuyruğu. Uzak tünel **devre dışı** (`KOPRU_TUNEL_BASLAT.cmd` → KAPALI).

## Normal kullanım (köprü yok)

| İşlem | Komut |
|-------|-------|
| Yol kontrolü | `YOLLARI_KONTROL.cmd` |
| Kurulu exe güncelle | `TEK_TIK_GUNCELLE.cmd` veya `KURULU_SURUMU_GUNCELLE.cmd` |
| Panel kapat | `ACIL_PANEL_KAPAT.cmd` |

Rehber: `operasyon_merkezi/kurulum/BASLANGIC_SIFIR.md`

---

## (İsteğe bağlı) Yerel köprü sunucusu

1. **`KOPRU_KURULUM.cmd`** çift tık — 127.0.0.1:19200  
2. Token: `config/kopru_bridge.json` → `token` (paylaşmayın)

**Tünel / Cloud erişimi:** kapalı — kullanmayın.

## Dosya kuyruğu (tünel yoksa)

`storage/kopru/inbox/*.json` → köprü işler → `storage/kopru/outbox/`

Örnek: `storage/kopru/inbox/ORNEK_gorev.json.example`

## Güvenlik

- Sadece **izinli** işlemler (`allowed_operations`)
- **Token** zorunlu (`X-Kopru-Token`)
- Sunucu varsayılan **127.0.0.1**

## Port

Varsayılan: **19200** (`config/kopru_bridge.json`)
