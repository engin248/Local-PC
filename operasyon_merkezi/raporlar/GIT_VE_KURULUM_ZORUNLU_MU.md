# Git ve Kurulum — Zorunlu mu? (gcloud kapalı, yerel only)

**Güncelleme:** gcloud sıfırlandı. Uzak Cloud Agent **kullanılmaz**.

---

## Sizin yapmanız GEREKMEYEN

| İş | |
|----|--|
| `git commit` / `git push` | Zorunlu değil |
| gcloud / GCP | **Kapalı** |
| Cloud Agent | **Kapalı** |
| Tünel / KOPRU_TUNEL | **Kapalı** |

---

## Sizin yapmanız GEREKEN (yerel, bir kez)

| İş | Dosya |
|----|--------|
| Hazırlık | `YEREL_HAZIR_BASLAT.cmd` |
| Exe güncelle | `KURULU_SURUMU_GUNCELLE.cmd` |
| Yol kontrol | `YOLLARI_KONTROL.cmd` |
| Emel ses | Panel → Emel'i Başlat |

Rehber: `operasyon_merkezi/kurulum/BASLANGIC_SIFIR.md`

---

## git pull ne zaman?

Yalnızca başka makineden kod almak isterseniz — **zorunlu değil**.

---

## Agent kuralı

Yerel Cursor Agent: komutan PC'de çalışır. Push/onay istemeden GitHub'a yazmaz.
