# Bolt force message pipeline

## Problem

Drift and Gravity Surge hazards currently mutate bolt `Velocity2D` directly from hazard systems, bypassing the bolt domain. Per `audit/hazards/drift.md`:

- Issue 1: `drift_apply_force` at `hazard/hazards/drift/system.rs:148-161` writes `Velocity2D` directly; code comment acknowledges "pending `ApplyBoltForce` pipeline."
- Issue 2: Drift's cross-domain `Velocity2D` write is NOT listed in the `plugins.md` Velocity2D exception registry — undocumented violation.
- Issue 5: Gravity Surge has the same root cause (shared concern).

The design intent: hazards emit a message, the bolt domain aggregates per-frame and applies the velocity delta. Bolt state stays owned by the bolt domain.

## Design

### Message

`breaker-game/src/bolt/messages.rs`:

```rust
#[derive(Message, Debug, Clone)]
pub struct ApplyBoltForce {
    pub bolt: Entity,
    /// Force in world-units per second². Aggregated per-frame;
    /// final velocity delta is `force_sum * delta_secs`.
    pub force: Vec2,
}
```

Register via `app.add_message::<ApplyBoltForce>()` in the bolt plugin.

### Consumer system

`breaker-game/src/bolt/systems/apply_bolt_forces/system.rs`:

```rust
pub(crate) fn apply_bolt_forces(
    mut reader: MessageReader<ApplyBoltForce>,
    mut bolts: Query<&mut Velocity2D, With<Bolt>>,
    time: Res<Time<Fixed>>,
) {
    let mut accum: HashMap<Entity, Vec2> = HashMap::new();
    for msg in reader.read() {
        *accum.entry(msg.bolt).or_default() += msg.force;
    }
    let dt = time.delta_secs();
    for (bolt_entity, force_sum) in accum {
        if let Ok(mut velocity) = bolts.get_mut(bolt_entity) {
            velocity.0 += force_sum * dt;
        }
    }
}
```

**Schedule:** `FixedUpdate`, `.after(DriftSystems::ApplyForce).after(GravitySurgeSystems::ApplyForce).before(BoltSystems::IntegrateMotion)`. Match whatever the project's existing integration ordering uses.

Aggregation matters: multiple hazards can emit for the same bolt on the same frame; summing the forces before converting to velocity delta is the canonical "force accumulator" pattern. Applying each message's delta directly would be correct but less efficient and less explicit.

### Drift migration

Open `breaker-game/src/hazard/hazards/drift/system.rs:148-161`. Replace:

```rust
// BEFORE (pseudo — exact current shape)
for (mut velocity, _) in &mut bolts {
    velocity.0 += wind.direction * force * dt;
}

// AFTER
pub(crate) fn drift_apply_force(
    wind: Res<DriftWind>,
    config: Option<Res<DriftConfig>>,
    active: Option<Res<ActiveHazards>>,
    bolts: Query<Entity, With<Bolt>>,
    mut writer: MessageWriter<ApplyBoltForce>,
) {
    let Some(config) = config else { return };
    let Some(active) = active else { return };
    let stacks = active.stacks(HazardKind::Drift);
    let force = config.force_magnitude(stacks);
    if force <= 0.0 { return }

    let force_vec = wind.direction * force;
    for bolt in &bolts {
        writer.write(ApplyBoltForce { bolt, force: force_vec });
    }
}
```

Note: `drift_apply_force` no longer takes `&mut Query<Velocity2D>` or `Res<Time>` (it's a pure emitter now).

### Gravity Surge migration

Locate `gravity_surge_apply_force` (likely `hazard/hazards/gravity_surge/system.rs`). Gravity Surge's force direction/magnitude computation stays in its own system; the final step becomes a message write. Same pattern as Drift.

Exact per-bolt force computation preserved — only the write path changes.

### plugins.md exception registry

`docs/architecture/plugins.md` §Velocity2D Cross-Domain Write Exception:

- Delete the Drift entry (if listed — audit says it ISN'T listed, but verify).
- Delete the Gravity Surge entry (if listed).
- Review every remaining entry — if any are migratable to `ApplyBoltForce`, either include them in this TODO's scope or flag them for follow-up. The registry should shrink to only genuine exceptions (effect-v3 paths, cells, protocol-specific one-offs).

### Design-doc updates

- `docs/design/hazards/drift.md` §Messages already specs `ApplyBoltForce`. Remove the "pending" hedge.
- `docs/design/hazards/gravity_surge.md` §Messages — update to match.
- `docs/architecture/messages.md` (or equivalent) — add `ApplyBoltForce` to the catalog of shared messages; note it's the canonical way to apply forces to bolts from non-bolt domains.

## Tests

`bolt/systems/apply_bolt_forces/tests.rs`:

1. **`single_force_converts_to_velocity_delta`** — emit `ApplyBoltForce { bolt, force: (100, 0) }`, tick one fixed frame (dt = 1/64), assert `velocity.x` increased by `100 / 64 ≈ 1.5625`.
2. **`multiple_forces_aggregate`** — emit three forces for the same bolt: `(50, 0)`, `(50, 0)`, `(0, 100)`. Tick. Assert velocity delta equals `(100/64, 100/64)`.
3. **`missing_bolt_no_panic`** — emit force for a despawned entity. Tick. Assert no panic; no effect on any remaining bolt.
4. **`multiple_bolts_independent`** — emit different forces for two different bolts. Tick. Assert each bolt's velocity delta reflects only its own messages.
5. **`zero_force_no_op`** — emit `ApplyBoltForce { bolt, force: Vec2::ZERO }`. Tick. Assert velocity unchanged.

Drift tests update: any test that asserted `Velocity2D` mutation now asserts `ApplyBoltForce` emission. The end-to-end "velocity actually changes" assertion moves to an integration test that runs `drift_apply_force → apply_bolt_forces` in sequence.

Gravity Surge tests update: same.

## Code change summary

| File | Change |
|------|--------|
| `bolt/messages.rs` | Add `ApplyBoltForce { bolt, force: Vec2 }`. |
| `bolt/plugin.rs` | Register the message. |
| `bolt/systems/apply_bolt_forces/system.rs` | **NEW.** Consumer system. |
| `bolt/systems/apply_bolt_forces/tests.rs` | **NEW.** Unit tests (5 cases above). |
| `bolt/plugin.rs` | Register `apply_bolt_forces` in `FixedUpdate` with ordering constraints. |
| `mutators/hazards/drift/system.rs` | `drift_apply_force` becomes pure emitter. Signature changes (drop `&mut Velocity2D` + `Res<Time>`, add `MessageWriter<ApplyBoltForce>`). |
| `mutators/hazards/gravity_surge/system.rs` | Same pattern. |
| `mutators/hazards/drift/tests.rs` | Rewrite velocity-assertion tests to message-emission assertions. |
| `mutators/hazards/gravity_surge/tests.rs` | Same. |
| `docs/architecture/plugins.md` | Remove Drift + Gravity Surge entries from Velocity2D exception registry. |
| `docs/architecture/messages.md` | Add `ApplyBoltForce` to shared-messages catalog. |
| `docs/design/hazards/drift.md` | Remove "pending" hedge on §Messages. |
| `docs/design/hazards/gravity_surge.md` | Align §Messages. |

## Dependencies

- **TODO #2 (mutators domain refactor)** — hazards live at `mutators/hazards/` post-refactor. Paths assume that. Must land after #2.

## Ordering

Lands after #2. Independent of #1, #3, #4, #5, #6, #7. Can land in parallel with any of them.

## Subsumes

- `audit/remediations/apply-bolt-force-message-pipeline.md`

## Scope boundary

In scope:
- `ApplyBoltForce` message + registration
- `apply_bolt_forces` consumer system + tests + scheduling
- Drift migration to emit
- Gravity Surge migration to emit
- `plugins.md` exception registry entries for Drift/Gravity Surge
- Design-doc alignment for Drift/Gravity Surge §Messages

Out of scope:
- Migrating OTHER cross-domain `Velocity2D` writes (effect-v3 paths, cells, protocol one-offs) — flag any that look migratable, but they ride their own TODOs.
- Any change to Drift's wind direction algorithm or Gravity Surge's force computation — only the write path changes.
- Integration semantics: force-based motion remains a per-frame Euler step. No switch to implicit/semi-implicit integration.

## TODO entry

> **[BLOCKED by #2]** Bolt force message pipeline — `ApplyBoltForce { bolt, force: Vec2 }` message + `apply_bolt_forces` consumer system in bolt domain (aggregates per-frame, writes `Velocity2D` deltas in `FixedUpdate` before `BoltSystems::IntegrateMotion`). Drift and Gravity Surge migrate from direct `Velocity2D` mutation to message emission. `plugins.md` Velocity2D exception registry loses Drift + Gravity Surge entries. Subsumes `apply-bolt-force-message-pipeline.md`. — [detail](detail/apply-bolt-force-pipeline.md)
