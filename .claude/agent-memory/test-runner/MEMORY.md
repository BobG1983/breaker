## ⚠️ CRITICAL RULES
**NEVER edit, remove, rename, or create source files (.rs, .ron, .toml, etc.).** Only report what needs fixing — never apply fixes. The only files you may write are memory files under `.claude/agent-memory/test-runner/`.

**NEVER use bare cargo commands.** Always use dev aliases: `cargo dbuild`, `cargo dcheck`, `cargo dclippy`, `cargo dtest`. Only exception: `cargo fmt`.

# Build Validation Status

**Last Validation: PASS** (2026-03-12, main)
- Format: PASS (2 files auto-formatted)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (221 passed, 0 failed, 0 ignored)

## Bevy 0.18.1 API Notes
- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`

## Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator
- Multi-line `assert!` with format args must be wrapped: condition, message, args on separate lines
- Multi-line method chains: wrap at logical points when necessary
- Long function call arguments: one argument per line when wrapping

## Key Patterns
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Message struct fields marked with `#[allow(dead_code)]` if intentional API not yet consumed
- Collapse nested `if let` with inner `if` condition into single `if let ... && ...` (collapsible_if lint)
- Keep test helper structs and functions at module level, not inside test functions (items_after_statements)

## Validation History
- **2026-03-12, main (current)**: PASS
  - Format: PASS (2 files auto-formatted: bolt/components.rs, breaker/systems/dash.rs)
  - Clippy: PASS (no warnings or errors)
  - Tests: PASS (221 passed, 0 failed, 0 ignored)
  - Change: +1 test since previous validation
  - Status: Main branch is clean and ready for development
- **2026-03-12, feature/grade-dependent-bump-cooldown**: PASS
  - Format: PASS (1 file auto-formatted: bolt_breaker_collision.rs)
  - Clippy: 1 warning (missing_const_for_fn in cooldown_for_grade)
  - Tests: 208 passed, 0 failed, 0 ignored
  - Change: +8 tests (bump grade cooldown mechanics)
  - Note: cooldown_for_grade in breaker/systems/bump.rs could be const; flagged by clippy nursery lint
- **2026-03-12, feature/bump-timing-rework**: PASS
  - Format: PASS (1 file auto-formatted: bump_visual.rs)
  - Clippy: PASS (no warnings or errors)
  - Tests: 200 passed, 0 failed, 0 ignored
  - Change: multi-line spawn chain condensed to single line per rustfmt
- **2026-03-12, main**: PASS
  - Format: PASS (1 file auto-formatted: tilt_visual.rs)
  - Clippy: PASS (no warnings or errors)
  - Tests: 184 passed, 0 failed, 0 ignored
  - Change: refactored tilt_visual tests to use parametrized helper function
