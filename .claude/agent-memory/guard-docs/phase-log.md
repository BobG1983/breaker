---
name: phase-log
description: Record of doc review sessions and what was updated (dates and scope)
type: project
---

## 2026-03-28 — feature/collision-cleanup review

**Branch:** feature/collision-cleanup (commit 35c10d1 effect system rewrite)

**Files reviewed:**
- `breaker-game/src/bolt/messages.rs`, `breaker/messages.rs`, `cells/messages.rs`
- `breaker-game/src/run/messages.rs`, `run/node/messages.rs`
- `breaker-game/src/ui/messages.rs`, `wall/messages.rs`
- `breaker-game/src/effect/core/types.rs`, `mod.rs`, `commands.rs`, `sets.rs`
- `breaker-game/src/effect/effects/mod.rs`, `speed_boost.rs`
- `breaker-game/src/effect/triggers/mod.rs`
- `breaker-game/src/game.rs`

**Docs updated:**
- `docs/architecture/messages.md` — 6 changes (collision message renames, new messages, DamageCell field, Observer Events section replaced)
- `docs/architecture/plugins.md` — 2 changes (cross-domain message names, full Effect Domain section rewrite)
- `docs/architecture/effects/core_types.md` — 2 changes (EffectKind enum corrected)
- `docs/architecture/effects/reversal.md` — 1 change (passive buffs and new effect types table)
- `docs/architecture/effects/node_types.md` — 1 change (SecondWind unit variant example)
- `docs/architecture/layout.md` — 1 change (effect domain file structure)

**Items confirmed no-drift:**
- `docs/design/chip-catalog.md` — TiltControl and MultiBolt were never present; SpawnBolts correct
- `docs/design/effects/ramping_damage.md` — `damage_per_trigger` matches code
- `docs/plan/index.md` — phase completion status accurate

## 2026-03-28 — feature/stat-effects review (merged to develop)

**Branch:** feature/stat-effects (merge commit 74d538b)

**Files reviewed:**
- `breaker-game/src/effect/sets.rs` — confirmed both Bridge and Recalculate variants
- `breaker-game/src/effect/plugin.rs` — confirmed Recalculate.after(Bridge) configure_sets
- `breaker-game/src/effect/mod.rs` — confirmed Effective* re-exports
- `breaker-game/src/effect/effects/damage_boost.rs`, `speed_boost.rs`, `size_boost.rs`, `piercing.rs`, `bump_force.rs`, `quick_stop.rs` — confirmed Active*/Effective* pattern
- `breaker-game/src/chips/components.rs` — confirmed intentional stub
- `breaker-game/src/bolt/components.rs` — confirmed PiercingRemaining lives here
- `breaker-game/src/bolt/queries.rs` — confirmed reads EffectivePiercing, EffectiveDamageMultiplier
- `breaker-game/src/breaker/queries.rs` — confirmed reads EffectiveSpeedMultiplier, EffectiveSizeMultiplier
- `breaker-game/src/bolt/sets.rs` — confirmed BoltSystems::CellCollision variant
- `breaker-game/src/breaker/sets.rs` — confirmed BreakerSystems::UpdateState variant
- `breaker-game/src/bolt/plugin.rs` — confirmed prepare_bolt_velocity.after(Recalculate)
- `breaker-game/src/breaker/plugin.rs` — confirmed move_breaker.after(Recalculate)

**Docs updated:**
- `docs/architecture/plugins.md` — 3 changes (Cross-Domain Read Access, EffectSystems entry, sets.rs line in Effect Domain section)
- `docs/architecture/ordering.md` — 3 changes (Defined Sets table additions, FixedUpdate chain Recalculate insertion, Reading narrative)
- `docs/architecture/data.md` — 1 addition (Active/Effective Component Pattern section)
- `docs/plan/index.md` — 1 addition (Stat Effects entry in Current section)

**Items confirmed no-drift:**
- `chips/components.rs` is intentionally a stub — correct, not a missing file
- `effect/effects/` module list in plugins.md is marked "(~24 total)" — non-exhaustive by design
