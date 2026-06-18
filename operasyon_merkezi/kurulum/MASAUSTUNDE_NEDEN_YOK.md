# Masaüstünde neden yok?

**Silinmedi.** Hiç sizin bilgisayara **gitmedi**.

---

## Ne oldu?

Cloud Agent (ben) dosyaları **uzak sunucuda** yazdım.  
Sizin PC'deki klasör **ayrı kopya** — otomatik senkron olmaz.

| Yer | YEREL_HAZIR_BASLAT.cmd | TEK_TIK_GUNCELLE.cmd |
|-----|------------------------|----------------------|
| Uzak sunucu (ben) | **VAR** | **VAR** |
| GitHub master | **YOK** (push edilmedi) | **YOK** |
| Sizin PC | **YOK** | **YOK** |

Kimse silmedi — **henüz oraya konmadı.**

---

## Nereye bakmalısınız?

Dosyalar **masaüstü simgesi değil**, **klasörün içinde**:

```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\
```

Windows'ta **Desktop** üzerinde tek tek ikon aramayın; önce bu **klasörü** açın.

---

## PC'nizde ŞU AN muhtemelen VAR (eski master)

Bu klasörde bunları arayın:

| Dosya | Tam yol |
|-------|---------|
| Kurulu güncelle | `...\KURULU_SURUMU_GUNCELLE.cmd` |
| Sesli özet | `...\SESLI_OZET_OKU.cmd` |
| Acil kapat | `...\ACIL_PANEL_KAPAT.cmd` |
| Emel | `...\EMEL_BASLAT.cmd` |

Bunlar **YEREL_HAZIR_BASLAT** yerine geçebilir:
- Güncelleme → `KURULU_SURUMU_GUNCELLE.cmd` çift tık

---

## Yeni dosyalar PC'ye nasıl gelir?

**Seçenek A — Onay verirseniz:** GitHub'a push ederim, siz (veya çocuk) bir kez `git pull` (sadece bu sefer).

**Seçenek B — Cursor yerel Agent:** Windows'ta Cursor açık, aynı klasörde Agent'a "eksik cmd dosyalarını oluştur" dersiniz.

**Seçenek C — Manuel:** Çocuk `KURULU_SURUMU_GUNCELLE.cmd` ile devam eder (zaten varsa).

---

## Özet

- Kontrol paneli / masaüstü simgesi ≠ proje klasörü  
- Silme yok; **senkron yok**  
- Şimdilik: `KURULU_SURUMU_GUNCELLE.cmd` kullanın
