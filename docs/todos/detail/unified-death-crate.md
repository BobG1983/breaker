# TODO #0 — Build `rantzsoft_dmg` crate (standalone)

> **Scope split (2026-04-22):** previously this todo bundled crate construction with the game port. Now split:
> - **TODO #0 (this file)** — Build the `rantzsoft_dmg` crate end-to-end, game untouched.
> - **TODO #1** — [Port game to `rantzsoft_dmg`](./port-to-rantzsoft-dmg.md).
>
> The crate must compile cleanly and pass its own integration tests before any `breaker-game` file is modified.

## Problem addressed

- The unified death pipeline (`src/shared/death_pipeline/`) is generic but lives inside `breaker-game`. Extraction to a `rantzsoft_*` crate creates a clean boundary and enables reuse by other 2D action games.
- The current 4-set chain (`ApplyDamage → DetectDeaths → HandleKill → ApplyHeal`) is too coarse to host mechanic mutators (Diffusion, Tether, Echo Strike, …). The new 11-set chain makes room for Emit/Mutate/Apply triplets on each of Damage/Kill/Heal.
- `EffectStack<DamageBoostConfig>` and `EffectStack<VulnerableConfig>` belong with the damage pipeline, not the effect system. Promoting them to first-class `DamageBoostStack` / `VulnerableStack` components in the crate decouples pipeline concerns from effect plumbing.
- Subsumes `audit/remediations/damage-message-mutator-chain.md` (its SystemSet expansion design lives inside this crate).

## Prerequisite for

- `iron-curtain-remove-invulnerable-filter.md` — `invulnerable_filter::<T>` is owned by this crate.
- `echo-strike-reads-damage-dealt.md` / `sympathy-reads-post-diffusion-damage.md` / `sympathy-design-doc-alignment.md` / `tether-design-doc-alignment.md` / `diffusion-flat-share-per-ring.md` — all ride on the 11-set chain delivered here.
- `damage-amplification-standardization.md` — Pattern B (consume-on-use) maps to `DamageBoostStack::one_shots`.
- `breaker-bolt-lost-effect-component.md` — BoltLost's parallel `BoltLifecycleSystems` triplet models on the pattern established here.

## Design

### Naming

- **Crate**: `rantzsoft_dmg` (lowercase-underscore per `rantzsoft-crates.md`).
- **Root plugin**: `RantzDmgPlugin`.
- **Ext trait**: `RantzDmgAppExt` on `App`.
- **Ext trait method**: `register_dmgable::<T: Dmgable>(&mut self) -> &mut Self`.
- **Marker trait**: `Dmgable: Component`. Crate-public. Replaces the game-side `GameEntity`. Each impl creates per-T message queues.
- **SystemSet enum**: `DmgSystems` (replaces `DeathPipelineSystems` — aligns with crate prefix).

### Mutation mechanism — `MessageMutator<T>`

Bevy 0.18 ships `MessageMutator<T>` (`bevy_ecs/src/message/message_mutator.rs`), whose docstring states: *"Mutably reads messages of type T… Ideal for chains of systems that all want to modify the same messages."* This is the mechanism the pipeline uses — no side-channel resource needed.

- `apply_damage_boosts::<T>`, `apply_vulnerable::<T>`, `invulnerable_filter::<T>`, and all game-side `MutateDamage` members use `MessageMutator<DamageDealt<T>>` and mutate `msg.amount` in place.
- `apply_damage::<T>` uses plain `MessageReader<DamageDealt<T>>` — reads the final, multi-system-mutated amount.
- Concurrency: `MessageMutator` systems of the same `T` cannot run concurrently (crate docs). Our `.chain()` ordering enforces this automatically.

### Crate layout

```
rantzsoft_dmg/
  Cargo.toml
  src/
    lib.rs                           // re-exports + doc root
    plugin.rs                        // RantzDmgPlugin
    app_ext.rs                       // RantzDmgAppExt + impl for App
    source_id.rs                     // SourceId(Cow<'static, str>)
    sets.rs                          // DmgSystems (11 variants)
    traits/
      mod.rs
      dmgable.rs                     // Dmgable marker trait
    components/
      mod.rs
      hp.rs                          // Hp
      dead.rs                        // Dead
      invulnerable.rs                // Invulnerable
      killed_by.rs                   // KilledBy
      heal_cap.rs                    // HealCap enum
      damage_boost_stack.rs          // DamageBoostStack (+ unit tests)
      vulnerable_stack.rs            // VulnerableStack (+ unit tests)
    messages/
      mod.rs
      damage_dealt.rs                // DamageDealt<T: Dmgable>
      heal_dealt.rs                  // HealDealt<T: Dmgable>
      kill_yourself.rs               // KillYourself<T: Dmgable>
      destroyed.rs                   // Destroyed<T: Dmgable>
      despawn_entity.rs              // DespawnEntity
    systems/
      mod.rs
      apply_damage_boosts.rs         // apply_damage_boosts::<T>
      apply_vulnerable.rs            // apply_vulnerable::<T>
      invulnerable_filter.rs         // invulnerable_filter::<T>
      apply_damage.rs                // apply_damage::<T>
      detect_deaths.rs               // detect_deaths::<T>
      handle_kill.rs                 // handle_kill::<T>
      apply_heal.rs                  // apply_heal::<T>
      process_despawn.rs             // process_despawn_requests
  tests/                             // integration tests — public API only
    pipeline_ordering.rs
    damage_boost.rs
    vulnerable.rs
    invulnerable.rs
    register.rs
    source_id.rs
    heal.rs
    despawn.rs
    mutator_chain_mutates_msg.rs
```

### `Dmgable` marker trait

```rust
// rantzsoft_dmg/src/traits/dmgable.rs
use bevy::prelude::*;

/// Marker trait for entity types that participate in the damage/kill/heal
/// pipeline. Each `impl Dmgable for X` establishes an independent per-`X`
/// message queue for `DamageDealt<X>`, `HealDealt<X>`, `KillYourself<X>`,
/// `Destroyed<X>`, and monomorphizes the pipeline systems for `X`.
pub trait Dmgable: Component {}
```

Zero game knowledge. Game-side impls (`impl Dmgable for Bolt`, etc.) land in TODO #1.

### `SourceId`

```rust
// rantzsoft_dmg/src/source_id.rs
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub Cow<'static, str>);

impl From<&'static str> for SourceId { fn from(s: &'static str) -> Self { Self(Cow::Borrowed(s)) } }
impl From<String>       for SourceId { fn from(s: String)       -> Self { Self(Cow::Owned(s))   } }
impl std::fmt::Display for SourceId { /* inline Cow */ }
```

Zero-alloc for `&'static str` literals (`SourceId::from("protocol:reckless_dash")`); owned allocation only for dynamic ids (`SourceId::from(format!("hazard:resonance:wave:{bits}"))`).

### Stacks — Vec-backed (append semantics)

```rust
// rantzsoft_dmg/src/components/damage_boost_stack.rs
#[derive(Component, Debug, Default)]
pub struct DamageBoostStack {
    persistent: Vec<(SourceId, f32)>,
    one_shots:  Vec<f32>,
}

impl DamageBoostStack {
    pub fn add(&mut self, source: SourceId, multiplier: f32);          // appends — same source repeats
    pub fn remove_by_source(&mut self, source: &SourceId);             // retain(|(s, _)| s != source)
    pub fn add_one_shot(&mut self, multiplier: f32);
    pub fn aggregate_persistent(&self) -> f32;                         // product of all persistent mults
    pub fn aggregate_and_consume_one_shots(&mut self) -> f32;          // persistent × drain(one_shots)
    pub fn is_empty(&self) -> bool;
}
```

**Why Vec (not HashMap):** `assets/chips/standard/damage_boost.chip.ron` sets `max_taken: 5`. Today's `EffectStack::push` appends and the aggregator multiplies every entry. HashMap upsert would collapse five stacked Damage Boost chips to one. Vec preserves the append-multiply semantic. `VulnerableStack` uses the identical API.

### System sets — 11 variants

```rust
// rantzsoft_dmg/src/sets.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DmgSystems {
    EmitDamage,         // game-side: writers of DamageDealt<T>
    ApplyDamageBoosts,  // crate-owned: apply_damage_boosts::<T>
    MutateDamage,       // game-side: Diffusion / Tether / Echo Strike (EMPTY at crate level)
    ApplyVulnerable,    // crate-owned: apply_vulnerable::<T>
    ApplyDamage,        // crate-owned: (invulnerable_filter::<T>, apply_damage::<T>).chain()

    EmitKill,           // crate-owned: detect_deaths::<T>
    MutateKill,         // game-side reserved (revive-on-kill / last-stand) (EMPTY at crate level)
    ApplyKill,          // crate-owned: handle_kill::<T>

    EmitHeal,           // game-side: writers of HealDealt<T>
    MutateHeal,         // game-side reserved (heal dampening) (EMPTY at crate level)
    ApplyHeal,          // crate-owned: apply_heal::<T>
}
```

Cross-set `.chain()` configured once in `RantzDmgPlugin::build` under `FixedUpdate`.

### Ordering rationale

- **EmitDamage → ApplyDamageBoosts**: dealer-side multipliers are folded into `msg.amount` before any mutator sees the message.
- **ApplyDamageBoosts → MutateDamage**: game-side mutators (Diffusion ring-share, Tether partner redirect, Echo Strike siblings) read and mutate an already-amplified `amount`, and can emit derived sibling messages.
- **MutateDamage → ApplyVulnerable**: target-side vulnerability applies to both primary and mutator-spawned sibling messages.
- **ApplyVulnerable → ApplyDamage**: final amount (post-mutation, post-vulnerability) is what reaches Hp decrement.
- **Inside ApplyDamage**: `invulnerable_filter::<T>` runs before `apply_damage::<T>` via `.chain()`, zeroing amount for `Invulnerable` targets — catches the entire chain, including siblings.
- **ApplyDamage → EmitKill**: Hp must be written before death is detected.
- **EmitKill → MutateKill → ApplyKill**: MutateKill is reserved for last-stand / revive-on-kill mechanics that can cancel a pending kill.
- **ApplyKill → ApplyHeal**: `handle_kill::<T>` inserts `Dead`; `apply_heal::<T>`'s `Without<Dead>` filter then blocks in-tick heal-revival of damage-killed entities.
- **EmitHeal / MutateHeal / ApplyHeal**: symmetric mirror for heal lifecycle. Sympathy-style post-apply reactors hook into `EmitHeal` after damage has resolved.
- **`process_despawn_requests` in `FixedPostUpdate`**: despawns run after all FixedUpdate systems finish, preventing mid-tick entity-lookup races.

### Messages

```rust
#[derive(Message)]
pub struct DamageDealt<T: Dmgable> {
    pub dealer:  Option<Entity>,
    pub target:  Entity,
    pub amount:  f32,
    pub source:  Option<SourceId>,
    pub _marker: PhantomData<T>,
}

#[derive(Message)]
pub struct HealDealt<T: Dmgable> {
    pub healer:  Option<Entity>,
    pub target:  Entity,
    pub amount:  f32,
    pub source:  Option<SourceId>,
    pub cap:     HealCap,
    pub _marker: PhantomData<T>,
}

#[derive(Message)]
pub struct KillYourself<T: Dmgable> {
    pub victim:  Entity,
    pub killer:  Option<Entity>,
    pub _marker: PhantomData<T>,
}

#[derive(Message)]
pub struct Destroyed<T: Dmgable> {
    pub victim:     Entity,
    pub killer:     Option<Entity>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
    pub _marker:    PhantomData<T>,
}

#[derive(Message)]
pub struct DespawnEntity { pub entity: Entity }
```

All generic messages ship a manual `Clone` impl (PhantomData is always Clone; T isn't required to be).

### `RantzDmgPlugin` — base wiring

```rust
// rantzsoft_dmg/src/plugin.rs
pub struct RantzDmgPlugin;

impl Plugin for RantzDmgPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnEntity>();

        app.configure_sets(FixedUpdate, (
            DmgSystems::EmitDamage,
            DmgSystems::ApplyDamageBoosts,
            DmgSystems::MutateDamage,
            DmgSystems::ApplyVulnerable,
            DmgSystems::ApplyDamage,
            DmgSystems::EmitKill,
            DmgSystems::MutateKill,
            DmgSystems::ApplyKill,
            DmgSystems::EmitHeal,
            DmgSystems::MutateHeal,
            DmgSystems::ApplyHeal,
        ).chain());

        app.add_systems(FixedPostUpdate, process_despawn_requests);
    }
}
```

No per-`T` wiring. No hardcoded entity knowledge.

### `RantzDmgAppExt::register_dmgable`

```rust
// rantzsoft_dmg/src/app_ext.rs
pub trait RantzDmgAppExt {
    fn register_dmgable<T: Dmgable>(&mut self) -> &mut Self;
}

impl RantzDmgAppExt for App {
    fn register_dmgable<T: Dmgable>(&mut self) -> &mut Self {
        self.add_message::<DamageDealt<T>>()
            .add_message::<HealDealt<T>>()
            .add_message::<KillYourself<T>>()
            .add_message::<Destroyed<T>>()
            .add_systems(FixedUpdate, (
                apply_damage_boosts::<T>.in_set(DmgSystems::ApplyDamageBoosts),
                apply_vulnerable::<T>.in_set(DmgSystems::ApplyVulnerable),
                (invulnerable_filter::<T>, apply_damage::<T>)
                    .chain()
                    .in_set(DmgSystems::ApplyDamage),
                detect_deaths::<T>.in_set(DmgSystems::EmitKill),
                handle_kill::<T>.in_set(DmgSystems::ApplyKill),
                apply_heal::<T>.in_set(DmgSystems::ApplyHeal),
            ))
    }
}
```

Consumer: `app.add_plugins(RantzDmgPlugin).register_dmgable::<Bolt>().register_dmgable::<Cell>()...`

### Systems — signatures

| System | Params | Behavior |
|---|---|---|
| `apply_damage_boosts::<T>` | `MessageMutator<DamageDealt<T>>` + `Query<&mut DamageBoostStack>` | For each msg with `dealer`, `msg.amount *= dealer_stack.aggregate_and_consume_one_shots()`. |
| `apply_vulnerable::<T>` | `MessageMutator<DamageDealt<T>>` + `Query<&mut VulnerableStack>` | For each msg, `msg.amount *= target_stack.aggregate_and_consume_one_shots()`. |
| `invulnerable_filter::<T>` | `MessageMutator<DamageDealt<T>>` + `Query<(), With<Invulnerable>>` | If target has `Invulnerable`, `msg.amount = 0.0`. |
| `apply_damage::<T>` | `MessageReader<DamageDealt<T>>` + `Query<&mut Hp, (With<T>, Without<Dead>)>` + `Commands` | Hp -= amount. On killing blow (`was_positive && hp.current <= 0.0`), **insert** `KilledBy { dealer }` via Commands. First-kill-wins falls out naturally: subsequent same-tick DamageDealt hit `was_positive = false` because Hp already ≤ 0. |
| `detect_deaths::<T>` | `Query<(Entity, Option<&KilledBy>, &Hp), (With<T>, Without<Dead>)>` + `MessageWriter<KillYourself<T>>` | For each entity with Hp ≤ 0, emit `KillYourself { victim, killer: killed_by.and_then(\|k\| k.dealer), .. }`. `Option` handles Hp-set-to-zero-without-DamageDealt callers gracefully (killer = None). |
| `handle_kill::<T>` | `MessageReader<KillYourself<T>>` + `Local<HashSet<Entity>>` + `Query<&Position2D, (With<T>, Without<Dead>)>` + `Query<&Position2D>` + writers + `Commands` | Insert `Dead`, emit `Destroyed<T>`, enqueue `DespawnEntity`. Idempotency via `Local<HashSet<Entity>>` **cleared at the top of the fn** (`seen.clear()` first line) — amortizes allocation across ticks (capacity reused, not re-allocated) while guaranteeing clean per-tick state. Combined with `Without<Dead>` on the victim query. Dedupes multiple `KillYourself<T>` messages for the same victim within one invocation, necessary because `Dead` inserts via Commands are deferred. Size bounded by tick-local message count. |
| `apply_heal::<T>` | `MessageReader<HealDealt<T>>` + `Query<&mut Hp, (With<T>, Without<Dead>, Without<Invulnerable>)>` | Hp += amount clamped by `HealCap::Starting` or `::Max`. |
| `process_despawn_requests` | `MessageReader<DespawnEntity>` + `Commands` | `commands.entity(e).try_despawn()`. |

### Public API (`lib.rs` re-exports)

```rust
pub use plugin::RantzDmgPlugin;
pub use app_ext::RantzDmgAppExt;
pub use sets::DmgSystems;
pub use source_id::SourceId;
pub use traits::Dmgable;
pub use components::{Hp, Dead, Invulnerable, KilledBy, HealCap, DamageBoostStack, VulnerableStack};
pub use messages::{DamageDealt, HealDealt, KillYourself, Destroyed, DespawnEntity};
```

Crate-internal: the individual systems (consumers wire through `register_dmgable`).

### What the crate does NOT own

- Mechanic-specific mutators (Diffusion, Tether, Echo Strike) — game-side (TODO #2).
- Reactors (Sympathy, Iron Curtain) — game-side.
- Emitters — game-side; bolt-impact, chip damage, shockwave, heal emitters live in their home domain and tag-into `EmitDamage` / `EmitHeal`.
- Game type markers (Bolt, Cell, Wall, Breaker, Salvo) — game-side.
- SourceId string constants — game-side.
- BoltLost and its parallel lifecycle triplet — bolt domain, not a death event.

## Implementation phases

7 phases, each its own TDD commit. Detailed scope per phase lives in the plan file; summary:

| Phase | Scope |
|---|---|
| **P1** | Scaffold: `Cargo.toml`, `src/lib.rs`, workspace member, cargo aliases. |
| **P2** | Core types: `Hp`, `Dead`, `Invulnerable`, `KilledBy`, `HealCap`, `SourceId`, `Dmgable`. |
| **P3** | Generic messages (`DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity`). |
| **P4** | Sets + `RantzDmgPlugin` skeleton (11-variant chain + `DespawnEntity` + `process_despawn_requests` stub). |
| **P5** | `DamageBoostStack` + `VulnerableStack` (Vec-backed). |
| **P6** | All seven generic systems + non-generic `process_despawn_requests` + `RantzDmgAppExt::register_dmgable`. |
| **P7** | Crate integration test suite (see §Tests). |

## Tests (crate-level)

All live in `rantzsoft_dmg/tests/` and exercise the public API only. Each test uses `#[derive(Component)] struct TestT; impl Dmgable for TestT {}`.

- `pipeline_ordering::orders_emit_mutate_apply`
- `damage_boost::aggregation_persistent_only`
- `damage_boost::aggregation_with_one_shots`
- `damage_boost::append_does_not_collapse_same_source` (pins Vec semantics)
- `damage_boost::remove_by_source`
- `vulnerable::aggregation_persistent_only`
- `vulnerable::aggregation_with_one_shots`
- `invulnerable::zeroes_amount`
- `invulnerable::applies_to_mutator_spawned_siblings`
- `register::register_dmgable_adds_all_messages_and_systems`
- `register::register_dmgable_idempotent_for_same_T`
- `source_id::borrowed_vs_owned` / `source_id::equality` / `source_id::display`
- `heal::cap_starting_vs_max`
- `heal::blocked_by_dead` / `heal::blocked_by_invulnerable` / `heal::respects_ceiling`
- `despawn::process_runs_in_fixed_post_update`
- `mutator_chain_mutates_msg::in_place_mutation_observable_by_apply_damage`

## Cargo.toml + aliases

Workspace-root `Cargo.toml` adds `"rantzsoft_dmg"` to `workspace.members`.

`.cargo/config.toml`:
```
dmgcheck  = "check  -p rantzsoft_dmg --features bevy/dynamic_linking"
dmgclippy = "clippy -p rantzsoft_dmg --all-targets --features bevy/dynamic_linking"
dmgtest   = "test   -p rantzsoft_dmg --features bevy/dynamic_linking"
```

`all-dcheck` / `all-dclippy` / `all-dtest` use `--workspace` — pick up the new crate automatically.

`rantzsoft_dmg/Cargo.toml` dependencies:
- `bevy 0.18.1` (default-features=false, features=["2d"])
- `tracing = "0.1"`
- `rantzsoft_spatial2d = { path = "../rantzsoft_spatial2d" }` (required by `handle_kill` for `Position2D`)

## Scope boundary

**In**: everything inside `rantzsoft_dmg/`, workspace `Cargo.toml` member list, `.cargo/config.toml` alias entries.

**Out**: every file under `breaker-game/`. The crate ships green; porting is TODO #1's job.

## TODO entry

```
0. **[done]** Build `rantzsoft_dmg` crate (standalone) — 11-variant Emit/Mutate/Apply pipeline
   for Damage/Kill/Heal via `MessageMutator`-based mutation, `DamageBoostStack` /
   `VulnerableStack` components, `SourceId`, `Dmgable` marker trait, `register_dmgable::<T: Dmgable>`
   ext trait. No game changes. — [detail](detail/unified-death-crate.md)
```

**Status: COMPLETE** — Shipped in commits 59932cf1, a0bc44e1, 7237da9a, d9af92c9, 47f1b555, b7f11a25, aefcdf94. Crate at `rantzsoft_dmg/`, all integration tests green. Zero `breaker-game` changes. Port is TODO #1.

## Followup

- **TODO #1 — Port game to `rantzsoft_dmg`**: [detail](./port-to-rantzsoft-dmg.md). Registers every game entity, deletes `breaker-game/src/shared/death_pipeline/`, migrates `EffectStack<DamageBoostConfig>` / `EffectStack<VulnerableConfig>` callers, sweeps heal emitters.
- **TODO #2 — Mutators domain refactor** (old TODO #1): closes the MutateDamage chain (Diffusion / Tether / Echo Strike), deletes `apply_damage_to_cells`, sweeps damage emitters into `EmitDamage`. Note: Deferral A (mutation mechanism design) is DISSOLVED — `MessageMutator<DamageDealt<T>>` is the mechanism.
