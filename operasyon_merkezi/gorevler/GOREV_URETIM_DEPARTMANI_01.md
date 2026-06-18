# GÖREV — Üretim Departmanı Faz 1

**Görev ID:** GOREV-URETIM-2026-06-11-01  
**Koordinatör:** Cloud Agent  
**Komutan:** Engin  
**Durum:** URETIM-01 TAMAMLANDI (yerel, push yok)

---

## Amaç

Sistem yarım kalmasın — **üretim icrası** gate sonrası gerçek çalışsın. Şu an `execute_task_pipeline` varsayılan `RunMode::ReadOnly`; onaylı görevlerde `ApprovedExecution` açılmalı.

---

## Kapsam (sıra)

1. **URETIM-01** — Onay tamamlanınca `run_mode = ApprovedExecution` (yüksek/kritik risk + çift onay sonrası)
2. **URETIM-02** — AI provider gerçek çağrı (Statement/Execution)
3. **URETIM-03** — Failover policy
4. **URETIM-04** — Swarm inbox/outbox
5. **URETIM-05** — Asker Motoru canlı köprü (lokal)

---

## İlk teslim (URETIM-01)

- [x] `execution_engine.rs`: onay sayısı yeterliyse context `ApprovedExecution`
- [x] `action_executor`: onaylı `write_file` uçtan uca test
- [x] `cargo test` yeşil (54 test)
- [x] Rapor: `raporlar/RAPOR_URETIM_01.md`

---

## Komutan notu

Git pull/commit/push **komutan onayı olmadan yapılmaz**. İş yerel depoda kalır. Cloud Agent uzak repoya yalnızca komutan açık onay verirse push eder.
