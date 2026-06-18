# Cursor Yerel Agent — Tünel yok

Komutanın gözü kapalıyken **çocuk bir kez** şunu yapar; sonra siz Cursor’da yazarsınız, iş yapılır.

---

## 1. Cursor yüklü mü?

Yoksa: https://cursor.com — **Download for Windows** — kur.

---

## 2. Projeyi aç

Cursor → **File → Open Folder**

```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli
```

---

## 3. Bir kez hazırlık (çift tık)

```
YEREL_HAZIR_BASLAT.cmd
```

İsteğe bağlı (her Windows açılışında köprü): `KOPRU_OTOMATIK_KAYIT.cmd`

---

## 4. Agent modunda yazın (tünel yok)

Cursor sohbetinde **Agent** seçin, örnek:

- `yol kontrol et`  
- `kurulu exe güncelle`  
- `paneli kapat`

Yerel Agent **aynı bilgisayarda** şunu çalıştırır:

```powershell
powershell -File scripts\yerel_panel_islem.ps1 -Islem kurulu_guncelle
```

**Cloud Agent değil** — bu sizin PC’nizdeki Cursor.

---

## Cloud Agent vs Yerel Agent

| | Yerel Cursor Agent | Cloud Agent (uzak) |
|--|-------------------|---------------------|
| Tünel | **Gerekmez** | PC’ye doğrudan erişemez |
| Kim çalıştırır | Sizin diskteki Cursor | Uzak sunucu |
| Öneri | **Sizin için bu** | Kod yazar, test eder |

---

## Çocuğa tek cümle

> Cursor’u aç, masaüstündeki Lokal Bilgisayar Kontrol Paneli klasörünü aç, `YEREL_HAZIR_BASLAT.cmd`’ye çift tıkla.
