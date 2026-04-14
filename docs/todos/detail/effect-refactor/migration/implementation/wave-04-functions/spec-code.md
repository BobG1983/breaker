## Implementation Spec: Effect — Wave 4 Non-System Functions

### Prerequisites

The following waves must be complete before Wave 4 begins:

- **Wave 1** (delete old) — old effect code removed
- **Wave 2** (scaffold) — all type definitions exist (Tree, ScopedTree, Terminal, ScopedTerminal, Trigger, TriggerContext, Condition, EffectType, ReversibleEffectType, EntityKind, RouteType, ParticipantTarget, BumpTarget, ImpactTarget, DeathTarget, BoltLostTarget, BoundEffects, BoundEntry, StagedEffects, EffectSourceChip, NodeState, ShieldWall)
- **Wave 3** (RON assets) — RON data files in place

### Domain
`src/effect_v3/`

### Failing Tests
Test file locations and counts will be established by the test spec. Tests will be distributed across the following files, matching the folder structure in `docs/todos/detail/effect-refactor/migration/folder-structure.md`:

- `src/effect_v3/stacking/effect_stack.rs` — EffectStack<T> push/remove/aggregate tests
- `src/effect_v3/walking/walk_effects.rs` — walk_effects outer loop tests (staged-before-bound ordering, trigger matching)
- `src/effect_v3/walking/fire.rs` — Fire node evaluation tests
- `src/effect_v3/walking/when.rs` — When node evaluation tests (match, skip, arming nested gates)
- `src/effect_v3/walking/once.rs` — Once node evaluation tests (match + removal)
- `src/effect_v3/walking/during.rs` — During node evaluation tests (condition transitions, apply/reverse)
- `src/effect_v3/walking/until.rs` — Until node evaluation tests (install + reversal on trigger)
- `src/effect_v3/walking/sequence.rs` — Sequence node evaluation tests
- `src/effect_v3/walking/on.rs` — On node evaluation tests (participant resolution, mismatch skip)
- `src/effect_v3/walking/route.rs` — Route node evaluation tests
- `src/effect_v3/dispatch/fire_dispatch.rs` — fire_dispatch tests (all 30 EffectType variants)
- `src/effect_v3/dispatch/reverse_dispatch.rs` — reverse_dispatch tests (all 16 ReversibleEffectType variants)
- `src/effect_v3/commands/stamp.rs` — stamp_effect tests
- `src/effect_v3/commands/fire.rs` — fire_effect command tests
- `src/effect_v3/commands/reverse.rs` — reverse_effect command tests
- `src/effect_v3/commands/route.rs` — route_effect command tests
- `src/effect_v3/commands/stage.rs` — stage_effect command tests
- `src/effect_v3/commands/remove.rs` — remove_effect command tests
- `src/effect_v3/effects/speed_boost/config.rs` — SpeedBoostConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/size_boost/config.rs` — SizeBoostConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/damage_boost/config.rs` — DamageBoostConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/bump_force/config.rs` — BumpForceConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/quick_stop/config.rs` — QuickStopConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/vulnerable/config.rs` — VulnerableConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/piercing/config.rs` — PiercingConfig fire/reverse/aggregate tests
- `src/effect_v3/effects/ramping_damage/config.rs` — RampingDamageConfig fire/reverse/aggregate tests
- `src/effect_v3/conditions/node_active.rs` — is_node_active evaluator tests
- `src/effect_v3/conditions/shield_active.rs` — is_shield_active evaluator tests
- `src/effect_v3/conditions/combo_active.rs` — is_combo_active evaluator tests

---

### What to Implement

#### 1. EffectStack<T> (stacking/effect_stack.rs)

Generic component for passive effect stacking. Monomorphized per config type.

```rust
#[derive(Component)]
pub struct EffectStack<T: PassiveEffect> {
    entries: Vec<(String, T)>,
}

// Manual Default impl — avoids T: Default bound from derive
impl<T: PassiveEffect> Default for EffectStack<T> {
    fn default() -> Self { Self { entries: Vec::new() } }
}

impl<T: PassiveEffect> EffectStack<T> {
    pub fn push(&mut self, source: String, config: T);
    pub fn remove(&mut self, source: &str, config: &T);
    pub fn aggregate(&self) -> f32;
}
```

**Generic Component derive note**: Bevy's `#[derive(Component)]` on a generic struct requires the type parameter to satisfy `T: 'static` at minimum (Bevy's Component trait bound). Since `PassiveEffect` already requires `Clone + PartialEq + Eq + Sized`, the writer-code must ensure the trait bound includes `'static` — i.e., `pub trait PassiveEffect: Fireable + Reversible + Sized + Clone + PartialEq + Eq + 'static`. Alternatively, add `T: 'static` directly on the struct. The `Default` derive also requires `T: Default` OR the writer-code must implement `Default` manually (preferred: implement manually with `entries: Vec::new()` so there is no `T: Default` bound). If Bevy's derive macro emits errors for the generic, consider `#[component(storage = "SparseSet")]` which may work better for generic types, or implement `Component` manually.

- `push`: Append `(source, config)` to `entries`.
- `remove`: Find and remove the **first** entry where `entry.0 == source && entry.1 == *config`. If no match, do nothing.
- `aggregate`: Delegate to `T::aggregate(&self.entries)`. Returns identity value when empty (1.0 for multiplicative, 0 for additive).

**File**: `src/effect_v3/stacking/effect_stack.rs`
**Module wiring**: `src/effect_v3/stacking/mod.rs` re-exports `EffectStack`.

#### 2. PassiveEffect trait (traits/passive_effect.rs)

```rust
pub trait PassiveEffect: Fireable + Reversible + Sized + Clone + PartialEq + Eq + 'static {
    fn aggregate(entries: &[(String, Self)]) -> f32;
}
```

The `'static` bound is required because `EffectStack<T: PassiveEffect>` derives `Component`, and Bevy's `Component` trait requires `'static`.

**File**: `src/effect_v3/traits/passive_effect.rs`
**Module wiring**: `src/effect_v3/traits/mod.rs` re-exports `PassiveEffect`.

#### 3. Fireable trait (traits/fireable.rs)

```rust
pub trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);
    fn register(app: &mut App) {}
}
```

**File**: `src/effect_v3/traits/fireable.rs`
**Module wiring**: `src/effect_v3/traits/mod.rs` re-exports `Fireable`.

#### 4. Reversible trait (traits/reversible.rs)

```rust
pub trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);
}
```

**File**: `src/effect_v3/traits/reversible.rs`
**Module wiring**: `src/effect_v3/traits/mod.rs` re-exports `Reversible`.

#### 5. walk_effects function (walking/walk_effects.rs)

The core tree-walking helper. Not a system. Called by bridge systems.

```rust
pub fn walk_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    bound: &BoundEffects,
    staged: &StagedEffects,
    commands: &mut Commands,
);
```

**Algorithm:**

**Step 1 — Walk StagedEffects:**
Iterate every `(source, tree)` entry in `staged.0`. For each:
1. Check if the tree's outermost node matches the trigger (exact `PartialEq` match on the Trigger enum).
2. If match, evaluate the tree via the per-node evaluator (see below).
3. Queue `remove_effect(entity, RouteType::Staged, source.clone(), tree.clone())` — the entry is consumed.
4. **Exception**: During nodes in StagedEffects are NOT consumed on first match — they have special lifecycle handling. (Note: in practice During nodes should not appear directly in StagedEffects per tree type constraints, but guard against it.)

**Step 2 — Walk BoundEffects:**
Iterate every `BoundEntry { source, tree, condition_active }` in `bound.0`. For each:
1. If `condition_active` is `Some(_)`, **skip** — this is a During entry handled by `evaluate_conditions`, not trigger walking.
2. Check if the tree's outermost node matches the trigger.
3. If match, evaluate the tree via the per-node evaluator.
4. Do NOT remove the entry (bound entries persist). Exception: Once nodes queue their own removal.

**Trigger matching:**
Exact `==` on the `Trigger` enum. `Trigger::Bumped == Trigger::Bumped`. `Trigger::Impacted(EntityKind::Cell) != Trigger::Impacted(EntityKind::Bolt)`. No wildcards.

**How to extract the outermost trigger from a Tree:**
- `Tree::When(trigger, _)` -> `trigger`
- `Tree::Once(trigger, _)` -> `trigger`
- `Tree::Until(trigger, _)` -> `trigger`
- `Tree::Fire(_)`, `Tree::Sequence(_)`, `Tree::On(_, _)`, `Tree::During(_, _)` -> no trigger to match (these are immediate nodes, not trigger-gated). They should not be the outermost node of a StagedEffects entry or a trigger-gated BoundEffects entry. If encountered, skip.

**File**: `src/effect_v3/walking/walk_effects.rs`
**Module wiring**: `src/effect_v3/walking/mod.rs` re-exports `walk_effects`.

#### 6. Per-Node Evaluators (walking/*.rs)

Each evaluator is a function called by `walk_effects` (or recursively by other evaluators) when a node matches.

##### 6a. evaluate_fire (walking/fire.rs)

```rust
fn evaluate_fire(
    entity: Entity,
    effect: &EffectType,
    source: &str,
    commands: &mut Commands,
);
```

Calls `commands.fire_effect(entity, effect.clone(), source.to_string())`. Fire is a leaf node — no recursion.

##### 6b. evaluate_when (walking/when.rs)

```rust
fn evaluate_when(
    entity: Entity,
    trigger: &Trigger,
    inner_tree: &Tree,
    source: &str,
    current_trigger: &Trigger,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

1. If `trigger != current_trigger`, return (no match).
2. If match, check if inner_tree is a trigger gate:
   - `Tree::When(_, _)` | `Tree::Once(_, _)` | `Tree::Until(_, _)` -> **arm**: call `commands.stage_effect(entity, source.to_string(), inner_tree.clone())`. Do NOT evaluate recursively.
   - Otherwise (`Tree::Fire(_)`, `Tree::Sequence(_)`, `Tree::On(_, _)`, `Tree::During(_, _)`) -> evaluate recursively via the appropriate per-node evaluator.
3. The When entry stays in storage (caller does not remove it).

##### 6c. evaluate_once (walking/once.rs)

```rust
fn evaluate_once(
    entity: Entity,
    trigger: &Trigger,
    inner_tree: &Tree,
    source: &str,
    full_tree: &Tree,  // the complete Once node, for removal
    current_trigger: &Trigger,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

1. If `trigger != current_trigger`, return.
2. If match, evaluate the inner_tree using the same arming rules as When (step 3 in evaluate_when).
3. Queue `commands.remove_effect(entity, RouteType::Bound, source.to_string(), full_tree.clone())` — Once is consumed after matching.

##### 6d. evaluate_during (walking/during.rs — condition transition handling)

During nodes are NOT evaluated by the walk_effects trigger loop. Instead, the `evaluate_conditions` system (a system, out of scope for this wave) polls conditions each frame and calls the during evaluator on transitions.

However, the **forward application** and **reverse application** helper functions must be implemented here:

```rust
pub fn apply_scoped_tree(
    entity: Entity,
    scoped: &ScopedTree,
    source: &str,
    context: &TriggerContext,
    commands: &mut Commands,
);

pub fn reverse_scoped_tree(
    entity: Entity,
    scoped: &ScopedTree,
    source: &str,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

**apply_scoped_tree** (condition becomes true, or Until installation):
- `ScopedTree::Fire(reversible_effect)` -> `commands.fire_effect(entity, reversible_to_effect_type(reversible_effect), source)`
- `ScopedTree::Sequence(effects)` -> for each effect left to right, `commands.fire_effect(...)` 
- `ScopedTree::When(trigger, tree)` -> `commands.stage_effect(entity, source, Tree::When(trigger, tree))` — install the When as a listener
- `ScopedTree::On(participant, scoped_terminal)` -> resolve participant from context, apply the scoped terminal on the resolved entity

**reverse_scoped_tree** (condition becomes false, or Until trigger fires):
- `ScopedTree::Fire(reversible_effect)` -> `commands.reverse_effect(entity, reversible_effect, source)`
- `ScopedTree::Sequence(effects)` -> for each effect in **reverse order** (right to left), `commands.reverse_effect(...)`
- `ScopedTree::When(trigger, tree)` -> `commands.remove_effect(entity, RouteType::Staged, source, Tree::When(trigger, tree))` — remove the listener. Do NOT reverse individual effects that already fired from past matches.
- `ScopedTree::On(participant, scoped_terminal)` -> resolve participant from context, reverse the scoped terminal on the resolved entity. **Edge case**: if the participant entity is gone (context resolution returns `None`), do nothing — same behavior as `Tree::On` during application when the participant cannot be resolved. This can happen when a condition deactivates after the participant entity has been despawned (e.g., a Cell that was destroyed).

**ScopedTerminal evaluation** (used by On within ScopedTree):
- `ScopedTerminal::Fire(reversible_effect)` -> fire or reverse depending on direction
- `ScopedTerminal::Route(route_type, tree)` -> `commands.route_effect(target, source, tree, route_type)` for apply; for reverse, `commands.remove_effect(target, route_type, source, tree)`

##### 6e. evaluate_until (walking/until.rs)

Until has two phases:

**Installation (called when Until is first encountered during walking — i.e., from a When/Once gate that matched):**

```rust
pub fn install_until(
    entity: Entity,
    trigger: &Trigger,
    scoped: &ScopedTree,
    source: &str,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

1. Call `apply_scoped_tree(entity, scoped, source, context, commands)` — apply effects immediately.
2. If trigger is `Trigger::TimeExpires(duration)`, register a timer: insert/push `(duration, duration)` onto the entity's `EffectTimers` component.

**Trigger match (called when the Until's trigger fires during walking):**

```rust
pub fn reverse_until(
    entity: Entity,
    trigger: &Trigger,
    scoped: &ScopedTree,
    source: &str,
    full_tree: &Tree,
    current_trigger: &Trigger,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

1. If `trigger != current_trigger`, return.
2. Call `reverse_scoped_tree(entity, scoped, source, context, commands)`.
3. Queue `commands.remove_effect(entity, RouteType::Bound, source.to_string(), full_tree.clone())` — Until is one-shot, does not re-arm.

##### 6f. evaluate_sequence (walking/sequence.rs)

```rust
fn evaluate_sequence(
    entity: Entity,
    terminals: &[Terminal],
    source: &str,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

Iterate terminals left to right. For each:
- `Terminal::Fire(effect)` -> `commands.fire_effect(entity, effect.clone(), source.to_string())`
- `Terminal::Route(route_type, tree)` -> `commands.route_effect(entity, source.to_string(), (*tree).clone(), route_type.clone())`

##### 6g. evaluate_on (walking/on.rs)

**Context separation note**: This function handles `Tree::On(ParticipantTarget, Terminal)` only. `ScopedTree::On(ParticipantTarget, ScopedTerminal)` is handled separately within `apply_scoped_tree`/`reverse_scoped_tree` — do not reuse `evaluate_on` there. The scoped variant deals with `ScopedTerminal` (which contains `ReversibleEffectType` and needs reversal support), while this function deals with `Terminal` (fire-and-forget).

```rust
fn evaluate_on(
    entity: Entity,
    participant: &ParticipantTarget,
    terminal: &Terminal,
    source: &str,
    context: &TriggerContext,
    commands: &mut Commands,
);
```

1. Resolve `ParticipantTarget` to an `Option<Entity>` from the `TriggerContext`:
   - `ParticipantTarget::Bump(BumpTarget::Bolt)` + `TriggerContext::Bump { bolt, .. }` -> `bolt` (Option)
   - `ParticipantTarget::Bump(BumpTarget::Breaker)` + `TriggerContext::Bump { breaker, .. }` -> `Some(breaker)`
   - `ParticipantTarget::Impact(ImpactTarget::Impactor)` + `TriggerContext::Impact { impactor, .. }` -> `Some(impactor)`
   - `ParticipantTarget::Impact(ImpactTarget::Impactee)` + `TriggerContext::Impact { impactee, .. }` -> `Some(impactee)`
   - `ParticipantTarget::Death(DeathTarget::Victim)` + `TriggerContext::Death { victim, .. }` -> `Some(victim)`
   - `ParticipantTarget::Death(DeathTarget::Killer)` + `TriggerContext::Death { killer, .. }` -> `killer` (Option)
   - `ParticipantTarget::BoltLost(BoltLostTarget::Bolt)` + `TriggerContext::BoltLost { bolt, .. }` -> `Some(bolt)`
   - `ParticipantTarget::BoltLost(BoltLostTarget::Breaker)` + `TriggerContext::BoltLost { breaker, .. }` -> `Some(breaker)`
   - Any mismatched context variant -> `None` (skip)
   - `TriggerContext::None` -> `None` (skip)

2. If resolution returns `None`, do nothing (entity doesn't exist or context mismatch).
3. If resolution returns `Some(target_entity)`, evaluate the terminal on `target_entity`:
   - `Terminal::Fire(effect)` -> `commands.fire_effect(target_entity, effect.clone(), source.to_string())`
   - `Terminal::Route(route_type, tree)` -> `commands.route_effect(target_entity, source.to_string(), (*tree).clone(), route_type.clone())`

##### 6h. evaluate_route (walking/route.rs)

```rust
fn evaluate_route(
    entity: Entity,
    route_type: &RouteType,
    tree: &Tree,
    source: &str,
    commands: &mut Commands,
);
```

Calls `commands.route_effect(entity, source.to_string(), tree.clone(), route_type.clone())`. Route installs a tree for later evaluation — does NOT evaluate the tree contents.

#### 7. Fire Dispatch (dispatch/fire_dispatch.rs)

```rust
pub fn fire_dispatch(
    effect: &EffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
);
```

Match on all 30 `EffectType` variants, calling `config.fire(entity, source, world)` for each:

```rust
match effect {
    EffectType::SpeedBoost(config) => config.fire(entity, source, world),
    EffectType::SizeBoost(config) => config.fire(entity, source, world),
    EffectType::DamageBoost(config) => config.fire(entity, source, world),
    EffectType::BumpForce(config) => config.fire(entity, source, world),
    EffectType::QuickStop(config) => config.fire(entity, source, world),
    EffectType::FlashStep(config) => config.fire(entity, source, world),
    EffectType::Piercing(config) => config.fire(entity, source, world),
    EffectType::Vulnerable(config) => config.fire(entity, source, world),
    EffectType::RampingDamage(config) => config.fire(entity, source, world),
    EffectType::Attraction(config) => config.fire(entity, source, world),
    EffectType::Anchor(config) => config.fire(entity, source, world),
    EffectType::Pulse(config) => config.fire(entity, source, world),
    EffectType::Shield(config) => config.fire(entity, source, world),
    EffectType::SecondWind(config) => config.fire(entity, source, world),
    EffectType::Shockwave(config) => config.fire(entity, source, world),
    EffectType::Explode(config) => config.fire(entity, source, world),
    EffectType::ChainLightning(config) => config.fire(entity, source, world),
    EffectType::PiercingBeam(config) => config.fire(entity, source, world),
    EffectType::SpawnBolts(config) => config.fire(entity, source, world),
    EffectType::SpawnPhantom(config) => config.fire(entity, source, world),
    EffectType::ChainBolt(config) => config.fire(entity, source, world),
    EffectType::MirrorProtocol(config) => config.fire(entity, source, world),
    EffectType::TetherBeam(config) => config.fire(entity, source, world),
    EffectType::GravityWell(config) => config.fire(entity, source, world),
    EffectType::LoseLife(config) => config.fire(entity, source, world),
    EffectType::TimePenalty(config) => config.fire(entity, source, world),
    EffectType::Die(config) => config.fire(entity, source, world),
    EffectType::CircuitBreaker(config) => config.fire(entity, source, world),
    EffectType::EntropyEngine(config) => config.fire(entity, source, world),
    EffectType::RandomEffect(config) => config.fire(entity, source, world),
}
```

Every arm is identical in shape — the dispatch is purely mechanical.

**File**: `src/effect_v3/dispatch/fire_dispatch.rs`
**Module wiring**: `src/effect_v3/dispatch/mod.rs` re-exports `fire_dispatch`.

#### 8. Reverse Dispatch (dispatch/reverse_dispatch.rs)

```rust
pub fn reverse_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
);
```

Match on all 16 `ReversibleEffectType` variants, calling `config.reverse(entity, source, world)` for each:

```rust
match effect {
    ReversibleEffectType::SpeedBoost(config) => config.reverse(entity, source, world),
    ReversibleEffectType::SizeBoost(config) => config.reverse(entity, source, world),
    ReversibleEffectType::DamageBoost(config) => config.reverse(entity, source, world),
    ReversibleEffectType::BumpForce(config) => config.reverse(entity, source, world),
    ReversibleEffectType::QuickStop(config) => config.reverse(entity, source, world),
    ReversibleEffectType::FlashStep(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Piercing(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Vulnerable(config) => config.reverse(entity, source, world),
    ReversibleEffectType::RampingDamage(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Attraction(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Anchor(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Pulse(config) => config.reverse(entity, source, world),
    ReversibleEffectType::Shield(config) => config.reverse(entity, source, world),
    ReversibleEffectType::SecondWind(config) => config.reverse(entity, source, world),
    ReversibleEffectType::CircuitBreaker(config) => config.reverse(entity, source, world),
    ReversibleEffectType::EntropyEngine(config) => config.reverse(entity, source, world),
}
```

**File**: `src/effect_v3/dispatch/reverse_dispatch.rs`
**Module wiring**: `src/effect_v3/dispatch/mod.rs` re-exports `reverse_dispatch`.

#### 9. Command Extensions (commands/*.rs)

All commands implement Bevy's `Command` trait (i.e., `fn apply(self, world: &mut World)`). The `EffectCommandsExt` trait extends `Commands` with ergonomic methods.

##### 9a. EffectCommandsExt trait (commands/ext.rs)

```rust
pub trait EffectCommandsExt {
    fn stamp_effect(&mut self, entity: Entity, source: String, tree: Tree);
    fn fire_effect(&mut self, entity: Entity, effect: EffectType, source: String);
    fn reverse_effect(&mut self, entity: Entity, effect: ReversibleEffectType, source: String);
    fn route_effect(&mut self, entity: Entity, source: String, tree: Tree, route_type: RouteType);
    fn stage_effect(&mut self, entity: Entity, source: String, tree: Tree);
    fn remove_effect(&mut self, entity: Entity, route_type: RouteType, source: String, tree: Tree);
}

impl EffectCommandsExt for Commands<'_, '_> { ... }
```

Each method queues the corresponding command struct (described below).

**File**: `src/effect_v3/commands/ext.rs`

##### 9b. StampEffectCommand (commands/stamp.rs)

```rust
struct StampEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
}
```

`apply`:
1. If entity does not exist in world, return.
2. Ensure entity has both `BoundEffects` and `StagedEffects` (insert with Default if absent — always inserted as a pair).
3. Determine `condition_active`: if tree root is `Tree::During(_, _)`, set to `Some(false)`. Otherwise `None`.
4. Append `BoundEntry { source, tree, condition_active }` to `BoundEffects`.

stamp_effect always appends — no deduplication check.

**File**: `src/effect_v3/commands/stamp.rs`

##### 9c. FireEffectCommand (commands/fire.rs)

```rust
struct FireEffectCommand {
    entity: Entity,
    effect: EffectType,
    source: String,
}
```

`apply`:
1. If entity does not exist in world, return.
2. Call `fire_dispatch(&self.effect, self.entity, &self.source, world)`.

**File**: `src/effect_v3/commands/fire.rs`

##### 9d. ReverseEffectCommand (commands/reverse.rs)

```rust
struct ReverseEffectCommand {
    entity: Entity,
    effect: ReversibleEffectType,
    source: String,
}
```

`apply`:
1. If entity does not exist in world, return.
2. Call `reverse_dispatch(&self.effect, self.entity, &self.source, world)`.

If no matching entry exists (already reversed, never fired), do nothing (the dispatch function handles this).

**File**: `src/effect_v3/commands/reverse.rs`

##### 9e. RouteEffectCommand (commands/route.rs)

```rust
struct RouteEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
    route_type: RouteType,
}
```

`apply`:
1. If entity does not exist in world, return.
2. Ensure entity has both `BoundEffects` and `StagedEffects` (insert with Default if absent — always paired).
3. Match `route_type`:
   - `RouteType::Bound` -> determine `condition_active` (Some(false) if During, else None). Append `BoundEntry { source, tree, condition_active }` to `BoundEffects`.
   - `RouteType::Staged` -> append `(source, tree)` to `StagedEffects`.

**File**: `src/effect_v3/commands/route.rs`

##### 9f. StageEffectCommand (commands/stage.rs)

```rust
struct StageEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
}
```

`apply`: Sugar for `RouteEffectCommand { entity, source, tree, route_type: RouteType::Staged }.apply(world)`.

1. If entity does not exist in world, return.
2. Ensure entity has both `BoundEffects` and `StagedEffects`.
3. Append `(source, tree)` to `StagedEffects`.

**File**: `src/effect_v3/commands/stage.rs`

##### 9g. RemoveEffectCommand (commands/remove.rs)

```rust
struct RemoveEffectCommand {
    entity: Entity,
    route_type: RouteType,
    source: String,
    tree: Tree,
}
```

`apply`:
1. If entity does not exist in world, return.
2. Match `route_type`:
   - `RouteType::Bound` -> find the **first** `BoundEntry` in `BoundEffects` where `entry.source == self.source && entry.tree == self.tree`. Remove it. If none found, do nothing.
   - `RouteType::Staged` -> find the **first** `(source, tree)` in `StagedEffects` where `source == self.source && tree == self.tree`. Remove it. If none found, do nothing.

**File**: `src/effect_v3/commands/remove.rs`

#### 10. Passive Effect Implementations (8 configs)

Each passive effect config struct implements `Fireable`, `Reversible`, and `PassiveEffect`. All follow the same pattern.

##### 10a. SpeedBoostConfig (effects/speed_boost/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeedBoostConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- `Fireable::fire`: Get or insert `EffectStack<SpeedBoostConfig>` on entity. Call `stack.push(source.to_string(), self.clone())`.
- `Reversible::reverse`: Get `EffectStack<SpeedBoostConfig>` on entity. Call `stack.remove(source, self)`. If stack is absent, do nothing.
- `PassiveEffect::aggregate`: Product of all `config.multiplier.0` values. Identity (empty): `1.0`.

##### 10b. SizeBoostConfig (effects/size_boost/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeBoostConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- Aggregate: multiplicative (product). Identity: `1.0`.

##### 10c. DamageBoostConfig (effects/damage_boost/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageBoostConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- Aggregate: multiplicative (product). Identity: `1.0`.

##### 10d. BumpForceConfig (effects/bump_force/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BumpForceConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- Aggregate: multiplicative (product). Identity: `1.0`.

##### 10e. QuickStopConfig (effects/quick_stop/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickStopConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- Aggregate: multiplicative (product). Identity: `1.0`.

##### 10f. VulnerableConfig (effects/vulnerable/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VulnerableConfig {
    pub multiplier: OrderedFloat<f32>,
}
```

- Aggregate: multiplicative (product). Identity: `1.0`.

##### 10g. PiercingConfig (effects/piercing/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PiercingConfig {
    pub charges: u32,
}
```

- `Fireable::fire`: Get or insert `EffectStack<PiercingConfig>` on entity. Call `stack.push(source.to_string(), self.clone())`.
- `Reversible::reverse`: Get `EffectStack<PiercingConfig>` on entity. Call `stack.remove(source, self)`.
- `PassiveEffect::aggregate`: Sum of all `config.charges as f32` values. Identity (empty): `0.0`. (Note: PiercingConfig derives Eq directly since charges is u32, no OrderedFloat needed.)

Wait — PiercingConfig has `charges: u32`, not `OrderedFloat<f32>`. Since u32 derives Eq natively, this works for EffectStack matching. However, for the PartialEq/Eq derive requirement of PassiveEffect, this is satisfied by u32's native Eq.

##### 10h. RampingDamageConfig (effects/ramping_damage/config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RampingDamageConfig {
    pub increment: OrderedFloat<f32>,
}
```

- Aggregate: additive (sum of all `config.increment.0`). Identity (empty): `0.0`.

**All 8 passive effects use the exact same fire/reverse pattern:**
- `fire`: get_or_insert_default `EffectStack<Self>`, push `(source, self.clone())`
- `reverse`: get `EffectStack<Self>`, remove `(source, self)`, do nothing if absent

Only `aggregate` differs between them.

#### 11. Condition Evaluators (conditions/*.rs)

##### 11a. is_node_active (conditions/node_active.rs)

```rust
pub fn is_node_active(world: &World) -> bool;
```

Reads `State<NodeState>` resource from world via `world.get_resource::<State<NodeState>>()`. Returns `true` when state is `NodeState::Playing`, `false` otherwise. Takes `&World` (shared reference) — this function only reads a resource, no query needed. Note: `is_shield_active` and `is_combo_active` take `&mut World` because they use `World::query` which requires mutable access in Bevy 0.18.

**File**: `src/effect_v3/conditions/node_active.rs`

##### 11b. is_shield_active (conditions/shield_active.rs)

```rust
pub fn is_shield_active(world: &mut World) -> bool;
```

Queries world for any entity with the `ShieldWall` component. Returns `true` if at least one exists, `false` otherwise. Logically read-only.

**Bevy 0.18 API note**: `World::query` and `World::query_filtered` require `&mut World` in Bevy 0.18 — there is no way to run an entity query on `&World`. The design doc at `docs/todos/detail/effect-refactor/evaluating-conditions/is-shield-active.md` shows `&World` but the implementation must use `&mut World` to compile. Use `world.query::<&ShieldWall>().iter(world).next().is_some()` or equivalent — the `QueryState` approach requires splitting the borrow: `let mut query = world.query::<&ShieldWall>(); query.iter(world).next().is_some()`.

**File**: `src/effect_v3/conditions/shield_active.rs`

##### 11c. is_combo_active (conditions/combo_active.rs)

```rust
pub fn is_combo_active(world: &mut World, threshold: u32) -> bool;
```

Reads the `ComboStreak` resource from world. The resource is defined as:

```rust
#[derive(Resource, Default)]
pub struct ComboStreak {
    pub count: u32,
}
```

Returns `true` when `combo_streak.count >= threshold`, `false` otherwise. If the `ComboStreak` resource does not exist, returns `false`. Logically read-only, takes `&mut World` for consistency with other condition evaluators.

**ComboStreak location**: The resource definition goes in `src/effect_v3/conditions/combo_active.rs` (or a shared `src/effect_v3/conditions/resources.rs` re-exported from `src/effect_v3/conditions/mod.rs`). It is updated by a `track_combo_streak` system (out of scope for this wave — Wave 5+).

**File**: `src/effect_v3/conditions/combo_active.rs`

#### 12. Conversion helpers

A helper function to convert `ReversibleEffectType` to `EffectType` is needed for `apply_scoped_tree` when calling `fire_effect` with a reversible effect type:

```rust
pub fn reversible_to_effect_type(reversible: &ReversibleEffectType) -> EffectType;
```

This is a mechanical mapping — each ReversibleEffectType variant maps to the corresponding EffectType variant with the same inner config. Place in `src/effect_v3/dispatch/mod.rs` or a small helper file.

---

### Patterns to Follow

- **Source docs only**: All patterns come from the effect-refactor documentation under `docs/todos/detail/effect-refactor/`. Do NOT reference existing `src/` code for patterns.
- **Folder structure**: Follow `docs/todos/detail/effect-refactor/migration/folder-structure.md` exactly.
- **Wiring an effect**: Follow the checklist in `docs/todos/detail/effect-refactor/creating-effects/wiring-an-effect.md`.
- **All f32 fields use OrderedFloat<f32>** to enable Eq derives for EffectStack matching.
- **Command pattern**: Each command struct implements `Command` with `fn apply(self, world: &mut World)`. All world mutations happen in `apply`.
- **Entity safety**: Always check entity existence before world access. Never panic on missing components — insert defaults or do nothing.
- **Deferred mutations only**: The walking algorithm and evaluators never mutate BoundEffects/StagedEffects directly. All mutations go through command extensions.

---

### RON Data
Not applicable for this wave. No RON file changes needed.

---

### Schedule
Not applicable for this wave. All items are non-system functions (helper functions, trait implementations, command structs). No systems are registered in any schedule.

The only schedule-adjacent note: `evaluate_conditions` (which calls the condition evaluators and during apply/reverse helpers) is a system that runs in FixedUpdate, but implementing that system is out of scope for this wave (Wave 5: Systems).

---

### Constraints

#### Off-Limits — Do NOT Modify
- Do NOT modify test files — tests are written by writer-tests and are immutable during the GREEN phase.
- Do NOT create or modify systems (Bevy system functions registered to schedules). This wave is non-system functions only.
- Do NOT implement non-passive effect fire/reverse (Shockwave, Explode, Shield, etc.). Only the 8 passive effect configs get trait implementations in this wave.
- Do NOT implement RON deserialization.
- Do NOT touch any files outside `src/effect_v3/`.
- Do NOT modify `src/effect_v3/plugin.rs` (system registration is Wave 5+).
- Do NOT modify `src/effect_v3/mod.rs` except to add new module declarations needed by the new folders (stacking, walking, dispatch, commands, conditions, traits, effects subfolders).

#### Out of Scope
- Bridge systems (triggers/) — Wave 5+
- Tick systems (effects/*/systems.rs) — Wave 5+
- evaluate_conditions system — Wave 5+
- Non-passive effect Fireable/Reversible implementations (Shockwave, Shield, Explode, etc.) — separate wave
- SpawnStampRegistry resource — separate wave
- EffectV3Plugin::build wiring — Wave 5+

#### Module Wiring Required

New `mod` declarations needed in `src/effect_v3/mod.rs` (or appropriate parent mod.rs files):

```
src/effect_v3/
  mod.rs          — add: pub mod traits; pub mod stacking; pub mod walking; pub mod dispatch; pub mod commands; pub mod conditions;
  traits/
    mod.rs        — pub mod fireable; pub mod reversible; pub mod passive_effect; + re-exports
  stacking/
    mod.rs        — pub mod effect_stack; + re-exports
  walking/
    mod.rs        — pub mod walk_effects; pub mod fire; pub mod when; pub mod once; pub mod during; pub mod until; pub mod sequence; pub mod on; pub mod route; + re-exports
  dispatch/
    mod.rs        — pub mod fire_dispatch; pub mod reverse_dispatch; + re-exports
  commands/
    mod.rs        — pub mod ext; pub mod fire; pub mod reverse; pub mod route; pub mod stamp; pub mod stage; pub mod remove; + re-exports
  conditions/
    mod.rs        — pub mod node_active; pub mod shield_active; pub mod combo_active; + re-exports
  effects/
    speed_boost/
      mod.rs      — pub mod config; + re-exports
    size_boost/
      mod.rs      — pub mod config; + re-exports
    damage_boost/
      mod.rs      — pub mod config; + re-exports
    bump_force/
      mod.rs      — pub mod config; + re-exports
    quick_stop/
      mod.rs      — pub mod config; + re-exports
    vulnerable/
      mod.rs      — pub mod config; + re-exports
    piercing/
      mod.rs      — pub mod config; + re-exports
    ramping_damage/
      mod.rs      — pub mod config; + re-exports
```

All `mod.rs` files follow the routing-only rule: `pub mod` declarations and `pub use` re-exports only. No logic, no types.

---

### Type Dependencies from Earlier Waves

This wave depends on types defined in earlier waves (Wave 1-3). The following types must already exist before this wave begins:

**From types/ (Wave 1-2):**
- `Tree`, `ScopedTree`, `Terminal`, `ScopedTerminal`, `RootNode`
- `Trigger`, `TriggerContext`, `Condition`
- `EffectType`, `ReversibleEffectType`
- `EntityKind`, `RouteType`, `StampTarget`
- `ParticipantTarget`, `BumpTarget`, `ImpactTarget`, `DeathTarget`, `BoltLostTarget`

**From storage/ (Wave 2-3):**
- `BoundEffects`, `BoundEntry`, `StagedEffects`

**From components/ (Wave 2-3):**
- `EffectSourceChip`

**External types (from other domains, already existing):**
- `NodeState` (for is_node_active condition evaluator)
- `ShieldWall` component (for is_shield_active condition evaluator)

**Defined in this wave:**
- `ComboStreak { count: u32 }` resource (defined in `src/effect_v3/conditions/combo_active.rs`, used by is_combo_active evaluator, updated by `track_combo_streak` system in Wave 5+)

---

### Summary of Deliverables

| Category | Count | Files |
|----------|-------|-------|
| Traits | 3 | fireable.rs, reversible.rs, passive_effect.rs |
| Generic container | 1 | effect_stack.rs |
| Walking functions | 9 | walk_effects.rs, fire.rs, when.rs, once.rs, during.rs, until.rs, sequence.rs, on.rs, route.rs |
| Dispatch functions | 2 | fire_dispatch.rs, reverse_dispatch.rs |
| Command structs | 6 | stamp.rs, fire.rs, reverse.rs, route.rs, stage.rs, remove.rs |
| Command trait | 1 | ext.rs |
| Condition evaluators | 3 | node_active.rs, shield_active.rs, combo_active.rs |
| Passive effect configs | 8 | speed_boost/config.rs, size_boost/config.rs, damage_boost/config.rs, bump_force/config.rs, quick_stop/config.rs, vulnerable/config.rs, piercing/config.rs, ramping_damage/config.rs |
| Conversion helpers | 1 | reversible_to_effect_type in dispatch/mod.rs |
| Module wiring (mod.rs) | ~20 | Various mod.rs files (routing-only) |
| **Total production files** | **~34** | |
