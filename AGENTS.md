# AGENTS.md

## Komutan kuralı (öncelik)

- **GCP / gcloud / Cloud Agent kullanılmaz** — sistem sıfırlandı, yerel depot.
- **Git push/pull:** komutan açık onayı olmadan **yapılmaz**.
- **Komutan PC işleri** (exe, ses, yol): yalnızca **Windows yerel Cursor Agent** veya `.cmd` çift tık.
- Agent **tünel, köprü uzaktan erişim, Cloud Agent** önermez; `YEREL_HAZIR_BASLAT.cmd` / `KURULU_SURUMU_GUNCELLE.cmd` yeterli.

Başlangıç rehberi: `operasyon_merkezi/kurulum/BASLANGIC_SIFIR.md`

---

## Primary product

**Lokal Bilgisayar Kontrol Paneli** — SvelteKit + Tauri 2 + SQLite (`storage/app.db`).

`openclaw/` ayrı monorepo; panele bağlı değil. Sadece istenirse dokunulur.

---

## Commands (panel)

| Task | Command |
|------|---------|
| Install JS deps | `npm install` |
| Typecheck | `npm run check` |
| Frontend build | `npm run build` |
| Rust tests | `cd src-tauri && cargo test` |
| Dev (desktop) | `npm run tauri dev` (port **200**) |
| Vite only | `npm run dev` |

---

## Windows komutan PC

Proje kökü:
```
C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli
```

| İş | Dosya |
|----|--------|
| İlk hazırlık | `YEREL_HAZIR_BASLAT.cmd` |
| Exe güncelle | `KURULU_SURUMU_GUNCELLE.cmd` veya `TEK_TIK_GUNCELLE.cmd` |
| Yol kontrol | `YOLLARI_KONTROL.cmd` |
| Ses | `SESLI_OZET_OKU.cmd` + panel Emel'i Başlat |

Yerel script:
```powershell
powershell -File scripts\yerel_panel_islem.ps1 -Islem kurulu_guncelle
```

---

## Optional services

Supabase, AI API, Pinokio — `config/` kapalı kalabilir; panel çalışır.

---

## Deprecated (kullanma)

- `KOPRU_TUNEL_BASLAT.cmd`, `scripts/kopru_cloud_call.sh` — uzak cloud için; komutan akışında **kapalı**
- Cursor Cloud Agent — komutan işleri için **kapalı**
