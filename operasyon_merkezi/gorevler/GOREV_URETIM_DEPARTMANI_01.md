# GÖREV — Üretim Departmanı Faz 1

**Görev ID:** GOREV-URETIM-2026-06-11-01  
**Durum:** FAZ 1 TAMAMLANDI (yerel, push yok)

---

## Madde durumu

| ID | Konu | Durum |
|----|------|-------|
| URETIM-01 | Onay → ApprovedExecution | **TAMAM** |
| URETIM-02 | AI provider Statement/Execution | **TAMAM** — Statement Gate + action_executor |
| URETIM-03 | Failover policy | **TAMAM** — `failover_policy.json` + `select_with_failover` |
| URETIM-04 | Swarm inbox/outbox | **TAMAM** — `sync_outbox_reports` eklendi |
| URETIM-05 | Asker canlı köprü | **DOSYA KÖPRÜSÜ AKTİF** — canlı API `enabled: false` (bilinçli; komutan PC'de açılır) |

---

## Komutan kuralı

Git push/pull **onay olmadan yapılmaz**. İş yerel depoda.

Kurulu `.exe`: `KURULU_SURUMU_GUNCELLE.cmd` — taşıyıcı klasöründen, git yok.

---

## Raporlar

- `raporlar/RAPOR_URETIM_01.md`
- `raporlar/RAPOR_URETIM_FAZ1.md`
