---
name: Universal failure — Bridge/Recalculate/BoltLost scheduling cycle (RESOLVED)
description: RESOLVED 2026-03-30. Removed Recalculate.after(Bridge) set ordering. All scenarios now run (no longer universal panic).
type: project
---

## Bug: Unsolvable Schedule Ordering Cycle

**Error:** `system set Bridge and system <...> (in set Bridge) have both in_set and before-after relationships (these might be transitive). This combination is unsolvable.`

**Root cause:** Four constraints form a circular dependency:

1. `Recalculate.after(Bridge)` — configured in `breaker-game/src/effect/plugin.rs`
2. checkers chain `.after(EffectSystems::Recalculate)` — in `breaker-scenario-runner/src/lifecycle/systems/plugin.rs`
3. checkers chain `.before(BoltSystems::BoltLost)` — same file
4. `bridge_bolt_lost.in_set(EffectSystems::Bridge).after(BoltSystems::BoltLost)` — in `breaker-game/src/effect/triggers/bolt_lost.rs`

**The cycle:** `Bridge < Recalculate < checkers < BoltLost < bridge_bolt_lost (∈ Bridge)` — a member of Bridge is transitively after Bridge. Bevy 0.18 forbids a system from having a before/after constraint with its own set.

**Effect:** ALL scenarios fail with a system panic at schedule initialization, before a single frame runs.

**This is a game bug (scheduling defect), not a runner bug.**

**Fix options (do NOT apply — describe only):**
- Option A (preferred): Remove `Recalculate.after(Bridge)` from plugin.rs and instead order individual Recalculate systems after specific Bridge triggers that feed them (avoids the global set constraint)
- Option B: Remove `.before(BoltSystems::BoltLost)` from the checkers chain and use a different ordering anchor
- Option C: Change `bridge_bolt_lost` to not be in `Bridge` set but still run during the Bridge phase (e.g., use a separate set BridgeBoltLost)

**Files involved:**
- `breaker-game/src/effect/plugin.rs` line 12 — the `Recalculate.after(Bridge)` configure_sets call
- `breaker-game/src/effect/triggers/bolt_lost.rs` lines 41-42 — `in_set(Bridge).after(BoltLost)`
- `breaker-scenario-runner/src/lifecycle/systems/plugin.rs` lines 171-172 — `after(Recalculate).before(BoltLost)`

**Why:** Introduced when fix #2 (ungating Recalculate) was combined with fix #4 (ordering checkers after Recalculate). The pre-existing `.before(BoltLost)` on the checkers chain interacts fatally with `bridge_bolt_lost.after(BoltLost)` via the new Bridge→Recalculate set ordering.
