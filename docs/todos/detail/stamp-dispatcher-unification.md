# Stamp dispatcher unification + code-driven protocol migrations

## Problem

The effect_v3 dispatch system has a flagship bug: `StampTarget` is IGNORED by four of the five dispatch sites, so `Stamp(EveryBolt, tree)` and `Stamp(EveryCell, tree)` roots never reach their intended targets. Instead:

- **Site D (protocol)** — `breaker-game/src/protocol/systems/dispatch_protocol_selection.rs:52-56` pattern-matches `RootNode::Stamp(_target, tree)` with the target BOUND with `_`, then stamps on EVERY Breaker regardless. Every protocol whose RON uses `Stamp(EveryBolt, ...)` (Anchor, Deadline, Kickstart, Ricochet) currently installs its effects on the wrong entity.
- **Site A (chip)** — `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs:60-79` resolves `StampTarget::Breaker` correctly but for every NON-Breaker target falls back to stamping on the Breaker (line 76-78) and wraps in an `On(target, ...)` re-dispatch at trigger time. The resolver at `resolve_target_entities` exists but collapses `Bolt | ActiveBolts | EveryBolt` → `With<Bolt>`, ignoring the spawn-watching semantics of `Every*`.
- **Site B (bolt_def)** — `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs:54` calls `commands.stamp_effect(entity, String::new(), tree.clone())` with an EMPTY source string. The resolver at lines 38-51 is duplicated (three copies across sites A/B/C). This site is also REDUNDANT: every bolt spawns through the bolt builder, which processes def.effects at terminal time — Site B just re-dispatches the same effects a tick later.
- **Site C (cell_def)** — `breaker-game/src/state/run/node/systems/dispatch_cell_effects/system.rs:76` same empty source string, same redundancy with the cell builder terminal, plus `breaker_query.single()` (panics with multiple breakers).
- **Site F (builder terminals)** — `breaker-game/src/breaker/builder/core/terminal.rs:335` matches `RootNode::Stamp(_target, tree)` with `_` and always stamps on `entity` (the entity being built). Works only when the definition's `StampTarget` is `Self` (semantically `Bolt` on a bolt builder, `Breaker` on a breaker builder, etc.). Silently wrong for any other target.

The `SpawnStampRegistry` at `breaker-game/src/effect_v3/storage/spawn_stamp_registry/` is FULLY implemented with working watchers for bolts/cells/walls/breakers (`stamp_spawned_bolts`, `stamp_spawned_cells`, etc.) — but no dispatch site populates `registry.entries`, so the `Every*` variants never trigger retroactive stamping for future-spawned entities.

Four protocols that depended on `Stamp(EveryBolt, ...)` or similar cross-entity stamps are broken:
- **Deadline** — `StampTarget::EveryBolt` ignored; boost lands on breaker; `Until(TimeExpires, ...)` never reverses because no production path arms `EffectTimers`.
- **Kickstart** — same dispatch bug; countdown should start on first bump, not node start; RON uses wrong primitive.
- **Ricochet** — same dispatch bug; tree never activates; all 7 design behaviors fail; `Until` re-entry semantics unverified.
- **Anchor** — same dispatch bug; `Stamp(EveryBolt, Fire(Piercing))` lands on breaker, duplicating `tick_anchor`'s direct `EffectStack` push. `bump_force_multiplier` read from RON but never applied. Design references `During(StillFor, …)` effect-system primitive that doesn't exist.

Per user direction, ALL four protocols migrate to CODE-DRIVEN (canonical, not a workaround). RON becomes tuning-only; systems fire/reverse effects via `commands.fire_effect` / `commands.reverse_effect` in response to game events. With every protocol code-driven, Site D (`dispatch_protocol_selection`) no longer needs ANY effect-tree dispatch — it just inserts into `ActiveProtocols` and calls `protocols::activate(kind, &tuning, &mut commands)`. The effect-tree dispatch branch is deleted.

Chips and definition-based entities (bolt/cell/breaker/wall) remain as the `stamp_root` facade's consumers.

Dispatch is load-bearing across the entire game: every chip, every bolt/cell definition, every tree-using protocol, every builder terminal with RON-sourced effects goes through this code path. The unification must be precise.

## Design

### Canonical resolver: `resolve_stamp_target`

**Location:** `breaker-game/src/effect_v3/commands/stamp_root/resolve.rs` (new file in a new `stamp_root/` sub-module under `commands/`).

**Signature:**

```rust
pub(crate) fn resolve_stamp_target(
    target: StampTarget,
    world: &World,
) -> Vec<Entity>
```

**Body — the single source of truth:**

```rust
pub(crate) fn resolve_stamp_target(
    target: StampTarget,
    world: &World,
) -> Vec<Entity> {
    match target {
        // Breaker family → With<Breaker>
        StampTarget::Breaker
        | StampTarget::ActiveBreakers
        | StampTarget::EveryBreaker => world
            .iter_entities()
            .filter(|e| world.get::<Breaker>(e.id()).is_some())
            .map(|e| e.id())
            .collect(),

        // Bolt family → With<Bolt>
        StampTarget::Bolt
        | StampTarget::ActiveBolts
        | StampTarget::EveryBolt => world
            .iter_entities()
            .filter(|e| world.get::<Bolt>(e.id()).is_some())
            .map(|e| e.id())
            .collect(),

        // Cell family → With<Cell>
        StampTarget::ActiveCells
        | StampTarget::EveryCell => world
            .iter_entities()
            .filter(|e| world.get::<Cell>(e.id()).is_some())
            .map(|e| e.id())
            .collect(),

        // Wall family → With<Wall>
        StampTarget::ActiveWalls
        | StampTarget::EveryWall => world
            .iter_entities()
            .filter(|e| world.get::<Wall>(e.id()).is_some())
            .map(|e| e.id())
            .collect(),
    }
}
```

(Alternative: use `world.query_filtered::<Entity, With<T>>()` — prefer whichever is idiomatic in the existing `effect_v3` codebase; the iter-entities form above is shown for explicitness.)

**Rules:**

- `resolve_stamp_target` is the ONLY function that maps `StampTarget` variants to entity sets. The duplicated inline matches in Sites A/B/C/D/F are ALL deleted.
- `Bolt`, `ActiveBolts`, `EveryBolt` ALL collapse to `With<Bolt>` today. The RON-level distinction (singular vs active vs every) is preserved in the enum for expressive RON intent but has no runtime difference in the resolver output itself. The `Every*` SEMANTIC (retroactive stamping of future-spawned entities) is handled separately by `SpawnStampRegistry` population (see next section).
- `StampTarget` enum is NOT simplified to 4 variants. Keep all 10.

### `Every*` → `SpawnStampRegistry` registration

**When a stamp root is dispatched with an `Every*` variant, the caller must additionally register the `(kind, name, tree)` triple with `SpawnStampRegistry` so newly-spawned entities get the tree.**

**Helper function:** `breaker-game/src/effect_v3/commands/stamp_root/register_every.rs`

```rust
pub(crate) fn register_every_if_applicable(
    target: StampTarget,
    source: &str,
    tree: &Tree,
    registry: &mut SpawnStampRegistry,
) {
    let kind = match target {
        StampTarget::EveryBolt => Some(EntityKind::Bolt),
        StampTarget::EveryCell => Some(EntityKind::Cell),
        StampTarget::EveryWall => Some(EntityKind::Wall),
        StampTarget::EveryBreaker => Some(EntityKind::Breaker),
        // Singular + `Active*` variants do NOT register — they're one-shot.
        _ => None,
    };
    if let Some(kind) = kind {
        registry.entries.push((kind, source.to_owned(), tree.clone()));
    }
}
```

**`Active*` variants do NOT register with `SpawnStampRegistry`.** `Active*` means "everything that exists RIGHT NOW." Newly-spawned entities after the stamp call do NOT receive the tree under `Active*`. This is the defining semantic split between `ActiveBolts` (one-shot snapshot) and `EveryBolt` (snapshot + retroactive).

**Existing watchers** at `effect_v3/storage/spawn_stamp_registry/watchers/stamp_spawned_{bolts,cells,walls,breakers}.rs` stay unchanged. They already iterate `Added<T>` and call `commands.stamp_effect(entity, name.clone(), tree.clone())` for matching entries. Currently dead code because nothing populates the registry; this TODO wires them into production.

### Commands extension: `stamp_root` + `stamp_roots`

**Location:** ADD to existing trait `EffectCommandsExt` in `breaker-game/src/effect_v3/commands/ext/system.rs:15-50`. Do NOT create a new trait. Do NOT create a new commands-extension file. The existing `EffectCommandsExt` is the one place all effect commands live; `stamp_root` joins `fire_effect`/`reverse_effect`/`stamp_effect`/`route_effect`/`stage_effect`/`remove_effect`/`remove_staged_effect`/`track_armed_fire`.

**New trait methods (added to `EffectCommandsExt`):**

```rust
pub trait EffectCommandsExt {
    // ... existing methods ...

    /// Dispatch a single `RootNode` — the canonical stamp/spawn entry point.
    /// Resolves the target via `resolve_stamp_target`, calls `stamp_effect`
    /// on every resolved entity, and registers with `SpawnStampRegistry`
    /// if the target is an `Every*` variant.
    ///
    /// `Stamp(target, tree)` → resolve + stamp each entity + (maybe) register.
    /// `Spawn(kind, tree)` → register with `SpawnStampRegistry`; the existing
    /// `stamp_spawned_*` watchers apply it to newly-spawned entities.
    fn stamp_root(&mut self, root: &RootNode, source: String);

    /// Convenience: dispatch every root in a slice with the same source.
    /// Implementation calls `stamp_root` for each.
    fn stamp_roots(&mut self, roots: &[RootNode], source: String);
}
```

**Implementation:** `breaker-game/src/effect_v3/commands/stamp_root/system.rs`

```rust
impl EffectCommandsExt for Commands<'_, '_> {
    // ... existing impls ...

    fn stamp_root(&mut self, root: &RootNode, source: String) {
        self.queue(StampRootCommand {
            root: root.clone(),
            source,
        });
    }

    fn stamp_roots(&mut self, roots: &[RootNode], source: String) {
        for root in roots {
            self.stamp_root(root, source.clone());
        }
    }
}
```

**Deferred `Command` type:** `breaker-game/src/effect_v3/commands/stamp_root/command.rs`

```rust
pub(crate) struct StampRootCommand {
    pub root: RootNode,
    pub source: String,
}

impl Command for StampRootCommand {
    fn apply(self, world: &mut World) {
        match self.root {
            RootNode::Stamp(target, tree) => {
                // 1. Resolve target entities (snapshot for Active*/Every*/singular).
                let entities = resolve_stamp_target(target, world);

                // 2. Stamp the tree onto each entity — go through the existing
                //    StampEffectCommand so BoundEffects insertion and name
                //    handling match every other call site.
                for entity in entities {
                    world.commands().stamp_effect(
                        entity,
                        self.source.clone(),
                        tree.clone(),
                    );
                }

                // 3. For Every*, register with SpawnStampRegistry so future
                //    spawns receive the tree via the existing watchers.
                if let Some(mut registry) = world.get_resource_mut::<SpawnStampRegistry>() {
                    register_every_if_applicable(
                        target,
                        &self.source,
                        &tree,
                        &mut registry,
                    );
                }
            }
            RootNode::Spawn(kind, tree) => {
                // Spawn roots go straight to the registry; the existing
                // stamp_spawned_* watchers handle application.
                if let Some(mut registry) = world.get_resource_mut::<SpawnStampRegistry>() {
                    registry.entries.push((kind, self.source.clone(), tree));
                }
            }
        }
    }
}
```

**Module wiring:** `breaker-game/src/effect_v3/commands/stamp_root/mod.rs`

```rust
mod command;
mod register_every;
mod resolve;
#[cfg(test)]
mod tests;

pub(crate) use command::StampRootCommand;
pub(crate) use register_every::register_every_if_applicable;
pub(crate) use resolve::resolve_stamp_target;
```

Re-export from `effect_v3/commands/mod.rs` so `StampRootCommand` is visible to `ext/system.rs`. The public API surface (`stamp_root`/`stamp_roots`) is on the trait; the command type and helpers stay `pub(crate)`.

### Source string conventions — pin them hard

Every dispatch site writes a source string identifying the origin of the stamp. These strings are consumed by:
- Source-tagged upsert semantics in `EffectStack::fire_effect` (replace-same-source, not stack).
- Downstream reverse paths that target `"protocol:Kickstart"` entries specifically.
- Debug/log output.

**Canonical format per dispatch site (post-migration):**

| Site | Format | Example |
|------|--------|---------|
| A (chip) | `"chip:{name}"` | `"chip:overcharge"` |
| D (protocol) | `"protocol:{kind:?}"` | `"protocol:Anchor"` |
| F (breaker builder) | `"breaker_def:{id}"` | `"breaker_def:aegis"` |
| F (bolt builder) | `"bolt_def:{id}"` | `"bolt_def:standard"` |
| F (cell builder) | `"cell_def:{alias}"` | `"cell_def:steel"` |
| F (wall builder) | `"wall_def:{id}"` | `"wall_def:default"` |
| F (required breaker effects) | `"breaker_def:{id}:{bolt_lost\|salvo_hit}"` | `"breaker_def:chrono:bolt_lost"` |

All sites call `commands.stamp_root(root, source)` (or `stamp_roots(&roots, source)`). Empty source strings were possible at Sites B/C/F — Sites B/C are deleted; Site F gains real sources. Non-empty sources are now an invariant enforced by the facade callers.

### Per-site migration — exact changes

#### Site A: chip dispatch (`chips/systems/dispatch_chip_effects/system.rs`)

**Current** (lines 58-86): matches `RootNode::Stamp(target, tree)`, branches on `target == StampTarget::Breaker` — if Breaker, resolves via `resolve_target_entities` + calls `dispatch_tree` per entity; otherwise stamps on every breaker as a deferred bridge. Handles bare `Fire` at line 109 via `commands.fire_effect`.

**After:** Replace the entire root-dispatch loop with:

```rust
for root in &effects {
    commands.stamp_root(root, format!("chip:{}", chip_name));
}
```

**Loss of explicit `Fire` short-circuit:** `dispatch_tree` at line 93 currently treats `Tree::Fire(effect)` as immediate `fire_effect`. `stamp_root` always goes through `stamp_effect`. This is intentional: `Fire(effect)` as a ROOT-LEVEL child of `Stamp(...)` means the effect fires at the stamp time, which for the effect-walking pipeline is equivalent to a one-tick-delayed stamp + walk. If current tests rely on the immediate-fire semantic, verify via the integration tests whether behavior changes observably; if yes, `stamp_root` gets a special-case for `Stamp(target, Tree::Fire(effect))` that delegates to `fire_effect` on each resolved entity. Default assumption: no behavior change (the walker catches it on the next tick).

**Delete:** `resolve_target_entities` function (lines 119-130), `DispatchTargets` SystemParam struct (lines 19-25, no longer needed since `stamp_root` resolves via world), `dispatch_tree` function (lines 93-116 — the Fire short-circuit logic). `dispatch_chip_effects` loses its `targets: DispatchTargets` parameter.

**Deferred-dispatch workaround:** The "`StampTarget != Breaker` → stamp on Breaker + wrap in On-bridge" path (lines 67-79) was a workaround for the fact that non-Breaker entities don't exist yet during ChipSelect. With `EveryBolt` → `SpawnStampRegistry` registration, this workaround becomes unnecessary: the chip registers with the registry at select time; the watchers apply the tree when bolts spawn. Delete the workaround branch entirely.

#### Sites B & C: DELETED

**`dispatch_bolt_effects` and `dispatch_cell_effects` are redundant and get deleted entirely.** Bolts and cells always spawn through their builders, and the builder terminals are now full dispatch sites that call `stamp_root`. There is no scenario where a bolt or cell appears in the world without its builder running. A parallel `Added<BoltDefinitionRef>` / `Added<Cell>` watcher re-dispatching the same def.effects is pure duplication.

**Files deleted:**
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs`
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/tests/` (entire directory — `basic_dispatch.rs`, `edge_cases.rs`, `entity_targeting.rs`)
- `breaker-game/src/state/run/node/systems/dispatch_cell_effects/system.rs`
- `breaker-game/src/state/run/node/systems/dispatch_cell_effects/tests/` (entire directory)
- `CellEffectsDispatched` marker component (unused after Site C deletion)
- Any plugin registration lines wiring these systems into the schedule

**Retargeted tests:** the useful coverage from the deleted test directories (e.g., "def with `Stamp(EveryBolt, ...)` reaches bolts") moves to the appropriate builder terminal tests under Site F. Tests that exercised the dispatch-bridge-specific behavior (e.g., "`CellEffectsDispatched` marker prevents double-dispatch") are deleted outright — the phenomenon doesn't exist anymore.

**What this leaves:** builder terminals are THE dispatch path for every definition-based entity (bolt, breaker, cell, wall). `Every*` variants populate `SpawnStampRegistry` at builder-spawn time; the existing `stamp_spawned_*` watchers apply trees to future-spawned entities. No post-spawn re-dispatch needed.

**`BoltDefinitionRef` is still inserted by the bolt builder** — other systems (e.g., bolt identity, debug tooling) may consume it for non-effect purposes. This TODO does not touch its lifecycle; only the `Added<BoltDefinitionRef>` effect-dispatch watcher goes away.

#### Site D: protocol dispatch (`protocol/systems/dispatch_protocol_selection.rs`)

**Current** (lines 52-63): `_target` ignored, always stamps on every breaker. Falls through to `protocols::activate(kind, &tuning, &mut commands)` only when `effects.is_none()`.

**After:** Since every protocol is code-driven (Anchor + Deadline + Kickstart + Ricochet + the custom-system protocols that already live at `protocols::activate`), Site D no longer dispatches ANY effect trees. The effects branch is deleted outright; every protocol goes through `protocols::activate`:

```rust
pub(crate) fn dispatch_protocol_selection(
    mut reader: MessageReader<ProtocolSelected>,
    registry: Res<ProtocolRegistry>,
    mut active: ResMut<ActiveProtocols>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let kind = msg.kind;
        let Some(def) = registry.get(kind) else {
            warn!("protocol {kind:?} not found in ProtocolRegistry");
            continue;
        };
        active.insert(def.clone());
        protocols::activate(kind, &def.tuning, &mut commands);
    }
}
```

**Deleted:**
- `breakers: Query<Entity, With<Breaker>>` param.
- `let effects = def.tuning.effects().map(...)` + the branching on `Some(roots)` vs `None`.
- The `for root in roots { RootNode::Stamp(_target, tree) => ... }` loop entirely.
- `ProtocolTuning::effects()` method (every caller gone). The `effects` field on each `ProtocolTuning` variant is also gone — tuning variants carry only tuning fields.
- `RootNode` import from this file.

Site D is no longer a `stamp_root` caller. It's purely a `ProtocolSelected → ActiveProtocols + activate()` bridge.

#### Site F: builder terminals (breaker + bolt + cell + wall) — THE dispatch path for defs

**Current:** Breaker terminal `stamp_root_nodes` at `breaker/builder/core/terminal.rs:332-339` matches `RootNode::Stamp(_target, tree)` with `_` and stamps on `entity` unconditionally. Similar pattern in bolt/cell/wall builder terminals. `_target` is thrown away, so any RON declaring `Stamp(EveryBolt, ...)` or `Stamp(EveryCell, ...)` on a def silently mis-applies onto the entity being built.

**After:** Builder terminals forward every root to `commands.stamp_root(root, source)` — the exact same facade Sites A and D use. No target-matching logic, no warnings, no special case, no `_entity` parameter. `stamp_root` runs as a deferred `Command` with full `&mut World` access, so it resolves any `StampTarget` variant correctly at flush time, populates `SpawnStampRegistry` for `Every*` variants, and applies the tree on every resolved entity:

```rust
fn stamp_root_nodes(
    commands: &mut Commands,
    effects: &[RootNode],
    source: String,
) {
    for root in effects {
        commands.stamp_root(root, source.clone());
    }
}
```

**Self-targeting works automatically.** `Stamp(Breaker, tree)` on a breaker build resolves `With<Breaker>` at flush time, and by then the newly-spawned breaker is in the world (commands flush FIFO: `Commands::spawn` queued first runs before the later-queued `StampRootCommand`). Same for bolt/cell/wall self-targets.

**Cross-entity stamping works automatically.** A breaker RON declaring `Stamp(EveryBolt, ...)` resolves `With<Bolt>` at flush time AND registers `(EntityKind::Bolt, source, tree)` with `SpawnStampRegistry`. Current bolts get the tree via the resolver; future bolts get it via `stamp_spawned_bolts`.

**`Every*` retroactivity is the registry's job.** That's the whole reason `SpawnStampRegistry` + its watchers exist. The builder's one job is "call `stamp_root` with the right source string"; the facade and the registry handle the rest.

**Source string at builder time:** `format!("{kind_prefix}_def:{def_id}")` where the builder terminal captures the definition ID at `.definition(def)` time. If no definition ID is available (manual builder chains with `.with_effects(...)`), source is `format!("{kind}_builder:manual")` — never empty.

**`stamp_required_effects`** at `breaker/builder/core/terminal.rs:320-328` gets the same treatment. `bolt_lost` and `salvo_hit` authored as `Stamp(Breaker, tree)` resolve to the just-spawned breaker via the `With<Breaker>` iteration. Any other target works too, without further code changes:

```rust
fn stamp_required_effects(commands: &mut Commands, optional: &OptionalBreakerData, def_id: &str) {
    for root in &optional.bolt_lost {
        commands.stamp_root(root, format!("breaker_def:{def_id}:bolt_lost"));
    }
    for root in &optional.salvo_hit {
        commands.stamp_root(root, format!("breaker_def:{def_id}:salvo_hit"));
    }
}
```

**Four builder terminals get the identical one-line-forward treatment:**
- `breaker-game/src/breaker/builder/core/terminal.rs:320-339` (`stamp_required_effects` + `stamp_root_nodes`)
- `breaker-game/src/bolt/builder/core/terminal.rs` (locate by grepping `RootNode::Stamp` in bolt builder)
- `breaker-game/src/cells/builder/core/terminal.rs` (locate by grepping `RootNode::Stamp` in cells builder)
- `breaker-game/src/walls/builder/core/terminal.rs` (locate by grepping `RootNode::Stamp` in walls builder)

All four already have `commands: &mut Commands` in their terminal signatures. The call is `commands.stamp_root(root, source.clone())` — nothing more.

**Clean dispatch table (post-migration):**

| Entity kind | Dispatch path | Source prefix |
|-------------|---------------|---------------|
| Breaker (def-based) | Site F (builder terminal) | `breaker_def:{id}` |
| Bolt (def-based) | Site F (builder terminal) | `bolt_def:{id}` |
| Cell (def-based) | Site F (builder terminal) | `cell_def:{alias}` |
| Wall (def-based) | Site F (builder terminal) | `wall_def:{id}` |
| Chip | Site A (`dispatch_chip_effects`) | `chip:{name}` |
| Protocol | Site D (`dispatch_protocol_selection`) | `protocol:{kind:?}` |

Sites B and C DELETED. No double-dispatch. No redundant watchers. Builders are the one and only def-dispatch path.

### Deleted code (after migration)

- `dispatch_chip_effects`: `resolve_target_entities`, `DispatchTargets` SystemParam, `dispatch_tree` — all replaced by the `stamp_roots` one-liner.
- `bolt/systems/dispatch_bolt_effects/` — **entire directory** (system + tests).
- `state/run/node/systems/dispatch_cell_effects/` — **entire directory** (system + tests).
- `CellEffectsDispatched` marker component — unused after Site C deletion.
- Plugin schedule entries for `dispatch_bolt_effects` and `dispatch_cell_effects`.
- `dispatch_protocol_selection`: `breakers: Query<Entity, With<Breaker>>` param; the `for breaker_entity in breakers.iter()` stamp loop.
- Regression-trap tests at `protocol/systems/dispatch_protocol_selection/tests.rs:442-481` that pin the `_target`-ignored behavior.
- RON chip file `breaker-game/assets/chips/standard/ricochet_protocol.chip.ron` (legacy from when Ricochet was a chip; verify no load path references it).
- `assets/protocols/deadline.protocol.ron` — effect tree deleted, tuning-only.
- `assets/protocols/kickstart.protocol.ron` — effect tree deleted, tuning-only.
- `assets/protocols/ricochet.protocol.ron` — effect tree deleted, tuning-only.

### Code-driven migrations (parallel to dispatcher work)

Four protocols peel off the dispatcher entirely — Anchor, Deadline, Kickstart, Ricochet. With all four code-driven, zero protocols use `stamp_root`. Each gets a module at `mutators/protocols/<name>/` (the `mutators/` relocation comes from TODO #2; Anchor's module already exists and needs finishing, the others are new).

#### Anchor (`mutators/protocols/anchor/`)

Anchor already has an `AnchorPlanted`/`AnchorActive` component pair + `tick_anchor` + `detect_breaker_movement` systems. This TODO finishes the code-driven migration by removing the RON effect tree, wiring `bump_force_multiplier` into bump impulse, and routing piercing through `fire_effect` instead of direct `EffectStack` manipulation.

**`AnchorConfig` resource** (tuning-only, loaded from RON):

```rust
pub(crate) struct AnchorConfig {
    pub still_threshold:           f32,  // velocity magnitude threshold for "still"
    pub still_for_secs:            f32,  // duration under threshold to plant
    pub perfect_window_multiplier: f32,  // while planted
    pub bump_force_multiplier:     f32,  // while planted
    pub piercing_amount:           u32,  // charges granted per plant
}
```

**Systems** (all `FixedUpdate`, `run_if = protocol_active(Anchor) + in_state(NodeState::Playing)` unless noted):

1. **`tick_anchor`** — EXISTS. Tracks time-under-threshold on the breaker; inserts `AnchorPlanted` + `AnchorActive { bump_force_multiplier, perfect_window_multiplier }` after `still_for_secs`.
2. **`detect_breaker_movement`** — EXISTS. Removes `AnchorPlanted` + `AnchorActive` on the first frame breaker velocity exceeds `still_threshold`.
3. **`anchor_on_plant_fires_piercing`** — NEW. Reads `Added<AnchorPlanted>`. For every `Query<Entity, With<Bolt>>`: `commands.fire_effect(bolt, EffectType::Piercing(PiercingConfig { count: config.piercing_amount }), "protocol:Anchor:piercing".to_owned())`. Source-tagged upsert replaces any prior `"protocol:Anchor:piercing"` entry — re-plant is idempotent, no accumulation.
4. **`anchor_on_bolt_spawn`** — NEW. Reads `Added<Bolt>` (or `BoltSpawned`). If any breaker has `AnchorPlanted`: fire the same piercing on the new bolt. Handles Fission/Afterimage-spawned bolts mid-plant.

**Piercing lifecycle:** Once granted, the `"protocol:Anchor:piercing"` entry lives on the bolt until normal piercing consumption in `bolt_cell_collision` drains `count` to 0. Unplant does NOT remove the entry (per user-directed design update 2026-04-21). This matches updated design behavior 7.

**`bump_force_multiplier` wiring:** `grade_bump` in `breaker/systems/bump/system.rs` reads `Option<&AnchorActive>` on the bumping breaker. If present, multiply the computed bump impulse by `anchor_active.bump_force_multiplier` before emitting `BumpPerformed`. Applies regardless of grade (Perfect/Early/Late) — Anchor is a planted-state bonus, not a grade-specific one.

**`perfect_window_multiplier` wiring:** `update_bump` reads `Option<&AnchorActive>` on each breaker. If present, multiply the active perfect-window duration by `anchor_active.perfect_window_multiplier`. Already wired per the existing `AnchorActive` integration — verify.

**RON** (`assets/protocols/anchor.protocol.ron`):

```ron
(
    still_threshold:           1.5,
    still_for_secs:            0.3,
    perfect_window_multiplier: 2.0,
    bump_force_multiplier:     1.5,
    piercing_amount:           5,
)
```

No `Root(...)`, `Stamp(...)`, effect tree of any kind. Description rewritten to match design: "Stand still to plant. While planted: wider perfect window, stronger bumps, and bolts pierce through cells."

**Delete** from Anchor's runtime: any direct `EffectStack<PiercingConfig>` pushes in `tick_anchor` or elsewhere. Piercing mutation flows through `fire_effect` only.

**Delete** the `During(StillFor, …)` reference from `docs/design/protocols/anchor.md` — that primitive never existed; hand-rolled plant/unplant IS the canonical approach.

**Tests** (`anchor/tests/`): `plant_grants_piercing_via_fire_effect`, `unplant_preserves_piercing`, `replant_is_idempotent_no_accumulation`, `piercing_consumed_by_cell_passthrough`, `bump_force_multiplier_applied_while_planted`, `bump_force_unmodified_when_unplanted`, `perfect_window_multiplier_applied_while_planted`, `late_spawned_bolt_gets_piercing_mid_plant`, `chip_piercing_survives_anchor_cycles`.

#### Deadline (`mutators/protocols/deadline/`)

**Module structure** (same layout Anchor uses — the canonical code-driven protocol template):
```
mutators/protocols/deadline/
  mod.rs            // wiring
  resources.rs      // DeadlineConfig, DeadlineArmed
  system.rs         // the three systems
  register.rs       // activate() + system registration
  tests/
```

**`DeadlineConfig` resource** (inserted by `activate()` from `ProtocolTuning::Deadline`):

```rust
pub(crate) struct DeadlineConfig {
    pub threshold_fraction: f32,  // default 0.25
    pub speed_multiplier:   f32,  // default 2.0
    pub damage_multiplier:  f32,  // default 2.0
}
```

**`DeadlineArmed` resource** (per-node guard):

```rust
pub(crate) struct DeadlineArmed(pub bool);  // default false
```

**Systems** (all `FixedUpdate`, `run_if = protocol_active(Deadline) + in_state(NodeState::Playing)` unless noted):

1. **`deadline_watch_threshold`** — reads `NodeTimer`, computes `fraction = remaining / total`. On the tick where `fraction` first crosses below `threshold_fraction` AND `!DeadlineArmed.0`: for every `Query<Entity, With<Bolt>>`, call `commands.fire_effect(bolt, EffectType::SpeedBoost(SpeedBoostConfig { multiplier: config.speed_multiplier }), "protocol:Deadline".to_owned())` + same for `DamageBoost`. Set `DeadlineArmed.0 = true`.
2. **`deadline_stamp_on_spawn`** — reads `MessageReader<BoltSpawned>` (or `Added<Bolt>`). If `DeadlineArmed.0`: fire the same two effects on the newly-spawned bolt. Handles Fission/Afterimage-spawned bolts post-threshold.
3. **`deadline_cleanup_on_node_exit`** — `OnExit(NodeState::Playing)`, `run_if = protocol_active(Deadline)`. For every `With<Bolt>`: `commands.reverse_effect(bolt, ReversibleEffectType::SpeedBoost(...), "protocol:Deadline")` + same for `DamageBoost`. Set `DeadlineArmed.0 = false`.

**RON** (`assets/protocols/deadline.protocol.ron`):

```ron
(
    threshold_fraction: 0.25,
    speed_multiplier:   2.0,
    damage_multiplier:  2.0,
)
```

No `Root(...)`, `When(...)`, `Stamp(...)`, `Until(...)`. `ProtocolTuning::Deadline { effects: vec![] }` becomes `ProtocolTuning::Deadline { threshold_fraction, speed_multiplier, damage_multiplier }`.

**Tests** (`deadline/tests/`): `threshold_crossing_fires_effects`, `armed_guard_prevents_refire`, `late_spawned_bolt_gets_effects`, `node_exit_reverses_all_bolts`, `fresh_armed_state_per_node`, `stacks_multiplicatively_with_chip_boost`.

#### Kickstart (`protocols/kickstart/`)

**Module structure:** same layout as Deadline.

**`KickstartConfig` resource:**

```rust
pub(crate) struct KickstartConfig {
    pub speed_multiplier:  f32,  // default 2.0
    pub damage_multiplier: f32,  // default 2.0
    pub piercing_count:    u32,  // default 2
    pub countdown_secs:    f32,  // default 3.0
}
```

**`KickstartWindow` resource:**

```rust
pub(crate) struct KickstartWindow {
    pub remaining: Option<f32>,  // None = not yet armed
}
```

**Systems:**

1. **`kickstart_on_node_start`** — `OnEnter(NodeState::Playing)`, `run_if = protocol_active(Kickstart)`. For every `With<Bolt>`: fire SpeedBoost + DamageBoost with source `"protocol:Kickstart"`. Insert `KickstartPiercing(config.piercing_count)` component (or the project's standard source-tagged piercing — verify `bolt/components.rs`). Insert `KickstartWindow { remaining: None }` resource.
2. **`kickstart_on_first_bump`** — `FixedUpdate`, `.after(grade_bump)`, `run_if = protocol_active(Kickstart) + in_state(NodeState::Playing)`. Reads `BumpPerformed`. If `KickstartWindow.remaining.is_none()`, set `remaining = Some(config.countdown_secs)`.
3. **`kickstart_tick_window`** — `FixedUpdate`, `.after(kickstart_on_first_bump)`. If `remaining = Some(t)`, decrement by `FIXED_DELTA_SECS`. When `t <= 0.0`, for every `With<Bolt>`: reverse SpeedBoost + DamageBoost with source `"protocol:Kickstart"`. Remove `KickstartPiercing` component. Remove `KickstartWindow` resource.
4. **`kickstart_stamp_on_spawn`** — `FixedUpdate`, `run_if` same. Reads `BoltSpawned` (or `Added<Bolt>`). While `KickstartWindow` resource is present: fire the same effects on the new bolt.
5. **`kickstart_on_node_exit`** — `OnExit(NodeState::Playing)`, `run_if = protocol_active(Kickstart)`. Same reverse + component-remove cleanup as the window-complete branch. Always runs, even if countdown wasn't armed.

**RON:**

```ron
(
    speed_multiplier:  2.0,
    damage_multiplier: 2.0,
    piercing_count:    2,
    countdown_secs:    3.0,
)
```

**Tests** (`kickstart/tests/`): `node_start_applies_effects`, `first_bump_arms_timer`, `second_bump_does_not_reset`, `countdown_removes_effects`, `effects_persist_without_bump`, `late_spawned_bolt_receives_effects`, `node_exit_reverses_effects`, `fresh_window_each_node`, `stacks_with_chip_effects`.

#### Ricochet (`protocols/ricochet/`)

**`RicochetConfig` resource:**

```rust
pub(crate) struct RicochetConfig {
    pub damage_multiplier: f32,  // default 3.0
}
```

**Systems:**

1. **`ricochet_on_bolt_impact_wall`** — `FixedUpdate`, `.after(BoltSystems::WallCollision)`, `run_if = protocol_active(Ricochet) + in_state(NodeState::Playing)`. Reads `BoltImpactWall`. For each impacted bolt: `commands.fire_effect(bolt, EffectType::DamageBoost(DamageBoostConfig { multiplier: config.damage_multiplier, consume_on_use: Some(ConsumeOnUse) }), "protocol:Ricochet".to_owned())`. Source-tagged upsert guarantees multiple wall bounces don't stack (replaces the existing `"protocol:Ricochet"` entry instead of adding a second).

Consume-on-use aggregation is handled by `damage-amplification-standardization.md` Pattern B — consumed on first `DamageDealt<Cell>` emission.

**RON:**

```ron
(
    damage_multiplier: 3.0,
)
```

**Tests** (`ricochet/tests/`): `wall_impact_fires_boost`, `cell_impact_consumes_boost`, `second_cell_impact_no_boost`, `second_wall_impact_rearms`, `multiple_walls_no_stack`, `chip_stacks_multiplicatively`, `per_bolt_independent`, `breaker_impact_no_consume`.

### Docs update: `docs/architecture/effects.md`

Add §Routing vs Stamping section distinguishing `StampTarget` (root-level entity selector, resolved by `resolve_stamp_target` at dispatch time) from `RouteType` (within-tree routing primitive used by trigger bridges at trigger time). Include a table mapping each `StampTarget` variant to its closest `RouteType` analogue (or "no analogue"). Subsumes `effects-routing-vs-stamping-doc.md`.

Add §Dispatch Sites table naming the five sites (A–F excluding E which doesn't exist), their source-string prefixes, and the one-liner behavior of each.

## Tests to author

**Dispatcher unit tests** (`effect_v3/commands/stamp_root/tests.rs`):

1. `stamp_root_on_stamp_breaker_targets_every_breaker` — spawn 3 breakers, `stamp_root(&RootNode::Stamp(StampTarget::Breaker, tree), "test".into())`, assert all 3 have the tree in `BoundEffects`.
2. `stamp_root_on_stamp_every_bolt_stamps_now_and_later` — spawn 2 bolts, `stamp_root(&Stamp(EveryBolt, tree), "test".into())`, assert both bolts get the tree AND `SpawnStampRegistry.entries` contains `(Bolt, "test", tree)`. Spawn a 3rd bolt on the next tick; assert via `stamp_spawned_bolts` watcher that it also gets the tree.
3. `stamp_root_on_stamp_active_bolts_stamps_now_but_not_later` — spawn 2 bolts, `stamp_root(&Stamp(ActiveBolts, tree), "test".into())`, assert both bolts get the tree AND `SpawnStampRegistry.entries` does NOT contain the entry. Spawn a 3rd bolt; assert it does NOT receive the tree.
4. `stamp_root_on_spawn_registers_only` — `stamp_root(&RootNode::Spawn(EntityKind::Bolt, tree), "test".into())`, assert no existing bolts are stamped; `SpawnStampRegistry.entries` contains `(Bolt, "test", tree)`; newly-spawned bolts receive the tree.
5. `resolve_stamp_target_breaker_family` — all three breaker variants resolve identically.
6. `resolve_stamp_target_empty_world` — zero entities of the target kind returns empty `Vec<Entity>` (no panic).

**Per-site integration tests:**

7. `site_a_chip_stamp_every_bolt_reaches_bolts_not_breakers` — activate chip whose RON contains `Stamp(EveryBolt, Fire(SpeedBoost))`. Spawn 1 bolt. Assert bolt's `EffectStack<SpeedBoostConfig>` has `"chip:..."` entry; breaker does NOT.
8. `site_d_protocol_every_bolt_targets_bolts` — activate Anchor (post-migration stays tree-declared). Spawn bolt. Assert bolt (not breaker) carries Anchor's effects.
9. `site_f_bolt_def_source_is_bolt_def_id_not_empty` — bolt def with `Stamp(Breaker, Fire(DamageBoost))`. Spawn bolt via builder. Assert breaker's `EffectStack<DamageBoostConfig>` has entry with source `"bolt_def:{id}"`, NOT empty string.
10. `site_f_cell_def_with_multiple_breakers_does_not_panic` — spawn 2 breakers (primary + extra) + 1 cell via builder whose def has `Stamp(Breaker, Fire(...))`. Previously Site C's `.single()` panicked; now the resolver iterates `With<Breaker>` and both breakers get the effect.
11. `site_f_breaker_def_stamp_every_bolt_reaches_bolts` — breaker def with `Stamp(EveryBolt, Fire(SpeedBoost))`. Spawn breaker via builder with 2 bolts in the world. Assert both bolts get the tree (not the breaker). Spawn a 3rd bolt after the breaker. Assert it also gets the tree (via `SpawnStampRegistry` + `stamp_spawned_bolts`).
12. `site_f_no_double_dispatch` — regression test: spawn a bolt via the builder, tick one frame, assert each `EffectStack<*>` entry appears exactly once per `(source, config-variant)` pair. Proves Sites B/C deletion eliminated the double-dispatch.

**Code-driven protocol tests** — listed per-protocol above.

**Regression trap rewrite:** `protocol/systems/dispatch_protocol_selection/tests.rs:442-481` tests that pin the `_target`-ignored behavior are deleted. Replace with a test asserting `Stamp(EveryBolt, ...)` from a protocol installs the tree on bolts, not breakers.

## Code change summary

| File | Change |
|------|--------|
| `effect_v3/commands/stamp_root/mod.rs` | **NEW.** Module declarations + pub(crate) re-exports. |
| `effect_v3/commands/stamp_root/resolve.rs` | **NEW.** `resolve_stamp_target(target: StampTarget, world: &World) -> Vec<Entity>`. |
| `effect_v3/commands/stamp_root/register_every.rs` | **NEW.** `register_every_if_applicable(target, source, tree, &mut registry)`. |
| `effect_v3/commands/stamp_root/command.rs` | **NEW.** `StampRootCommand` + `impl Command`. |
| `effect_v3/commands/stamp_root/tests.rs` | **NEW.** Unit tests for resolver + `StampRootCommand` semantics. |
| `effect_v3/commands/mod.rs` | Add `pub(crate) mod stamp_root;` + re-export `StampRootCommand`. |
| `effect_v3/commands/ext/system.rs` | Add `stamp_root` + `stamp_roots` to `EffectCommandsExt` trait + `impl`. |
| `effect_v3/commands/ext/tests.rs` | Add trait coverage for the new methods. |
| `chips/systems/dispatch_chip_effects/system.rs` | Replace root-dispatch loop with `stamp_roots(&effects, format!("chip:{chip_name}"))`. Delete `resolve_target_entities`, `dispatch_tree`, `DispatchTargets`. |
| `bolt/systems/dispatch_bolt_effects/` | **DELETE ENTIRE DIRECTORY** (system + tests). Redundant with Site F. |
| `state/run/node/systems/dispatch_cell_effects/` | **DELETE ENTIRE DIRECTORY** (system + tests). Redundant with Site F. |
| `cells/components.rs` (or wherever `CellEffectsDispatched` lives) | **DELETE `CellEffectsDispatched` marker** — no longer referenced. |
| Plugin registration files | Remove `dispatch_bolt_effects` + `dispatch_cell_effects` from schedule. |
| `protocol/systems/dispatch_protocol_selection.rs` | Replace root-dispatch loop with `stamp_roots(&roots, format!("protocol:{kind:?}"))`. Delete `breakers` param. |
| `breaker/builder/core/terminal.rs` | `stamp_root_nodes` + `stamp_required_effects` forward each root to `commands.stamp_root(root, source)`. Drop the `entity` param. Sources: `"breaker_def:{id}"`, `"breaker_def:{id}:bolt_lost"`, `"breaker_def:{id}:salvo_hit"`. |
| `bolt/builder/core/terminal.rs` | Same one-line-forward. Source: `"bolt_def:{id}"`. |
| `cells/builder/core/terminal.rs` | Same. Source: `"cell_def:{alias}"`. |
| `walls/builder/core/terminal.rs` | Same. Source: `"wall_def:{id}"`. |
| `mutators/protocols/deadline/` | **NEW MODULE.** `DeadlineConfig`, `DeadlineArmed`, 3 systems, tests. |
| `mutators/protocols/kickstart/` | **NEW MODULE.** `KickstartConfig`, `KickstartWindow`, 5 systems, tests. |
| `mutators/protocols/ricochet/` | **NEW MODULE.** `RicochetConfig`, 1 system, tests. |
| `protocol/definition.rs` | `ProtocolTuning::Deadline`/`Kickstart`/`Ricochet` variants drop `effects` field; add tuning fields per protocol. |
| `assets/protocols/deadline.protocol.ron` | Strip effect tree; tuning-only. |
| `assets/protocols/kickstart.protocol.ron` | Strip effect tree; tuning-only. |
| `assets/protocols/ricochet.protocol.ron` | Strip effect tree; tuning-only. |
| `assets/chips/standard/ricochet_protocol.chip.ron` | **DELETE** (legacy). Verify no load path references it. |
| `protocol/systems/dispatch_protocol_selection/tests.rs:442-481` | **DELETE** regression-trap tests. Replace with `Stamp(EveryBolt, ...)` correctness tests. |
| `docs/architecture/effects.md` | Add §Routing vs Stamping + §Dispatch Sites. |
| `docs/design/protocols/deadline.md` | Rewrite §Effect Tree → §Systems + §Config Resource. |
| `docs/design/protocols/kickstart.md` | Same. |
| `docs/design/protocols/ricochet.md` | Same. |

## Dependencies

- **TODO #1 (unified death pipeline crate)** — the reverse-effect path reacts to `DamageDealt<Cell>` via `damage-amplification-standardization.md` Pattern B, which rides the death pipeline's emit/mutate/apply chain. Must land after #1.
- **TODO #2 (mutators domain refactor)** — new protocol modules `deadline/`, `kickstart/`, `ricochet/` land at `mutators/protocols/` (not `protocol/protocols/`). Must land after #2.

## Ordering

Lands after #1 and #2. Can interleave with #3, #4, #5, #6.

**Parallel waves within this TODO:**

- Wave 1: `stamp_root` infrastructure (`resolve_stamp_target`, `StampRootCommand`, trait methods, unit tests).
- Wave 2 (parallel): Site A (chip) + Site D (protocol) + Site F (all four builder terminals) + Sites B/C deletion. Each is independent.
- Wave 3 (parallel): Deadline/Kickstart/Ricochet code-driven modules. Each is independent of the others.
- Wave 4: doc updates, RON cleanups, delete legacy chip file, delete regression-trap tests.

## Subsumes

- `audit/remediations/stamp-dispatcher-unification.md`
- `audit/remediations/deadline-code-driven-effects.md`
- `audit/remediations/kickstart-code-driven-effects.md`
- `audit/remediations/ricochet-code-driven-effects.md`
- `audit/remediations/effects-routing-vs-stamping-doc.md`

## Scope boundary

In scope:
- `stamp_root` / `stamp_roots` facade + single resolver
- `SpawnStampRegistry` population via the new facade
- All 5 dispatch sites migrated (A/B/C/D/F)
- Deadline/Kickstart/Ricochet code-driven migrations
- Source string conventions enforced
- Regression-trap test deletion
- Routing-vs-stamping docs addition

Out of scope:
- Anchor's three remediations (`anchor-bump-force-multiplier.md`, `anchor-piercing.md`, `anchor-retire-during-primitive.md`) — separate TODO. This TODO fixes Anchor's DISPATCHER; Anchor's mechanic-specific fixes ride separately.
- `damage-amplification-standardization.md` Pattern B itself — separate work. Ricochet depends on it but implementing Pattern B is its own remediation.
- `effect-system-time-expires-wiring.md` — Deadline/Kickstart abandoned `Until(TimeExpires)` via code-driven migration; that remediation still stands for any other `Until(TimeExpires)` consumer.
- Site F Phase 2 (post-spawn deferred dispatch from builder) — parked until concrete caller appears.

## TODO entry

> **[BLOCKED by #1, #2]** Stamp dispatcher unification + code-driven Deadline/Kickstart/Ricochet — introduce `EffectCommandsExt::stamp_root(&RootNode, source)` backed by `StampRootCommand` (deferred `Command`) at `effect_v3/commands/stamp_root/`, with `resolve_stamp_target(StampTarget, &World) -> Vec<Entity>` as the SOLE resolver (Breaker family → `With<Breaker>`, Bolt family → `With<Bolt>`, Cell family → `With<Cell>`, Wall family → `With<Wall>`). `Every*` variants additionally register `(EntityKind, source, Tree)` with `SpawnStampRegistry` so the existing `stamp_spawned_*` watchers apply the tree to future spawns. All 5 dispatch sites migrated (A/B/C/D/F) with source conventions (`chip:`/`bolt_def:`/`cell_def:`/`protocol:`/builder-kind prefix); empty-string sources become impossible. Deadline, Kickstart, Ricochet strip effect trees from RON and become code-driven (new modules under `mutators/protocols/`, RON = tuning only, systems fire/reverse via `commands.fire_effect`/`reverse_effect`). Regression-trap tests at `dispatch_protocol_selection/tests.rs:442-481` deleted. Legacy `assets/chips/standard/ricochet_protocol.chip.ron` deleted. `docs/architecture/effects.md` gains §Routing vs Stamping + §Dispatch Sites. Subsumes `stamp-dispatcher-unification.md`, `deadline-code-driven-effects.md`, `kickstart-code-driven-effects.md`, `ricochet-code-driven-effects.md`, `effects-routing-vs-stamping-doc.md`. — [detail](detail/stamp-dispatcher-unification.md)
