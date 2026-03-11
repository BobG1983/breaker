## ⚠️ CRITICAL RULES
**NEVER edit, remove, rename, or create source files (.rs, .ron, .toml, etc.).** Only report what needs fixing — never apply fixes. The only files you may write are memory files under `.claude/agent-memory/test-runner/`.

**NEVER use bare cargo commands.** Always use dev aliases: `cargo dbuild`, `cargo dcheck`, `cargo dclippy`, `cargo dtest`. Only exception: `cargo fmt`.

# Build Validation Status

**Last Validation: PASS** (2026-03-11, feature/main-menu-screen branch, latest)
- Format: clean
- Clippy: clean (no warnings or errors)
- Tests: 131 passed, 0 failed, 0 ignored

## Bevy 0.18.1 API Notes
- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`

## Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator

## Key Patterns
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Message struct fields marked with `#[allow(dead_code)]` if intentional API not yet consumed

## FixedUpdate Test Issue
- FixedUpdate systems DO NOT run on first `app.update()` in tests (fixed timestep accumulation)
- Solution: use `Update` schedule in test systems instead (see dash.rs for pattern)
- Affects: bump_visual tests `animate_applies_y_offset_during_animation` and `animate_removes_bump_visual_when_done`
- File: src/breaker/systems/bump_visual.rs lines 190, 195-225, 228-259
