# Masaüstünde neden yok?

**Silinmedi.** Hiç sizin bilgisayara **gitmedi**.

---

## Ne oldu?

Dosyalar önce **yerel depot / uzak geliştirme ortamında** yazıldı.  
Sizin PC'deki klasör **ayrı kopya** — otomatik senkron olmaz. **Cloud Agent kullanılmıyor.**

| Yer | YEREL_HAZIR_BASLAT.cmd | TEK_TIK_GUNCELLE.cmd |
|-----|------------------------|----------------------|
| Uzak sunucu (ben) | **VAR** | **VAR** |
| GitHub master | **YOK** (10 commit push edilmedi) | **YOK** |
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

**Seçenek A — Önerilen:** Windows'ta **Cursor yerel Agent** — proje klasörünü açın; eksik `.cmd` dosyalarını oluşturmasını isteyin. Rehber: `CURSOR_YEREL_AGENT.md`

**Seçenek B — Manuel:** `KURULU_SURUMU_GUNCELLE.cmd` ile devam (zaten varsa).

**Seçenek C — GitHub (onay gerekir):** Push + bir kez `git pull`. Bkz. `raporlar/GITHUB_DURUM.md`

---

## Özet

- Kontrol paneli / masaüstü simgesi ≠ proje klasörü  
- Silme yok; **senkron yok**  
- Şimdilik: `KURULU_SURUMU_GUNCELLE.cmd` kullanın
