# Fission

Assumes: #1 (`rantzsoft_dmg` — `Destroyed<Cell>` is the crate's generic message), #2 (`mutators/protocols/fission/`). Independent of #7 (Fission stays tree-free — it's custom-system, not tree-declared).

## What's broken

1. **Fission-spawned bolts are INVISIBLE.** The spawn call at `system.rs:209` uses `.headless()` (a test-fixture terminal) in production. No `Mesh2d`, no `MeshMaterial2d`, no `GameDrawLayer::Bolt`. Bolts exist in the world, collide, deal damage — but the player never sees them.
2. **`.headless()` is being used in production.** Architectural violation: test-only transitions bleed into runtime.
3. **`BoltRegistry` lookup is wrong-shaped.** Fission spawns from a `BoltRegistry::get(def_ref)` lookup with a `warn!` fallback. Design intent is "split the bolt you already have" — replicate the current primary's tuning/effects, not look up a stored definition.
4. **`FissionCounter` lifecycle is wrong.** Currently reset on `OnExit(MenuState::Main)` (fires at run START, not end). The counter represents progress-toward-next-split within a node; it should reset on node exit, not run entry.
5. **Spawned bolt gets the wrong role.** Current `.extra()` → `ExtraBolt + CleanupOnExit<NodeState>` makes replicas despawn at node end. Design behavior 7 says replicas are "permanent" — should be `PrimaryBolt + CleanupOnExit<RunState>`.
6. **No test covers visibility or permanence.** The invisible-bolts bug shipped unnoticed because nothing asserts `Mesh2d` on replicas; the role-regression could recur because nothing asserts `PrimaryBolt` on replicas.

## Fix

### Add `.replicate_of(entity)` to the bolt builder

`bolt/builder/core/transitions.rs`:

```rust
/// Typestate marker: builder has been configured to copy from a source entity.
pub struct ReplicateSource;

impl BoltBuilder<NoPosition, NoSpeed, NoAngle, NoVelocity, NoRadius, NoVisual, NoRole> {
    /// Configure the builder to read tuning/effect components from `source` at `.spawn()`
    /// time. Position, velocity, and role are NOT copied — the caller sets those explicitly.
    ///
    /// If `source` is despawned or missing a required component at build time, `.spawn()`
    /// returns `Err(BuilderError::SourceUnavailable)` and the caller decides how to handle.
    pub fn replicate_of(self, source: Entity) -> BoltBuilder<NoPosition, NoSpeed, NoAngle, NoVelocity, NoRadius, NoVisual, NoRole, ReplicateSource> {
        // Store `source` in builder state; the terminal reads components via world at spawn.
    }
}
```

The `replicate_of` state carries a single `source: Entity`. At `.spawn()` time, the terminal queries the world for: `BoltDefinitionRef`, `BoltBaseDamage`, `BoltBaseSpeed`, `BoltRadius`, `BoundEffects`, `StagedEffects`, and any other per-bolt tuning components the standard rendered terminal requires. All are cloned onto the new entity.

The builder owns the canonical list of "what counts as a bolt's replicable state." Adding a new per-bolt tuning component in the future only requires updating the builder's replicate-read set — not every caller.

### Fission's spawn path

`mutators/protocols/fission/system.rs`:

```rust
pub(crate) fn fission_split_on_kill(
    mut reader: MessageReader<Destroyed<Cell>>,
    mut counter: ResMut<FissionCounter>,
    config: Res<FissionConfig>,
    primary_query: Query<(Entity, &Position2D, &Velocity2D), With<PrimaryBolt>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _msg in reader.read() {
        counter.kills += 1;
        if counter.kills < config.kills_per_split { continue }
        counter.kills = 0;

        // Pick the first primary bolt as the replication source. If no primary exists,
        // the split is silently skipped — splitting requires something to copy.
        let Some((source, pos, vel)) = primary_query.iter().next() else { continue };

        let rotated = rotate_velocity(vel.0, config.divergence_angle_rad);

        let _ = Bolt::builder()
            .replicate_of(source)
            .at_position(pos.0)
            .with_velocity(Velocity2D(rotated))
            .primary()                                    // NOT .extra()
            .rendered(&mut meshes, &mut materials)        // NOT .headless()
            .spawn(&mut commands);
    }
}
```

Three interlocking decisions pinned at this call site:

- **`.rendered(...)`** — replicas are first-class gameplay bolts. Full mesh, material, `GameDrawLayer::Bolt`.
- **`.replicate_of(source)`** — replicas inherit the current primary's tuning + effects, not some stored definition.
- **`.primary()`** — replicas are permanent (`PrimaryBolt + CleanupOnExit<RunState>`), not node-scoped.

Delete:
- `BoltRegistry::get(def_ref)` lookup and the `warn!` fallback.
- The `parent_def_ref.0` read.
- The `.headless()` terminal call.
- The `.extra()` terminal call.

### Fix `FissionCounter` lifecycle

`mutators/protocols/fission/system.rs`:

```rust
pub(crate) fn fission_reset_counter_on_node_exit(mut commands: Commands) {
    commands.insert_resource(FissionCounter::default());
}
```

Schedule: `OnExit(NodeState::Playing)`, `run_if = protocol_active(Fission)`.

Delete the existing `OnExit(MenuState::Main)` registration. `FissionConfig` (tuning) is unchanged — survives across both node and run boundaries.

### Design doc updates (ride with code)

`docs/todos/detail/mod-system-design/protocols/fission.md`:
- §Components: "`FissionCounter` resets at node exit; does NOT persist across nodes."
- §Game Design: add "Splits replicate the primary bolt — the new bolt is functionally identical to the one you're already shooting with. Only velocity direction differs."
- §Cross-Domain Dependencies: replace `BoltRegistry` language with "Spawns via `Bolt::builder().replicate_of(primary)` which copies tuning + effects from the current primary."
- §Edge Cases: replace "registry miss" language with "If no primary bolt exists at split time, the split is silently skipped."

## Tests

`bolt/builder/tests/replicate_of.rs`:

1. **`replicate_copies_tuning_and_effects`** — spawn source bolt with known `BoltDefinitionRef`, `BoltBaseDamage(25.0)`, `BoltBaseSpeed(400.0)`, populated `BoundEffects`, populated `StagedEffects`. Call `.replicate_of(source).at_position((0,0)).with_velocity((100, 0)).extra().rendered(...).spawn()`. Assert new entity has equivalent `BoltDefinitionRef`, `BoltBaseDamage(25.0)`, `BoltBaseSpeed(400.0)`, same `BoundEffects` entries, same `StagedEffects` entries.
2. **`replicate_does_not_copy_position_or_velocity`** — same setup. Assert new entity's `Position2D == (0, 0)` (NOT source's position); `Velocity2D == (100, 0)` (NOT source's velocity).
3. **`replicate_with_despawned_source_returns_err`** — despawn source before `.spawn()`. Assert `.spawn()` returns `Err(BuilderError::SourceUnavailable)`.

`mutators/protocols/fission/tests/visibility.rs`:

4. **`fission_replicas_are_rendered`** — activate Fission + spawn primary bolt; drive `kills_per_split` `Destroyed<Cell>` messages. Query all bolts spawned this frame (not the primary). Assert each has `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Bolt`.

`mutators/protocols/fission/tests/replicates_primary.rs`:

5. **`replica_inherits_primary_tuning`** — primary with `BoltBaseDamage(42.0)` + `BoundEffects` containing Piercing. Trigger one split. Assert replica's `BoltBaseDamage == 42.0`; replica's `BoundEffects` contains the same Piercing entry.
6. **`replica_velocity_is_rotated_from_primary`** — primary with velocity `(300, 0)`. Trigger split with `divergence_angle_rad = 0.5`. Assert replica's velocity magnitude equals primary's (`300`); direction differs by `0.5` rad.

`mutators/protocols/fission/tests/permanence.rs`:

7. **`replica_has_primary_role_and_run_cleanup`** — trigger split. Query the replica. Assert `PrimaryBolt` present; `ExtraBolt` absent; `CleanupOnExit<RunState>` present; `CleanupOnExit<NodeState>` absent; no bolt-lifespan-like component.
8. **`replica_survives_node_exit`** — trigger split; drive `OnExit(NodeState::Playing)` → `OnEnter`. Assert replica still alive.

`mutators/protocols/fission/tests/counter_lifecycle.rs`:

9. **`counter_resets_at_node_exit`** — set `FissionCounter { kills: 5 }`; drive `OnExit(NodeState::Playing)`; assert `FissionCounter.kills == 0`.
10. **`counter_persists_mid_node_state_transitions`** — set `FissionCounter { kills: 3 }`; transition to paused (NOT node exit); tick; assert `kills == 3`.

`mutators/protocols/fission/tests/no_primary_no_split.rs`:

11. **`split_with_no_primary_is_silent_noop`** — activate Fission; drive `kills_per_split` `Destroyed<Cell>` messages with no primary bolt in the world. Assert zero new bolt entities spawned; assert no `warn!` or panic.

### Tests to DELETE

- Any existing Fission test asserting `.headless()`, `ExtraBolt`, `CleanupOnExit<NodeState>` on replicas — those pin buggy behavior.
- Any existing test exercising the `BoltRegistry::get` warn path — the path is gone.
- The existing `tests/persistence.rs` (if it asserts counter-persists-across-nodes) — flipped to test #9 above.

## Code changes summary

| File | Change |
|------|--------|
| `bolt/builder/core/transitions.rs` | Add `ReplicateSource` typestate + `.replicate_of(source)` method |
| `bolt/builder/core/terminal.rs` | Terminal reads `replicate_of` source at spawn; clones tuning + effects |
| `bolt/builder/core/types.rs` | Add `replicate_source: Option<Entity>` field; add `BuilderError::SourceUnavailable` variant |
| `mutators/protocols/fission/system.rs` | REWRITE `fission_split_on_kill`; REWRITE `fission_reset_counter_on_node_exit` (new schedule + body) |
| `mutators/protocols/fission/register.rs` | Register reset on `OnExit(NodeState::Playing)` instead of `OnExit(MenuState::Main)` |
| `mutators/protocols/fission/tests/visibility.rs` | NEW — test 4 |
| `mutators/protocols/fission/tests/replicates_primary.rs` | NEW — tests 5-6 |
| `mutators/protocols/fission/tests/permanence.rs` | NEW — tests 7-8 |
| `mutators/protocols/fission/tests/counter_lifecycle.rs` | NEW — tests 9-10 |
| `mutators/protocols/fission/tests/no_primary_no_split.rs` | NEW — test 11 |
| `bolt/builder/tests/replicate_of.rs` | NEW — tests 1-3 |
| `docs/todos/detail/mod-system-design/protocols/fission.md` | Design-doc updates per Fix section |

## Out of scope

- Divergence-angle RON tuning (→ `ron-tuning-values.md`)
- Conductor interaction (Fission replica + Conductor role marker semantics — already noted in design doc, no code change here)
- `.headless()` deprecation across the codebase (separate sweep)
