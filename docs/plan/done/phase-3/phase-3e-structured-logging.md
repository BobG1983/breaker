# Phase 3e: Structured Logging

**Goal**: Persistent, structured log output for diagnosing scenario failures and live play sessions without needing a debugger.

---

## What Was Built

- `tracing-appender` daily rolling file appender writing to `logs/breaker.log`
- `--log <true|false>` dev flag — disable file logging (e.g. during CI)
- `--log-level <level>` dev flag — override log filter (e.g. `--log-level debug`)
- `info!`/`debug!` calls at lifecycle boundaries in the game crate:
  - run start / run end
  - bolt spawn / bolt lost
  - node cleared
  - layout set
- Scenario runner logs at `breaker_scenario_runner` target: scenario start, pass, fail, per-violation debug lines

## Architecture Notes

- `file_log_layer` in `app.rs` — `LogPlugin::custom_layer` factory; creates `logs/` dir, attaches rolling appender
- `dev_log_config` in `app.rs` — reads `--log` and `--log-level` from process args before `LogPlugin` is registered
- In dev builds: default filter `breaker=debug,bevy=warn`; in release: `breaker=warn,bevy=error`

---

## Checklist

- [x] `tracing-appender` dependency in `breaker-game/Cargo.toml`
- [x] `file_log_layer` factory function in `app.rs`
- [x] `dev_log_config` reads `--log` / `--log-level` flags
- [x] `--log false` disables file appender
- [x] `--log-level <level>` overrides filter string
- [x] `info!`/`debug!` at lifecycle boundaries (run, bolt, node, layout)
- [x] Scenario runner logs at `breaker_scenario_runner` target
