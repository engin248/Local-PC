# GÖREV — Üretim Departmanı Faz 1

**Görev ID:** GOREV-URETIM-2026-06-11-01  
**Koordinatör:** Cloud Agent  
**Komutan:** Engin  
**Durum:** BAŞLADI

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

- [ ] `execution_engine.rs`: onay sayısı yeterliyse context `ApprovedExecution`
- [ ] `action_executor`: onaylı `write_file` uçtan uca test
- [ ] `cargo test` yeşil
- [ ] Rapor: `raporlar/RAPOR_URETIM_01.md`

---

## Komutan notu

Git pull/commit **komutan tarafından zorunlu değil**. Agent push eder. Kurulu `.exe` güncellemesi yalnızca alarm/eski sürüm varsa zaruri.
