# Protocol: Anchor

## Category
`code-driven` (per TODO #7 — all protocols that were previously `effect-tree` are now code-driven; RON carries tuning only).

## Game Design
You WANT to commit to a position and predict where the bolt will be, rather than chasing reactively.

Stand still for a brief delay, then become "planted." While planted: better bump force, wider perfect bump window, and piercing. Start moving to unplant.

Tuning targets:
- Plant delay: 0.3s of standing still
- Bump force multiplier: 2.0x
- Perfect window multiplier: 1.5x
- Piercing amount: 1

**Rival to Burnout**: both reward stillness with different patterns. Anchor is positional commitment (stay put, predict bolt path); Burnout is heat rhythm (move to build heat, stop to drain + boost).

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct AnchorConfig {
    pub plant_delay: f32,
    pub bump_force_multiplier: f32,
    pub perfect_window_multiplier: f32,
    pub piercing_amount: u32,
}
```

Populated from `ProtocolTuning::Anchor` (RON carries tuning only, no effect tree).

## Components
None owned by Anchor. The planted/unplanted state transition is managed by a `During` condition evaluated in code (TODO #7 dispatcher), which fires and reverses effects via `commands.fire_effect` / `commands.reverse_effect`.

## Messages
**Reads**: Breaker `Velocity2D` (to detect "standing still for N seconds" and "started moving"). `BumpPerformed` is read indirectly through the already-fired effects (bump-force and perfect-window multipliers are applied by the existing bump-grading pipeline once the effects are on the breaker).
**Sends**: None. All effect application flows through `commands.fire_effect` / `commands.reverse_effect` with source `"protocol:anchor"`.

## Systems

### `anchor_evaluate_planted_state`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Anchor)` + `in_state(NodeState::Playing)`.
- **Behavior**: Tracks per-breaker stillness. If the breaker has been stationary (`velocity.length() < epsilon`) for at least `plant_delay` seconds AND is not currently planted: fires `BumpForceBoost`, `PerfectWindowBoost`, and `Piercing` effects on the breaker (and propagates `Piercing` to active bolts) via `commands.fire_effect(breaker, EffectType::*, "protocol:anchor")`. Marks the breaker as planted. If the breaker moves while planted: reverses all three effects via `commands.reverse_effect(breaker, ReversibleEffectType::*, "protocol:anchor")`. Marks as unplanted.
- **Ordering**: `.after(BreakerSystems::Move)` — needs current-frame velocity.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Anchor does not participate in any `DeathPipelineSystems` set.
- **Trigger**: breaker stillness duration.
- **Writes**: source-tagged `EffectStack<SpeedBoostConfig>` / `EffectStack<SizeBoostConfig>`-style entries on breaker + bolts through `commands.fire_effect` / `commands.reverse_effect`. (Specific stack types depend on which effects Anchor composes; the fire/reverse commands upsert by source tag.)
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Cross-Domain Dependencies
- **breaker domain**: Reads `Velocity2D`. The bump-force and perfect-window multipliers are consumed by the existing bump-grading pipeline.
- **bolt domain**: Piercing effect propagates to active bolts via `Route(Bolt, ...)` semantics in the code-driven dispatcher.
- **effect_v3 domain**: Uses `fire_effect` / `reverse_effect` extension methods.

## Expected Behaviors (for test specs)

1. **Breaker plants after standing still for 0.3s** — stationary for `plant_delay`: effects fire; planted marker inserted.
2. **Planted breaker has 2x bump force** — base bump force 500.0 → 1000.0 while planted.
3. **Planted breaker has 1.5x perfect-bump window** — base 0.2s → 0.3s.
4. **Bolts gain Piercing(1) while breaker is planted** — each active bolt has `Piercing(1)` from source `"protocol:anchor"`.
5. **Moving unplants immediately** — `Velocity2D.length() >= epsilon`: all three effects reversed; bump force, perfect window, bolt piercing return to baseline.
6. **Re-planting requires standing still again for `plant_delay`** — stillness timer resets on movement.
7. **Piercing removed from bolt on unplant** — the source-tagged `"protocol:anchor"` entries are cleared; chip-granted piercing unaffected.

## Edge Cases
- **Multiple bolts**: all active bolts gain/lose piercing simultaneously on plant/unplant.
- **Bolt spawned while planted** (e.g., from Fission): the code-driven dispatcher applies piercing to new bolts via `SpawnStampRegistry` (TODO #7) — `Every(Bolt, ...)` semantics.
- **Interaction with existing piercing**: Anchor's piercing stacks additively with chip-granted piercing (both live in the same stack).
- **Dash while planted**: dashing counts as movement — unplants immediately.
- **Node end while planted**: all `"protocol:anchor"` effects cleaned up at node end via `SourceId`-based removal.
- **Interaction with Burnout**: both reward stillness with complementary timing windows (Anchor plants at 0.3s; Burnout's still-drain fires at 1.5s).
- **Bump during plant delay**: if the bolt bumps the breaker before planting completes, the bump uses normal force and window.
