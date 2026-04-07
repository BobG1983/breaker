---
name: Casting lint patterns in civil_from_days
description: The civil_from_days function in output_dir.rs — cast errors fixed with try_from; remaining warnings are expect_used and dead_code
type: project
---

The `civil_from_days` function at `breaker-scenario-runner/src/runner/output_dir.rs:60` implements Howard Hinnant's date algorithm.

## Current state (2026-04-06)

All `clippy::cast_*` errors have been fixed using `try_from(...).expect(...)` with range-invariant messages. The function is now a plain `fn` (not `const fn`).

**Remaining warnings (all expected, not errors):**

- `dead_code` (8 warnings): All public functions in `output_dir.rs` are unused — `BASE_DIR`, `next_run_number`, `today_date_string`, `civil_from_days`, `create_run_dir`, `format_violation_entry`, `write_violations_log`, `clean_output_dir`. These are scaffolding for future output logging.
- `expect_used` (6 warnings): The `try_from(...).expect(...)` pattern used to replace the raw `as` casts triggers `clippy::expect-used`. Panics are acceptable here because the values are mathematically bounded by the Hinnant algorithm invariants.
- `option_if_let_else` (1 warning): `format_violation_entry` uses a `match` on `Option` instead of `map_or_else`. Cosmetic — nursery lint.

**Total: 16 warnings, 0 errors** (2 duplicates in test build).

## History

**Why cast errors occurred:** clippy::pedantic (`-D clippy::pedantic`) is enabled workspace-wide. The Hinnant algorithm uses raw `as` casts that clippy cannot prove are safe statically.

**Why `const fn` must NOT be used:** `i64::from(yoe)` uses the `From` trait, which is not yet stable as a const trait (Rust issue #143874). Making `civil_from_days` a `const fn` causes a compile error. The function is only called from `today_date_string()`, a regular function — no const evaluation is needed.

**Fix applied:** Each `as` cast replaced with `try_from(...).expect("range message")`. This satisfies clippy::cast_* at the cost of clippy::expect_used warnings, which are acceptable here.
