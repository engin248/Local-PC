# Asker Motoru Canlı API — Ne demek? Artı / Eksi

## Ne bu?

Panel ile **Asker Motoru** arasında iki bağlantı türü var:

| Tür | Şu an | Nasıl çalışır |
|-----|-------|----------------|
| **Dosya köprüsü** | **AÇIK** | Diskteki JSON dosyalarını okur (`PLANLAMA_DURUMU.json`, `UZMAN_HAVUZU.json` vb.) |
| **Canlı API köprüsü** | **KAPALI** (`enabled: false`) | Asker Motoru bir HTTP sunucu çalıştırırsa (`http://127.0.0.1:3100`) panel oradan anlık durum alır |

`enabled: false` = panel **internete/sunucuya bağlanmaya çalışmaz**; sadece dosyaları okur.

---

## Canlı API açılırsa (`enabled: true`)

### Artıları (+)

- Planlama / alarm / eğitim durumu **anlık** gelir (saniyelik)
- Komut cümlesi doğrudan Asker Motoru’na gönderilebilir (`post_command`)
- Dosya kopyalamadan güncel durum

### Eksileri (−)

- Asker Motoru’nun **3100 portunda çalışması şart** — kapalıysa panel “erişilemedi” yazar
- Ek ağ/süreç bağımlılığı; dosya köprüsünden daha kırılgan
- İki sistem aynı anda açık olmalı (bellek, CPU)

---

## Öneri (komutan PC)

| Durum | Ayar |
|-------|------|
| Asker Motoru her zaman açık değil | **`enabled: false` kalsın** — dosya köprüsü yeterli |
| Asker Motoru sürekli çalışıyor ve API hazır | `asker_motoru_bridge.json` içinde `"enabled": true` |

Dosya: `config/asker_motoru_bridge.json` satır 10.

**Şu an doğru ayar:** kapalı — UZMAN havuzu ve planlama dosyadan okunuyor.
