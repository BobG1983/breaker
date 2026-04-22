# Extract the unified death pipeline into `rantzsoft_dmg` crate

## Problems addressed

- The unified death pipeline is generic and game-agnostic but currently lives inside `breaker-game` at `src/shared/death_pipeline/`. Extracting it to a `rantzsoft_*` crate makes it reusable by other 2D action games and forces a clean boundary between generic pipeline plumbing and game-specific mechanics.
- Today's `EffectStack<DamageBoostConfig>` and `EffectStack<VulnerableConfig>` live in `effect_v3/` and are aggregated inside `bolt_cell_collision` + `apply_damage_to_cells`. They're part of the damage pipeline conceptually, not the effect system. Moving them into the crate as first-class components (`DamageBoostStack`, `VulnerableStack`) decouples pipeline concerns from effect plumbing.
- Subsumes `damage-message-mutator-chain.md`. That remediation's design (SystemSet expansion, Emit/Mutate/Apply triplets for Damage/Kill/Heal) lives inside the new crate.

## Prerequisite for

This remediation is foundational. Downstream remediations that depend on it:
- `iron-curtain-remove-invulnerable-filter.md` — `invulnerable_filter::<T>` moves to the crate.
- `echo-strike-reads-damage-dealt.md` — Echo Strike becomes a game-side MutateDamage chain member.
- `sympathy-reads-post-diffusion-damage.md` — Sympathy becomes a game-side `EmitHeal` reactor.
- `sympathy-design-doc-alignment.md` — align design doc with chain position.
- `tether-design-doc-alignment.md` — Tether becomes a game-side MutateDamage chain member; `TetherRedirectBuffer` deleted.
- `diffusion-flat-share-per-ring.md` — Diffusion becomes a game-side MutateDamage chain member.
- `damage-amplification-standardization.md` — Pattern B consume-on-use moves to `DamageBoostStack::one_shots`; `apply_damage_boosts::<T>` owns aggregation.
- `breaker-bolt-lost-effect-component.md` — BoltLost gets a parallel `BoltLifecycleSystems` triplet (bolt-domain-owned, not in the crate).

## Design

### Crate layout

```
rantzsoft_dmg/
  src/
    lib.rs                 // Plugin + RantzDmgAppExt trait
    source_id.rs           // SourceId newtype
    components.rs          // Hp, Dead, Invulnerable, KilledBy, DamageBoostStack, VulnerableStack
    messages.rs            // DamageDealt<T>, KillYourself<T>, HealDealt<T>, Destroyed<T>, DespawnEntity
    heal_cap.rs            // HealCap enum
    sets.rs                // DeathPipelineSystems enum (11 variants)
    systems/
      apply_damage.rs
      apply_heal.rs
      detect_deaths.rs
      handle_kill.rs
      apply_damage_boosts.rs
      apply_vulnerable.rs
      invulnerable_filter.rs
      process_despawn_requests.rs
  Cargo.toml
```

Workspace member in `Cargo.toml`. Naming convention: `rantzsoft_dmg` (lowercase, underscore-separated, per `rantzsoft-crates.md`).

### `SourceId`

```rust
// rantzsoft_dmg/src/source_id.rs
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub Cow<'static, str>);

impl From<&'static str> for SourceId {
    fn from(s: &'static str) -> Self { Self(Cow::Borrowed(s)) }
}
impl From<String> for SourceId {
    fn from(s: String) -> Self { Self(Cow::Owned(s)) }
}
```

Compile-time literals (`SourceId::from("protocol:reckless_dash")`) are zero-alloc. Dynamic ids (Resonance per-wave: `SourceId::from(format!("hazard:resonance:wave:{bits}"))`) allocate only when the caller needs them.

### Stacks

```rust
// rantzsoft_dmg/src/components.rs
use std::collections::HashMap;
use bevy::prelude::*;
use crate::source_id::SourceId;

#[derive(Component, Debug, Default)]
pub struct DamageBoostStack {
    persistent: HashMap<SourceId, f32>,
    one_shots:  Vec<f32>,
}

impl DamageBoostStack {
    pub fn add(&mut self, source: SourceId, multiplier: f32) {
        // Upsert: replace any existing entry for this source.
        self.persistent.insert(source, multiplier);
    }
    pub fn remove(&mut self, source: &SourceId) {
        self.persistent.remove(source);
    }
    pub fn add_one_shot(&mut self, multiplier: f32) {
        self.one_shots.push(multiplier);
    }
    /// Computes the aggregate product of persistent + one-shot multipliers, then
    /// clears the one-shots. Used by `apply_damage_boosts::<T>` exactly once per
    /// damage message in which this dealer participates.
    pub fn aggregate_and_consume_one_shots(&mut self) -> f32 {
        let p = self.persistent.values().copied().product::<f32>();
        let s = self.one_shots.drain(..).product::<f32>();
        if p == 0.0 { s } else if s == 0.0 { p } else { p * s }
    }
}

#[derive(Component, Debug, Default)]
pub struct VulnerableStack {
    persistent: HashMap<SourceId, f32>,
    one_shots:  Vec<f32>,
}

// Same API as DamageBoostStack — mirrored for target-side multipliers.
impl VulnerableStack {
    pub fn add(&mut self, source: SourceId, multiplier: f32) { /* ... */ }
    pub fn remove(&mut self, source: &SourceId) { /* ... */ }
    pub fn add_one_shot(&mut self, multiplier: f32) { /* ... */ }
    pub fn aggregate_and_consume_one_shots(&mut self) -> f32 { /* ... */ }
}
```

**Semantics:**
- `add(source, multiplier)` is UPSERT — idempotent. A chip system that fires every tick with the same source replaces, not double-adds. This is the `retain_by_source` pattern from today's `EffectStack`, preserved.
- `remove(source)` is explicit reversal. Chip reversal or protocol-exit calls this.
- `add_one_shot(multiplier)` appends a single-use multiplier that is consumed on the first damage aggregation. Pattern B (Burnout mega-bump, Debt Collector cashout, Reckless Dash risky catch).
- `aggregate_and_consume_one_shots()` is called by the crate-owned applier; callers never invoke it directly.

### SystemSets (11 variants)

```rust
// rantzsoft_dmg/src/sets.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeathPipelineSystems {
    EmitDamage,
    ApplyDamageBoosts,
    MutateDamage,
    ApplyVulnerable,
    ApplyDamage,

    EmitKill,
    /// Reserved for last-stand / revive-on-kill mechanics. No systems today;
    /// pub enum variants in a lib crate don't trigger dead_code warnings.
    MutateKill,
    ApplyKill,

    EmitHeal,
    /// Reserved for heal-dampening mechanics. No systems today.
    MutateHeal,
    ApplyHeal,
}
```

Cross-set chain (configured once inside `RantzDmgPlugin::build`):

```rust
app.configure_sets(FixedUpdate, (
    DeathPipelineSystems::EmitDamage,
    DeathPipelineSystems::ApplyDamageBoosts,
    DeathPipelineSystems::MutateDamage,
    DeathPipelineSystems::ApplyVulnerable,
    DeathPipelineSystems::ApplyDamage,
    DeathPipelineSystems::EmitKill,
    DeathPipelineSystems::MutateKill,
    DeathPipelineSystems::ApplyKill,
    DeathPipelineSystems::EmitHeal,
    DeathPipelineSystems::MutateHeal,
    DeathPipelineSystems::ApplyHeal,
).chain());
```

### Plugin + `register_damage_type::<T>`

```rust
// rantzsoft_dmg/src/lib.rs
pub struct RantzDmgPlugin;

impl Plugin for RantzDmgPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnEntity>();
        app.add_systems(FixedUpdate, process_despawn_requests.in_set(...));
        app.configure_sets(FixedUpdate, /* the 11-variant chain */);
    }
}

pub trait RantzDmgAppExt {
    fn register_damage_type<T: Component>(&mut self) -> &mut Self;
}

impl RantzDmgAppExt for App {
    fn register_damage_type<T: Component>(&mut self) -> &mut Self {
        self.add_message::<DamageDealt<T>>()
            .add_message::<KillYourself<T>>()
            .add_message::<HealDealt<T>>()
            .add_message::<Destroyed<T>>()
            .add_systems(FixedUpdate, (
                apply_damage_boosts::<T>.in_set(DeathPipelineSystems::ApplyDamageBoosts),
                apply_vulnerable::<T>.in_set(DeathPipelineSystems::ApplyVulnerable),
                (invulnerable_filter::<T>, apply_damage::<T>).chain().in_set(DeathPipelineSystems::ApplyDamage),
                detect_deaths::<T>.in_set(DeathPipelineSystems::EmitKill),
                handle_kill::<T>.in_set(DeathPipelineSystems::ApplyKill),
                apply_heal::<T>.in_set(DeathPipelineSystems::ApplyHeal),
            ))
    }
}
```

### Game-side wiring

```rust
// breaker-game/src/game.rs (or wherever plugins are composed)
app.add_plugins(RantzDmgPlugin)
   .register_damage_type::<Bolt>()
   .register_damage_type::<Cell>()
   .register_damage_type::<Wall>()
   .register_damage_type::<Breaker>();

// Mechanic-specific mutators live in breaker-game/src/mutators/ via MutatorsPlugin:
app.add_plugins(MutatorsPlugin);
```

`MutatorsPlugin` owns the intra-`MutateDamage` chain ordering (see `damage-message-mutator-chain.md` for members and ordering rationale).

### What the crate does NOT own

- Mechanic-specific mutators: Diffusion, Tether, Echo Strike. Game-side in `breaker-game/src/mutators/` and their home domains.
- Reactors: Sympathy (reads `DamageDealt<Cell>` post-apply, emits `HealDealt<Cell>`). Game-side.
- Emitters: Iron Curtain, Burnout, bolt-impact base-damage emission, chip-driven damage. Game-side, registered into `DeathPipelineSystems::EmitDamage`.
- Game-specific component markers (Bolt, Cell, Wall, Breaker). Game-side.
- Source-id string constants. Game-side.
- BoltLost and its parallel Emit/Mutate/Apply triplet. Lives in bolt domain with its own `BoltLifecycleSystems` — see `breaker-bolt-lost-effect-component.md`. NOT a death event.

### Migration: delete `apply_damage_to_cells`

The cells-domain `apply_damage_to_cells` system and its 361-line file (plus `tests/` directory) are DELETED. `Cell` registers via `app.register_damage_type::<Cell>()`, which adds the generic `apply_damage::<Cell>` to `ApplyDamage`. The cells-specific BFS (Diffusion) and redirect math (Tether) move to their respective hazard-owned `MutateDamage` chain members per `damage-message-mutator-chain.md`.

### Migration: delete `EffectStack<DamageBoostConfig>` and `EffectStack<VulnerableConfig>`

Both are replaced by the new `DamageBoostStack` and `VulnerableStack` components from the crate. Migration mechanics:

- Every caller that does `commands.fire_effect(entity, EffectType::DamageBoost(cfg), source)` becomes `damage_boost_stack.add(SourceId::from(source), cfg.multiplier)` (via a direct component query or a small helper).
- Every `reverse_effect(..., DamageBoost, source)` becomes `.remove(SourceId::from(source))`.
- Every `fire_effect(..., DamageBoost { consume_on_use: Some(_) }, source)` becomes `.add_one_shot(multiplier)` — source tag is irrelevant for one-shots (they're anonymous).
- Same pattern for `VulnerableConfig` → `VulnerableStack`.

The `effect_v3` dispatch paths for `DamageBoost` and `Vulnerable` variants are DELETED. The `EffectType::DamageBoost` and `EffectType::Vulnerable` enum variants are REMOVED from `effect_v3/types/effect_type.rs`. Other effect types (SpeedBoost, SizeBoost, Piercing, TetherBeam, TimePenalty, LoseLife, etc.) are UNCHANGED — only the two damage-pipeline-adjacent stacks migrate out of the effect system.

### Migration: `bolt_cell_collision` simplification

Today `bolt_cell_collision:281` computes `effective_damage * vulnerability.aggregate()` and emits `DamageDealt<Cell>.amount` with BOTH the bolt's damage boost aggregate AND the cell's vulnerability aggregate baked in. After this remediation:

- `bolt_cell_collision` emits `DamageDealt<Cell> { amount: base_damage, dealer: Some(bolt), target: cell, .. }` — raw base damage only.
- `apply_damage_boosts::<Cell>` (crate-owned) reads the bolt's `DamageBoostStack`, aggregates, applies to `msg.amount`. Runs in `ApplyDamageBoosts`, after `EmitDamage`, before `MutateDamage`.
- `apply_vulnerable::<Cell>` (crate-owned) reads the target cell's `VulnerableStack`, aggregates, applies to `msg.amount`. Runs in `ApplyVulnerable`, after `MutateDamage`, before `ApplyDamage`.
- The collision system still needs a LOCAL `effective_damage` calc for pierce-lookahead ("will this hit kill the cell?"). That stays — it's a local read of the same stacks, not a message-field concern.

### Migration: other emitters

Every system that writes `DamageDealt<T>` moves into `DeathPipelineSystems::EmitDamage`:

- `iron_curtain_on_bolt_lost` (protocol) → `EmitDamage`
- Burnout shockwave emitter → `EmitDamage`
- Fracture debris-damage emitter (if applicable) → `EmitDamage`
- Chip systems that emit direct damage → `EmitDamage`

Effect-system ticks that MAY emit damage (depending on stack state) use `.before(DeathPipelineSystems::EmitDamage)` instead of `.in_set()` — they're upstream producers that the pipeline treats as pre-Emit.

Heal emitters similarly move into `DeathPipelineSystems::EmitHeal`:
- Renewal periodic heal → `EmitHeal`
- Sympathy heal-adjacent → `EmitHeal` (post-apply reactor; `EmitHeal` sits after `ApplyDamage` in the cross-set chain, so Sympathy sees final post-mutation damage amounts)
- Chip heal effects → `EmitHeal`

### Process_despawn_requests

Lives in the crate. `DespawnEntity` message is also in the crate. Runs in `FixedPostUpdate` (outside the Fixed-main-loop death pipeline), consistent with current behavior.

## Remediation plan

Land in phases. Each phase is its own TDD commit.

1. **Crate skeleton.** Create `rantzsoft_dmg/` workspace member. Move `Hp`, `Dead`, `Invulnerable`, `KilledBy`, `HealCap`, `DamageDealt<T>`, `KillYourself<T>`, `HealDealt<T>`, `Destroyed<T>`, `DespawnEntity`, `detect_deaths::<T>`, `handle_kill::<T>`, `apply_damage::<T>`, `apply_heal::<T>`, `process_despawn_requests` from `breaker-game/src/shared/death_pipeline/` into the crate. Game-side imports update via prelude re-exports. `DeathPipelineSystems` stays 4-variant (current shape) in this phase — no behavior change yet.

2. **Expand `DeathPipelineSystems` to 11 variants.** Rename `DetectDeaths` → `EmitKill`, `HandleKill` → `ApplyKill`. Add `EmitDamage`, `ApplyDamageBoosts`, `MutateDamage`, `ApplyVulnerable`, `EmitHeal`, `MutateKill`, `MutateHeal`. Configure the full cross-set chain. Empty variants need no annotation — pub enum variants in a lib don't trigger dead_code.

3. **Add `SourceId`, `DamageBoostStack`, `VulnerableStack` to the crate.** Add `apply_damage_boosts::<T>` and `apply_vulnerable::<T>` systems. Add `register_damage_type::<T>` extension trait.

4. **Migrate callers off `EffectStack<DamageBoostConfig>` and `EffectStack<VulnerableConfig>`.** Every `fire_effect(DamageBoost, source)` becomes `stack.add(source.into(), mult)`. Every `reverse_effect` becomes `stack.remove`. One-shots via `add_one_shot`. Delete `effect_v3::effects::damage_boost` and `effect_v3::effects::vulnerable` modules. Delete the two `EffectType` variants. Update tests.

5. **Simplify `bolt_cell_collision`.** Emit raw `base_damage` in `DamageDealt<Cell>`. Keep local `effective_damage` for pierce lookahead. Tests update: message amount now base-only; Hp delta tests still assert final post-chain amount.

6. **Delete `apply_damage_to_cells`.** Add `Cell` to `register_damage_type` list. Move Diffusion + Tether + Echo Strike into `MutateDamage` chain per `damage-message-mutator-chain.md`. Move `invulnerable_filter` into `ApplyDamage` as a crate-owned system.

7. **Sweep emitters into `EmitDamage` / `EmitHeal` sets.** Iron Curtain, Burnout, chip systems, Renewal, Sympathy. Drop per-emitter `Without<Invulnerable>` filters.

Each phase lands with its own tests and Standard Verification Tier gate.

## Tests (crate-level)

Add `rantzsoft_dmg/tests/` integration tests that exercise the pipeline with dummy entity types:

- `pipeline_orders_emit_mutate_apply` — register a dummy T, drive a DamageDealt, assert systems run in the 11-set order.
- `damage_boost_aggregation` — add persistent + one-shot entries, assert aggregate product, assert one-shot is consumed.
- `vulnerable_aggregation` — same for VulnerableStack.
- `invulnerable_zeroes_amount` — mark target Invulnerable, assert final amount is zero.
- `register_damage_type_idempotent` — calling register twice doesn't double-schedule systems.
- `source_id_upsert_semantics` — adding same source twice replaces, doesn't double-count.
- `process_despawn_requests_runs_in_fixed_post_update` — lifecycle test.

Game-side tests stay in `breaker-game` and continue to exercise the pipeline end-to-end.

## Cargo.toml

Add workspace member; game crate adds dependency. Alias entries per `cargo.md`:

```toml
# Workspace root Cargo.toml aliases
dmgcheck = "check -p rantzsoft_dmg --features bevy/dynamic_linking"
dmgclippy = "clippy -p rantzsoft_dmg --features bevy/dynamic_linking"
dmgtest = "test -p rantzsoft_dmg --features bevy/dynamic_linking"
```

Add to the `all-dcheck` / `all-dclippy` / `all-dtest` workspace-wide aliases.

## Naming note

`rantzsoft_dmg` vs `rantzsoft_death`: the pipeline handles damage, kill, AND heal — "dmg" is a slight misnomer but is concise and the common industry shorthand. Alternative: `rantzsoft_death_pipeline`. Pick `rantzsoft_dmg` for brevity; the README inside the crate documents the full scope.

## TODO

Add to `docs/todos/TODO.md` Backlog:

> **[ready]** Extract unified death pipeline to `rantzsoft_dmg` crate — generic Emit/Mutate/Apply pipeline for Damage/Kill/Heal with `DamageBoostStack`/`VulnerableStack` components and `register_damage_type::<T>` registration. Subsumes `damage-message-mutator-chain.md`. See `audit/remediations/unified-death-crate.md`.
