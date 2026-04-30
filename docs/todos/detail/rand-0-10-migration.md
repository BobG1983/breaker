# rand 0.9 → 0.10 Migration

## Summary
Migrate `breaker-game` and `breaker-scenario-runner` from rand 0.9 to rand 0.10.

## Context
Researched 2026-04-30 during a guard-dependencies finding. rand 0.10 was released 2026-02-14.
There is no security or correctness risk to staying on 0.9; this is a maintenance migration.

**Breaking changes that affect this project:**
- `Rng` trait renamed → `RngExt`
- `gen()` → `random()`, `gen_range()` → `random_range()`
- `rand::distributions` module → `rand::distr`
- `Standard` distribution → `StandardUniform`
- `OsRng` → `SysRng`
- `StdRng` loses `Clone`
- `ReseedingRng` removed entirely
- `rand_chacha` crate has no 0.10 release — ChaCha RNGs are now accessed via rand's `chacha`
  feature flag directly. `rand_chacha = "0.9"` in `breaker-game/Cargo.toml` must be dropped.

**No getrandom conflict** — no `getrandom` 0.2 pins found in this workspace.

**Affected crates:**
- `breaker-game/Cargo.toml` — pins `rand = "0.9"` and `rand_chacha = "0.9"`
- `breaker-scenario-runner/Cargo.toml` — pins `rand = "0.9"`

## Scope
- In:
  - Update `rand` to `"0.10"` in both affected crates
  - Drop `rand_chacha` dep; migrate to rand's `chacha` feature flag if ChaCha RNGs are used
  - Fix all call sites: `gen()`, `gen_range()`, `distributions::`, `OsRng`, `choose_multiple`, etc.
  - Verify `rand_chacha::ChaCha*Rng` usages and migrate to `rand::rngs::StdRng` or rand chacha feature
- Out:
  - `rantzsoft_*` crates (none use rand directly)
  - getrandom changes (no direct getrandom dependency in this workspace)

## Dependencies
- Depends on: nothing (standalone maintenance)
- Blocks: nothing

## Notes
- Do a call-site audit first (`grep -r "\.gen\(\)\|gen_range\|distributions::\|rand_chacha\|OsRng\|ReseedingRng\|StdRng.*clone"`) before writing an impl spec
- `rand_chacha` removal is the highest-effort change — need to confirm whether the project uses `ChaChaRng` directly or just re-exports via rand

## Status
`ready`
