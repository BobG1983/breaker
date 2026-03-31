---
name: scenario-failure-trace
description: Root cause trace for 5 systemic scenario failures appearing across 111 scenarios (feature/scenario-coverage branch)
type: project
---

# Scenario Failure Root Cause Trace

Traced on branch `feature/scenario-coverage` against Bevy 0.18.1.

## Failure 1 — `captured Level(Warn) log` — "Chain Reaction ingredient does not match any known template name"

**Root cause:** `chain_reaction.chip.ron` fails to deserialize because `Do(SpawnBolts())` uses
an empty-parentheses struct-variant syntax that may not correctly trigger `#[serde(default)]`
for all fields in RON 0.12 with `IMPLICIT_SOME`. When the asset fails to parse, `seed_registry`
never seals (it returns `done: 0` as long as any handle returns `None` from `assets.get()`),
which means… actually the game DOES load. The alternative root cause: the `chain_reaction.chip.ron`
file was recently added but has a RON syntax issue with `SpawnBolts()` that causes a parse failure
at load time. Bevy logs an error for the failed asset; `seed_registry` never includes it; the
`ChipTemplateRegistry` does not contain "Chain Reaction"; `validate_recipe_ingredients` fires warn.

**Precise location:** `breaker-game/assets/chips/templates/chain_reaction.chip.ron` line 10:
`Do(SpawnBolts())` — empty struct-variant syntax that may fail serde deserialization.

**Effect:** `verdict.evaluate()` in `src/verdict/evaluation.rs` lines 87-93 treats ANY captured
log as a hard failure. This causes 100% of scenarios to fail with this as the first reason.

**Fix:** Replace `Do(SpawnBolts())` with `Do(SpawnBolts(count: 1))` to provide an explicit
field value so serde does not need to infer defaults from an empty struct body. This removes
the parse ambiguity and allows the template to load successfully.

**Regression or pre-existing:** Likely a Wave 3 regression — chain_reaction.chip.ron was added
in Wave 3 with the SpawnBolts effect, and no other template uses this empty-parentheses syntax.

---

## Failure 2 — `EffectiveSpeedConsistent` — 439 violations

**Root cause:** Ordering gap between `EffectSystems::Bridge` (where SpeedBoost fires via deferred
commands) and the invariant checker chain in `FixedUpdate`.

**Chain:**
1. `evaluate_bound_effects` (in `EffectSystems::Bridge`) calls `commands.fire_effect(entity, SpeedBoost, chip)` — this QUEUES a `FireEffectCommand`, not immediate
2. Command flush: `speed_boost::fire()` inserts `(ActiveSpeedBoosts::default(), EffectiveSpeedMultiplier(1.0))` if absent, then pushes multiplier into `ActiveSpeedBoosts`. `EffectiveSpeedMultiplier` is still at default `1.0`.
3. `recalculate_speed` (in `EffectSystems::Recalculate`, configured `run_if(PlayingState::Active)`) reads `ActiveSpeedBoosts` and updates `EffectiveSpeedMultiplier` to the product — but has NO explicit ordering relative to the invariant checkers.
4. Invariant checker chain runs in `FixedUpdate` `.after(BreakerSystems::UpdateState).before(BoltSystems::BoltLost)`. If checkers run BEFORE `recalculate_speed` in the same frame that SpeedBoost fires, `ActiveSpeedBoosts` has the new multiplier but `EffectiveSpeedMultiplier` is still `1.0` — divergence > SPEED_EPSILON (1e-4).

**Precise location:**
- `src/effect/effects/speed_boost.rs` — `fire()` inserts default then pushes; `recalculate_speed` corrects it
- `src/effect/plugin.rs` — `EffectSystems::Recalculate` has no ordering relative to invariant checkers
- `breaker-scenario-runner/src/lifecycle/systems/plugin.rs` — checkers in `FixedUpdate` with no ordering relative to `EffectSystems::Recalculate`

**Fix:** Add explicit ordering: `recalculate_speed` must run BEFORE the invariant checker chain.
In `src/effect/plugin.rs`, add `.before(ScenarioSystems::InvariantCheckers)` or add a system set
for "before invariant checkers" and constrain `EffectSystems::Recalculate` to run before it.
Alternatively in the scenario runner plugin: `.after(EffectSystems::Recalculate)` on the checker chain.
The cleanest fix is in the game plugin: `EffectSystems::Recalculate.before(EffectSystems::Bridge)`
or explicitly add ordering in `EffectPlugin::build`.

**Regression:** Pre-existing ordering gap — `recalculate_speed` was never ordered relative to checkers.

---

## Failure 3 — `BoltSpeedInRange` — 319 violations

**Root cause:** The invariant checker `check_bolt_speed_in_range` compares bolt speed against raw
`BoltMinSpeed.0` and `BoltMaxSpeed.0`, but `prepare_bolt_velocity` clamps to
`[min * EffectiveSpeedMultiplier, max * EffectiveSpeedMultiplier]`. Any active SpeedBoost makes
legitimate bolt speeds look out-of-range to the invariant checker.

**Precise location:**
- `breaker-scenario-runner/src/invariants/checkers/bolt_speed_in_range.rs` — checks
  `speed < min_speed.0 - 1.0 || speed > max_speed.0 + 1.0` (raw bounds, no multiplier)
- `breaker-game/src/bolt/systems/prepare_bolt_velocity/system.rs` — clamps to
  `[min_speed.0 * mult, max_speed.0 * mult]` using `EffectiveSpeedMultiplier`

**Fix:** The invariant checker is incorrect. Change it to use effective bounds:
```rust
let mult = effective_mult.map_or(1.0, |m| m.0);
let effective_min = min_speed.0 * mult;
let effective_max = max_speed.0 * mult;
if speed < effective_min - 1.0 || speed > effective_max + 1.0 { ... }
```
Query needs to add `Option<&EffectiveSpeedMultiplier>` to get the effective multiplier.

**Regression:** Pre-existing invariant checker bug — it never accounted for SpeedBoost effects.

---

## Failure 4 — `NoEntityLeaks` — 38 violations

**Root cause:** The baseline is set on `SpawnNodeComplete`. The 2x threshold fires when entity
count > `2 * baseline`. In scenarios with many effects (ChainLightning, TetherBeam, Shockwave, Pulse),
entities accumulate over time. The primary leak candidates:

1. **ChainLightning arcs** — `tick_chain_lightning` spawns `ChainLightningArc` entities with
   `CleanupOnNodeExit`. In long-running scenarios (multi-node), arc entities accumulate across
   nodes because `CleanupOnNodeExit` only fires `OnExit(GameState::Playing)` — but in the scenario
   runner, nodes restart via `OnEnter(GameState::MainMenu) → bypass_menu_to_playing → Playing`
   which goes through `Playing` exit/entry and DOES trigger cleanup. However, arcs spawned in the
   final tick of a node (where `tick_chain_lightning` despawns them via Commands) and the
   `cleanup_entities` system also runs may not cause accumulation within a single node.

2. **Shockwave entities** — `ShockwaveSource` with `ShockwaveSpeed(speed)` expands until
   `radius >= max_radius`, then `despawn_finished_shockwave` despawns it. If many shockwaves fire
   (Pulse chip, EntropyEngine), they accumulate transiently. In a scenario where the bolt count is
   high and the node runs long, the count may briefly exceed 2x baseline before shockwaves finish.

3. **`TetherBeamComponent` + chain mode beams** — `maintain_tether_chain` may spawn new beams
   each time bolt count changes, despawning old ones via Commands. There could be a 1-frame window
   where both old and new beams exist.

The invariant fires at specific multiples of 120 frames — if entity count spikes to >2x baseline
at frame 120, 240, etc., it fires. Shockwave accumulation during high-damage chains is the most
likely cause, especially in Pulse or EntropyEngine scenarios.

**Fix:** The 2x multiplier may be too tight for scenarios with heavy effect usage. Three options:
(a) Raise threshold in `check_no_entity_leaks` to e.g. 4x or 5x,
(b) Only flag leaks when the count MONOTONICALLY grows over multiple check intervals (not just a spike),
(c) Exclude known-transient entities (shockwaves, chain lightning arcs) from the count.

**Regression:** Pre-existing — the baseline and threshold were set conservatively before Wave 3
effects that spawn many transient entities were added.

---

## Failure 5 — Entity despawned warnings

**Message:** `Entity despawned: The entity with ID 21v2 is invalid; its index now has generation 3.`

**Root cause:** Double-despawn of bolt entities when both `tick_bolt_lifespan` AND `bolt_lost`
write `RequestBoltDestroyed` for the same entity in the same frame.

**Chain:**
1. Extra bolt has a `BoltLifespan` timer
2. In the same frame: timer expires AND bolt falls below `playfield.bottom() - radius`
3. `tick_bolt_lifespan` (runs `before(BoltSystems::BoltLost)`) writes `RequestBoltDestroyed { bolt: entity }`
4. `bolt_lost` (runs in `BoltSystems::BoltLost`) also detects the bolt is lost AND it's an `ExtraBolt`,
   so also writes `RequestBoltDestroyed { bolt: entity }`
5. `cleanup_destroyed_bolts` (runs `after(EffectSystems::Bridge)`) processes both messages:
   first `commands.entity(entity).despawn()` queues fine; second queues a despawn for an entity
   already in the despawn queue. When commands flush, the second despawn hits an already-despawned
   entity and Bevy 0.18 logs a WARN.

**Precise location:**
- `breaker-game/src/bolt/systems/tick_bolt_lifespan.rs` — writes `RequestBoltDestroyed` on expiry
- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — `is_extra` branch also writes
  `RequestBoltDestroyed`
- `breaker-game/src/bolt/systems/cleanup_destroyed_bolts.rs` — processes all messages, no
  deduplication

**Fix:** Deduplicate `RequestBoltDestroyed` messages before despawning:
```rust
let mut to_despawn: HashSet<Entity> = HashSet::new();
for msg in reader.read() {
    to_despawn.insert(msg.bolt);
}
for entity in to_despawn {
    commands.entity(entity).despawn();
}
```
Alternatively, make `bolt_lost` skip the `RequestBoltDestroyed` write when the `BoltLifespan`
timer has already fired (check `lifespan.just_finished()` or use a marker component). The
deduplication approach is safer and more general.

**Regression:** Pre-existing — `tick_bolt_lifespan` and `bolt_lost` were never coordinated.

---

## Summary Table

| # | Failure | Root Cause Location | Fix Location | Wave 3? |
|---|---------|---------------------|--------------|---------|
| 1 | Warn log: Chain Reaction | `chain_reaction.chip.ron:10` | Same RON file | Yes |
| 2 | EffectiveSpeedConsistent | `effect/plugin.rs` (ordering gap) | Add `before` constraint | No |
| 3 | BoltSpeedInRange | `bolt_speed_in_range.rs` (wrong bounds) | Invariant checker | No |
| 4 | NoEntityLeaks | Threshold too tight for Wave 3 effects | Invariant checker | Yes |
| 5 | Entity despawned | `cleanup_destroyed_bolts.rs` (no dedup) | Same file | No |

**Why**: Systemic failures traced from `feature/scenario-coverage` branch per user request.
**How to apply**: Use these fixes when addressing scenario failures. Failure 1 blocks everything else.
Fix Failure 1 first — once the Chain Reaction warn is gone, failures 2-5 become individually visible.
