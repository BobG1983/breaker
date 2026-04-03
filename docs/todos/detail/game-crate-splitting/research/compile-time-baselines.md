# Compile Time Baselines — `breaker-game` Crate

Measured 2026-04-01 on macOS (Darwin 25.4.0). All times via `time cargo dcheck` (type-check with dynamic linking).

## Crate Size

| Metric | Value |
|--------|-------|
| Total `.rs` files | 783 |
| Total lines of code | 104,658 |
| Domains | 14 (audio, bolt, breaker, cells, chips, debug, effect, fx, input, run, screen, shared, ui, wall) |

### Lines per Domain

| Domain | Files | Lines |
|--------|------:|------:|
| effect | 241 | 34,160 |
| bolt | 97 | 17,380 |
| run | 106 | 13,844 |
| breaker | 72 | 9,938 |
| chips | 63 | 8,998 |
| cells | 50 | 6,615 |
| screen | 65 | 5,780 |
| debug | 35 | 3,642 |
| fx | 9 | 1,276 |
| wall | 14 | 901 |
| input | 5 | 587 |
| ui | 10 | 652 |
| shared | 10 | 451 |
| audio | 2 | 32 |
| top-level (app.rs, game.rs, lib.rs, main.rs) | 4 | 402 |

## Compile Time Measurements

| Scenario | Command | Wall Clock | Notes |
|----------|---------|------------|-------|
| Clean check (full rebuild) | `cargo clean && cargo dcheck` | **1m 24.3s** | Includes all dependencies (bevy, etc.) |
| No-change incremental | `cargo dcheck` (second run) | **0.6s** | No-op, nothing to recheck |
| Single file touch (leaf) | `touch bolt/systems/launch_bolt.rs && cargo dcheck` | **0.9s** | Single domain system file |
| Shared module touch | `touch shared/mod.rs && cargo dcheck` | **0.9s** | Widely-imported shared types |
| Crate root touch | `touch lib.rs && cargo dcheck` | **0.9s** | Worst-case invalidation |

## Analysis

**Incremental recheck is already very fast (~0.9s)** regardless of which file is touched. Even touching `lib.rs` (crate root, maximum invalidation) takes the same time as touching a leaf file. This is because `cargo check` with incremental compilation can skip codegen entirely and the Rust compiler's query-based incremental system is effective at this crate size.

**Key takeaway for sub-crate evaluation**: The primary benefit of splitting into sub-crates would NOT be incremental check time (already sub-second). Potential benefits would be:
1. **Parallel compilation of sub-crates** on clean builds (currently 1m 24s is dominated by dependency compilation, not `breaker-game` itself)
2. **Organizational clarity** (enforced dependency boundaries between domains)
3. **Avoiding recompilation of unrelated domains during `cargo build`/`cargo test`** (codegen is more expensive than type-checking)

A useful follow-up measurement would be `cargo dbuild` (full codegen) touch tests, since codegen time is where monolith costs are higher than type-checking alone.
