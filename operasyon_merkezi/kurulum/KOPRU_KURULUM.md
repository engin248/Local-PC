# Lokal Komuta Köprüsü — Kurulum

**Amaç:** Cloud Agent’ın “masaüstüne erişemiyorum” dememesi için Windows’ta köprü sunucusu.

---

## Komutan / çocuk — 3 adım

### 1. Köprüyü kur
```
KOPRU_KURULUM.cmd   (çift tık)
```
Yeni pencere açılır — **kapatmayın** (köprü çalışır).

### 2. Cloud’un erişmesi için tünel (bir kez)
```
KOPRU_TUNEL_BASLAT.cmd   (çift tık)
```
Ekranda çıkan adres örneği:
```
https://abc-xyz.trycloudflare.com
```
Bunu `config/kopru_bridge.json` içine yapıştırın:
```json
"tunnel": {
  "tunnel_public_url": "https://abc-xyz.trycloudflare.com"
}
```

### 3. Token
`config/kopru_bridge.json` içindeki `"token"` değeri — Cloud Agent’a söylemeniz gerekmez; dosya repoda kalır (gizli tutun).

---

## Cloud Agent bundan sonra ne yapar?

Tünel URL config’deyse:
```bash
./scripts/kopru_cloud_call.sh yol_kontrol
./scripts/kopru_cloud_call.sh kurulu_guncelle
```

---

## Artı / Eksi

| Artı | Eksi |
|------|------|
| Uzaktan yol kontrolü, exe güncelleme | İlk kurulumda 2 cmd çalıştırma gerekir |
| Token + sadece izinli işlemler | Tünel penceresi açıkken URL geçerli |
| Git push gerekmez | Tünel kapanınca URL değişebilir (yeniden tünel) |

---

## Günlük kullanım

Windows açılınca: **`KOPRU_BASLAT.cmd`** (otomatik başlatma için Görev Zamanlayıcı’ya eklenebilir)

Güncelleme: Cloud Agent `kurulu_guncelle` gönderir veya siz `TEK_TIK_GUNCELLE.cmd`
