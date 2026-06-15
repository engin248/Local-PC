# AGENTS.md

## Cursor Cloud specific instructions

### Primary product

The main runnable product is **Lokal Bilgisayar Kontrol Paneli** at the repository root: SvelteKit frontend + Tauri 2 / Rust backend + embedded SQLite (`storage/app.db`). See `README.md` for architecture and gate workflow.

`openclaw/` is a separate pnpm monorepo (OpenClaw gateway/CLI). It is not wired into the panel’s `src/` or `src-tauri/` code. Only work there when explicitly asked.

### Commands (panel)

| Task | Command |
|------|---------|
| Install JS deps | `npm install` |
| Typecheck / lint | `npm run check` (`svelte-check`; no root `npm run lint`) |
| Frontend build | `npm run build` |
| Rust tests | `cd src-tauri && cargo test` (51 tests: 50 unit + 1 e2e) |
| Dev (desktop) | `npm run tauri dev` (starts Vite on port **200** + Tauri window) |
| Vite only | `npm run dev` → http://localhost:200/ (Tauri IPC will not work in a plain browser) |
| Desktop release build | `npm run tauri build` |

### Linux / Cloud Agent gotchas

1. **Port 200 is privileged** on default Linux (`ip_unprivileged_port_start` = 1024). Vite/Tauri dev will fail with `EACCES` unless Node can bind low ports, e.g. once per VM image:
   `sudo setcap 'cap_net_bind_service=+ep' "$(readlink -f "$(which node)")"`
   Do not put `setcap` in the VM update script.

2. **Rust toolchain**: Tauri 2 dependencies may require **Rust ≥ 1.85+** (edition 2024 crates). If `cargo test` fails on `edition2024`, run `rustup default stable` before building. The update script does not run `rustup`.

3. **Tauri system packages** (Debian/Ubuntu): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`, `build-essential`. Install via image/snapshot, not the update script.

4. **Long-running dev**: Use a named tmux session (e.g. `tauri-dev`) for `npm run tauri dev`; first compile can take ~1–2 minutes.

5. **Optional services** (Supabase, AI APIs, Pinecone): configured under `config/`; app runs without them.

6. **Startup config validator can silently block the GUI**: `run()` (in `src-tauri/src/lib.rs`) calls `SystemValidator::validate_or_fail()` *before* building the Tauri window. Any `error`-severity inconsistency in the `config/*.json` files (validated by `src-tauri/src/core/validator/`) makes `run()` print a Turkish `Sistem doğrulaması başarısız:` message plus a forensic report and `return` immediately — the window never opens and there is no crash/panic. If `npm run tauri dev` compiles and exits cleanly within ~1s back to the shell prompt, check `/tmp/tauri-dev.log` for this message. A common cause is a `system_connectors.json` connector that lists a write-like action (e.g. `api_write`) in `approval_required_actions` without also listing it in `rollback_required_actions` and `test_required_actions` (every write-capable connector must include the action in all three lists). Note: `cargo test` does **not** catch this — the `run_validator` test only asserts `validate()` returns `Ok`, not that the issue list is empty.

7. **Known audit-logging alarm on writes**: In the running app, every mutating UI action (create task, save plan, execute, approve, rollback) triggers a red `SİSTEM HATASI / KRİTİK ALARM` banner reading `Audit kayıt hatası: ... append_operation_audit_cmd missing required key input`. This is a pre-existing frontend/backend contract mismatch (the Rust `append_operation_audit_cmd` expects a wrapped `{ input: {...} }` arg, but `+page.svelte`'s `appendOperationAudit` sends fields un-wrapped). The underlying operation (e.g. `create_task_cmd`) still succeeds and persists to SQLite before the audit call fails, so the alarm does NOT mean the environment is broken. Do not treat this alarm as a setup failure.

### Secrets (optional)

- `SUPABASE_URL`, `SUPABASE_SERVICE_KEY` or `SUPABASE_ANON_KEY` — cloud sync
- Provider keys in `config/ai_providers.json` / env as documented in connector code
