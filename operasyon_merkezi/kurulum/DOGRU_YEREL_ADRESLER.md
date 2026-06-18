# DOĞRU YEREL ADRESLER (Komutan PC — Esisya)

Tüm yollar tek dosyada: `config/yerel_veri_yollari.json`

---

## skill_library.sqlite — DOĞRU ADRES

```
C:\Users\Esisya\Desktop\Lokal Kütüphane\database\skill_library.sqlite
```

Yedek arama (dosya yukarıda yoksa):

1. `C:\Users\Esisya\Desktop\asker motoru\database\skill_library.sqlite`
2. `C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli\storage\skill_library.sqlite`

Kod: `config/skill_library.json` + `src-tauri/src/core/skill_library.rs`

---

## UZMAN_HAVUZU.json — DOĞRU ADRES

```
C:\Users\Esisya\Desktop\asker motoru\runtime\indexes\UZMAN_HAVUZU.json
```

Kod: `config/asker_motoru_bridge.json` + `asker_module_registry.rs`

---

## Kurulu panel .exe

```
C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe
```

Güncelleme: proje klasöründe `KURULU_SURUMU_GUNCELLE.cmd` çift tık (git çekmeden: `-SkipPull`).

---

## Yerel not

.exe` güncellemesi ve yol kontrolü **sizin PC'de** yapılır (`KURULU_SURUMU_GUNCELLE.cmd`, `YOLLARI_KONTROL.cmd`). Uzak agent erişimi yok; gcloud kapalı.
