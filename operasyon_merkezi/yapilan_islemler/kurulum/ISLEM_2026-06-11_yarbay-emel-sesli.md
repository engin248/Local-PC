# İşlem Kaydı — Yarbay Emel Hanım Sesli Erişilebilirlik

| Alan | Değer |
|------|-------|
| Tarih | 2026-06-11 |
| Konu | Göz bandajı erişilebilirlik — Yarbay Emel sesli sohbet |
| Commit | master |

## Yapılanlar

1. `config/voice_persona.json` — Yarbay Emel Hanım persona
2. `YarbayEmelSohbetPanel.svelte` — büyük yazı, panodan oku, otomatik ses
3. Varsayılan sekme: **YARBAY EMEL — SESLİ**
4. `scripts/emel_panodan_oku.ps1` — Windows panodan sesli okuma
5. Tüm panel mesajları Emel hattına düşer ve seslendirilir

## Aile / çocuk için

```powershell
git pull origin master
.\ACIL_SES_KES_VE_GUNCELLE.cmd
```

Panel açılınca ilk sekme sesli hat. Mesaj yapıştır → **Oku**.

Cursor sohbetini okutmak için metni kopyalayıp panelde **Panodan Oku** veya:

```powershell
powershell -File .\scripts\emel_panodan_oku.ps1
```
