# Conductor

Assumes: #2 (`mutators/protocols/conductor/`), #5 (phantom breaker — irrelevant here), #6 (phantom bolt — irrelevant here). Independent of #7's code-driven protocol migration (Conductor isn't tree-declared in RON today).

## What's broken

1. **`PunchScale` VFX insertion is undocumented.** The swap path in `protocol/protocols/conductor/system.rs` inserts a `PunchScale` component on the entity. Design doc does not mention any VFX. Cross-domain write from `protocol/` → `fx/` without design spec or `plugins.md` exception registration.
2. **Direct cross-domain writes from `protocol/` → `bolt/`.** The swap path mutates `PrimaryBolt`/`ExtraBolt` markers, `BoundEffects`/`StagedEffects`, and `CleanupOnExit<RunState>`/`CleanupOnExit<NodeState>` on bolt entities directly from a protocol system. Bolt identity/lifecycle components are owned by the bolt domain and should only be written there.

## Fix

### Delete PunchScale insertion

`mutators/protocols/conductor/system.rs`:
- Remove the `insert(PunchScale { .. })` call and `PunchScale` import.
- Delete any Conductor tests asserting `PunchScale` presence after a swap.

No replacement VFX — design doc doesn't spec one. When Phase 5 VFX work reaches Conductor, it designs a real swap visual and wires an FX-domain listener on the new `ConductorSwapped` message (see below) rather than a cross-domain component insert.

### Introduce `SwapBoltRoles` message in the bolt domain

`bolt/messages.rs`:

```rust
#[derive(Message, Debug, Clone)]
pub struct SwapBoltRoles {
    pub primary: Entity,
    pub extra:   Entity,
}
```

Register via `app.add_message::<SwapBoltRoles>()` in the bolt plugin.

### Bolt-domain consumer

`bolt/systems/apply_role_swap/system.rs` (new):

```rust
pub(crate) fn apply_role_swap(
    mut reader: MessageReader<SwapBoltRoles>,
    mut commands: Commands,
    primary_query: Query<
        (&BoundEffects, &CleanupOnExit<RunState>),
        (With<PrimaryBolt>, Without<ExtraBolt>),
    >,
    extra_query: Query<
        (&StagedEffects, &CleanupOnExit<NodeState>),
        (With<ExtraBolt>, Without<PrimaryBolt>),
    >,
) {
    for msg in reader.read() {
        let Ok((primary_bound, _primary_cleanup)) = primary_query.get(msg.primary) else { continue };
        let Ok((extra_staged, _extra_cleanup)) = extra_query.get(msg.extra) else { continue };

        let primary_bound = primary_bound.clone();
        let extra_staged = extra_staged.clone();

        // Former primary becomes extra.
        commands.entity(msg.primary)
            .remove::<(PrimaryBolt, BoundEffects, CleanupOnExit<RunState>)>()
            .insert((ExtraBolt, StagedEffects::default(), CleanupOnExit::<NodeState>::default()));

        // Former extra becomes primary — carries the primary's effects.
        commands.entity(msg.extra)
            .remove::<(ExtraBolt, StagedEffects, CleanupOnExit<NodeState>)>()
            .insert((PrimaryBolt, primary_bound, CleanupOnExit::<RunState>::default()));

        // NOTE: the above transfers the primary's BoundEffects to the new primary entity.
        // If design requires the former primary to retain its effects as a StagedEffects
        // set, insert `extra_staged` onto msg.primary instead of `StagedEffects::default()`.
        // Confirm against conductor's design doc during implementation — default assumption
        // is "roles swap, effects follow the primary role."
    }
}
```

Schedule: `FixedUpdate`, `.after(ProtocolSystems::DispatchSelection)`. Registered in the bolt plugin.

The bolt-domain consumer owns the canonical list of "what primary-vs-extra means." Future role-divergent components added to the builder (like `CleanupOnExit<T>` variants that don't exist today) only require updating `apply_role_swap`, not every emitter.

### Conductor becomes a thin dispatcher

`mutators/protocols/conductor/system.rs`:
- Delete the direct `PrimaryBolt`/`ExtraBolt`/`BoundEffects`/`StagedEffects`/`CleanupOnExit` write block.
- Delete the `PunchScale` insertion.
- Swap logic collapses to:

```rust
pub(crate) fn conductor_on_swap_trigger(
    // ...existing trigger source (e.g., MessageReader<BumpPerformed> filtered to Perfect)...
    primaries: Query<Entity, With<PrimaryBolt>>,
    extras: Query<Entity, With<ExtraBolt>>,
    mut writer: MessageWriter<SwapBoltRoles>,
) {
    // Conductor's selection logic picks which primary and extra to swap
    // (existing rules unchanged — typically first-primary + nearest-extra
    // or whatever conductor's design specifies).
    for (primary, extra) in conductor_select_swap_pair(&primaries, &extras) {
        writer.write(SwapBoltRoles { primary, extra });
    }
}
```

### plugins.md exception

The Conductor entry in `docs/architecture/plugins.md` §Cross-domain writes (if it exists today — confirm at impl time) is DELETED. The bolt-domain writes now live in the bolt domain; no exception needed.

## Tests

`bolt/systems/apply_role_swap/tests.rs`:

1. **`swap_flips_role_markers`** — spawn primary (`PrimaryBolt` + `BoundEffects` with known entries + `CleanupOnExit<RunState>`) and extra (`ExtraBolt` + `StagedEffects` with known entries + `CleanupOnExit<NodeState>`). Emit `SwapBoltRoles { primary, extra }`. Tick. Assert the former-extra entity now has `PrimaryBolt` + the former primary's `BoundEffects` + `CleanupOnExit<RunState>`; the former-primary entity has `ExtraBolt` + `CleanupOnExit<NodeState>`.
2. **`swap_then_node_exit_despawns_new_extra_only`** — after swap, drive `OnExit(NodeState::Playing)` → `OnEnter(NodeState::Playing)`. Assert the new primary (formerly extra) survives; the new extra (formerly primary) is despawned.
3. **`swap_twice_restores_original_roles`** — emit two `SwapBoltRoles` with swapped arg order. Assert roles flip back. Second node-exit despawns the correct entity.
4. **`swap_with_despawned_entity_is_noop`** — emit `SwapBoltRoles { primary, extra }` where `extra` was despawned in a prior frame. Assert no panic; no role change on `primary`.

`mutators/protocols/conductor/tests/emits_swap.rs`:

5. **`conductor_trigger_emits_one_swap_message`** — activate Conductor; drive its swap trigger; assert exactly one `SwapBoltRoles` message emitted with the right (primary, extra) entity pair.
6. **`conductor_no_swap_when_no_extras`** — spawn primary but no extras. Drive trigger. Assert zero `SwapBoltRoles` messages emitted.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/conductor/system.rs` | DELETE direct bolt-component write block; DELETE `PunchScale` insertion; REWRITE `conductor_on_swap_trigger` to emit `SwapBoltRoles` only |
| `bolt/messages.rs` | ADD `SwapBoltRoles { primary: Entity, extra: Entity }` |
| `bolt/plugin.rs` | Register the message + `apply_role_swap` system |
| `bolt/systems/apply_role_swap/system.rs` | NEW — consumer system |
| `bolt/systems/apply_role_swap/tests.rs` | NEW — tests 1-4 |
| `mutators/protocols/conductor/tests/emits_swap.rs` | NEW — tests 5-6 |
| `docs/architecture/plugins.md` | DELETE Conductor cross-domain-write exception entry (if present) |

## Out of scope

- Future Conductor VFX (Phase 5 combat-effect VFX batch when that work lands)
- Tests against the old `PunchScale` behavior — deleted alongside the insertion
- Conductor's selection logic (which primary + which extra to swap) — unchanged, only the WRITE path changes
