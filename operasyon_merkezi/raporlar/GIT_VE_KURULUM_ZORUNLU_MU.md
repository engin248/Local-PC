# Git ve Kurulum — Zorunlu mu?

**Kural (komutan):** Zorunlu / mecbur / zaruri haller dışında `git pull`, `commit`, `push` **yapılmaz**.

---

## Sizin yapmanız GEREKMEYEN (asla)

| İş | Kim yapar? |
|----|------------|
| `git commit` | Siz yapmayın — Cloud Agent yapar |
| `git push` | Siz yapmayın — Cloud Agent yapar |
| GitHub'a kod yazmak | Cloud Agent (uzak sunucu) |

---

## `git pull` — ne zaman ZORUNLU?

| Durum | Zorunlu mu? |
|-------|-------------|
| Masaüstü klasörünüzde yeni script/ses/düzeltme **yok** ve panel hâlâ eski | **Evet** — sadece indirme (pull), siz commit etmezsiniz |
| Zaten güncel kod var veya `.exe` güncel çalışıyor | **Hayır** |
| Sadece paneli normal kullanıyorsunuz, sorun yok | **Hayır** |

**Pull istemiyorsanız** (git kuralınıza uygun):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\update_installed_exe.ps1 -SkipPull
```

---

## `ACIL_SES_KES_VE_GUNCELLE.cmd` — ne zaman ZORUNLU?

| Durum | Zorunlu mu? |
|-------|-------------|
| Eski panel sesi durmuyor / sahte alarm | **Evet (zaruri)** |
| Kurulu `.exe` eski, Emel paneli yok | **Evet** |
| Panel zaten yeni ve sessiz çalışıyor | **Hayır** |
| Sadece üretim koduna devam (Cloud Agent) | **Hayır — sabah bekleyebilir** |

Bu komut **git push değildir**; sizin bilgisayarda derleme ve kurulumdur.

---

## Sabah özeti

- **Zorunlu değil:** `git pull` + güncelleme (panel düzgünse)
- **Zorunlu olabilir:** Sadece eski `.exe` / alarm sorunu devam ediyorsa
- **Sizin commit/push:** Hiçbir zaman gerekmez

---

## Üretim devam

Gece devri kapanmadı — **Üretim Departmanı** görevleri sürüyor. Cloud Agent kod yazar ve GitHub'a push eder; siz git işlemi yapmazsınız.

Görev dosyası: `gorevler/GOREV_URETIM_DEPARTMANI_01.md`
