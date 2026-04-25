# TODO #1 — Port `breaker-game` to `rantzsoft_dmg`

> Depends on **TODO #0 — [Build `rantzsoft_dmg` crate](./unified-death-crate.md)**. Crate ships standalone (all 11 sets, all systems, all types, all tests green) before this TODO starts. No interleaved crate/game work.

## Problem addressed

After TODO #0 lands, the game still has its own in-tree death pipeline in `breaker-game/src/shared/death_pipeline/` and still routes damage boost / vulnerability through `EffectStack<DamageBoostConfig>` / `EffectStack<VulnerableConfig>`. This TODO ports the game to consume `rantzsoft_dmg` directly:

- Deletes `breaker-game/src/shared/death_pipeline/` (13 production files + ~4k lines of tests — tests move with the crate as its integration suite; game keeps end-to-end behavioral tests that exercise the pipeline through real emitters).
- Wires game entity markers (Bolt, Cell, Wall, Breaker, Salvo) to the crate via `impl Dmgable` + `register_dmgable::<T>()` calls.
- Migrates every `EffectStack<DamageBoostConfig>` and `EffectStack<VulnerableConfig>` caller to the new crate-owned `DamageBoostStack` and `VulnerableStack` components.
- Sweeps heal emitters (Momentum, Sympathy, Renewal-adjacent) into `DmgSystems::EmitHeal`.
- Rewrites the `source_chip: Option<String>` field on existing call sites as `source: Option<SourceId>` on `DamageDealt<T>` / `HealDealt<T>`.

## Prerequisite for

- **TODO #2 — Mutators domain refactor** (old TODO #1). That TODO handles the Cell-side deletion of `apply_damage_to_cells` and migration of Diffusion / Tether / Echo Strike into the `MutateDamage` chain. Port (this TODO) must land first so the crate is the game's only source of Hp / damage / kill / heal types.
- All downstream remediations listed in the TODO #0 detail file's `Prerequisite for` section.

## Scope

**In**:
1. Wire the crate: `add_plugins(RantzDmgPlugin)` + `register_dmgable::<T>()` for each game entity.
2. `impl Dmgable for X` for `X ∈ {Bolt, Cell, Wall, Breaker, Salvo}` — one line per components file.
3. Delete `breaker-game/src/shared/death_pipeline/`.
4. Update imports across `breaker-game/` (prelude re-exports the crate's public items).
5. Migrate every caller of `EffectStack<DamageBoostConfig>` → `DamageBoostStack`; same for Vulnerable.
6. Rewrite `Fireable::fire` and `Reversible::reverse` impls for `EffectType::DamageBoost` and `EffectType::Vulnerable` to operate on the new stack components (RON schema preserved — `EffectType::DamageBoost(DamageBoostConfig)` variant kept).
7. Delete the `PassiveEffect` impls for `DamageBoostConfig` / `VulnerableConfig` + the `EffectStack<DamageBoostConfig>` / `EffectStack<VulnerableConfig>` instantiations.
8. Heal emitter set-tag sweep: Momentum, Sympathy, Renewal that currently write `HealDealt<T>` from inside `ApplyHeal` move to `EmitHeal`. No behavior change — set rename only.
9. Migrate `source_chip: Option<String>` call sites on `DamageDealt<T>` to `source: Option<SourceId>`. Same for `HealDealt<T>::source`.
10. Game-side `configure_sets` edge: `EffectV3Systems::Tick.before(DmgSystems::ApplyDamage)` added in `game.rs`. Expressed `.before()` from the game side — the game set is the one taking a dependency on a crate set, so the edge reads "effects tick comes before the crate's apply-damage." The crate itself is EffectV3-agnostic.
11. Update `breaker-game/src/prelude/death_pipeline.rs` (or equivalent) to re-export from `rantzsoft_dmg::*`.

**Out** (TODO #2 handles):
- Cell-side migration: `apply_damage_to_cells` deletion, `register_dmgable::<Cell>()` addition, Diffusion / Tether / Echo Strike migration to `MutateDamage` chain.
- Damage emitter sweep: Iron Curtain, Burnout, chip damage emitters moving to `EmitDamage`.
- `bolt_cell_collision` simplification (emit raw `base_damage` and let crate-owned `apply_damage_boosts::<Cell>` + `apply_vulnerable::<Cell>` do the math) — requires Cell to be registered, hence deferred to TODO #2.
- Dropping per-emitter `Without<Invulnerable>` query filters — waits on `invulnerable_filter::<Cell>` being active, which requires Cell registration (TODO #2).

**Note on Cell:** Cell is intentionally NOT registered via `register_dmgable::<Cell>()` in this TODO. The generic `apply_damage::<Cell>` path competes with the existing `apply_damage_to_cells` system (which owns Diffusion / Tether inline). Both running simultaneously would double-apply damage. TODO #2 deletes `apply_damage_to_cells` and simultaneously adds `register_dmgable::<Cell>()`. Until then: Bolt, Wall, Breaker, Salvo register; Cell stays on its legacy in-tree path.

## Waves

| Wave | Scope | Parallelism | Commit |
|---|---|---|---|
| **W1** | Crate wiring + `impl Dmgable` + prelude re-exports + delete `shared/death_pipeline/` + set-name rename (`DeathPipelineSystems` → `DmgSystems` call sites) | — | `refactor(dmg): wire rantzsoft_dmg crate, delete in-tree death pipeline` |
| **W2** | Migrate `EffectStack<DamageBoostConfig>` / `<VulnerableConfig>` callers to `DamageBoostStack` / `VulnerableStack`. Rewrite `Fireable::fire` / `Reversible::reverse` / `reverse_all_by_source`. Delete `PassiveEffect` impls. | 3-way by area: effect_v3 / bolt / tests | `refactor: migrate DamageBoost/Vulnerable callers to new stacks` |
| **W3** | Heal emitter set-tag sweep: Momentum, Sympathy, Renewal → `EmitHeal` | — | `refactor(dmg): move heal emitters to EmitHeal` |
| **W4** | `source_chip: Option<String>` → `source: Option<SourceId>` at every emit + read site | — | `refactor(dmg): migrate DamageDealt/HealDealt source field to SourceId` |
| **W5** | `SourceIdExt` typestate builder in `breaker-game/src/shared/source_id_ext.rs`; migrate every `SourceId` construction + `EffectStack<T>` key site from raw strings to the builder | — | `refactor(dmg): introduce SourceIdExt builder, migrate all source construction` |
| **W6** | `EffectCommandsExt` callsite audit — move every direct `commands.queue(<EffectCommand>)` onto the extension helper | — | `refactor(effect_v3): route effect commands through EffectCommandsExt` |
| **W7** | `Fireable::fire` refactor — split imperative damage-emitting `fire()` bodies into request message + scheduled consumer system in `DmgSystems::EmitDamage` | — | `refactor(dmg): schedule damage emission from Fireable::fire via request+consumer pattern` |
| **W8** | Pre-merge cleanup sweep — 7 items surfaced during W5–W7 review + 2 pre-existing scenario failures that block the Full Verification Tier | 7-way parallel per item (disjoint files) | 1 commit per item (see §-by-§ subjects below) |

### W1 — Wire crate + delete in-tree pipeline

- `breaker-game/Cargo.toml`: add `rantzsoft_dmg = { path = "../rantzsoft_dmg" }` to `[dependencies]`.
- `breaker-game/src/prelude/death_pipeline.rs` (or the existing equivalent): re-export `use rantzsoft_dmg::{...};` — `Hp`, `Dead`, `Invulnerable`, `KilledBy`, `HealCap`, `SourceId`, `Dmgable`, `DamageBoostStack`, `VulnerableStack`, `DamageDealt`, `HealDealt`, `KillYourself`, `Destroyed`, `DespawnEntity`, `DmgSystems`, `RantzDmgPlugin`, `RantzDmgAppExt`.
- Game-side `impl Dmgable for X` additions (one each):
  - `breaker-game/src/bolt/components.rs` → `impl Dmgable for Bolt {}`
  - `breaker-game/src/cells/components.rs` → `impl Dmgable for Cell {}`
  - `breaker-game/src/walls/components.rs` → `impl Dmgable for Wall {}`
  - `breaker-game/src/breaker/components.rs` → `impl Dmgable for Breaker {}`
  - `breaker-game/src/cells/behaviors/survival/salvo/components.rs` → `impl Dmgable for Salvo {}`
- Delete `breaker-game/src/shared/death_pipeline/` entirely (directory, including plugin, systems, tests).
- Delete the `mod death_pipeline;` line from `breaker-game/src/shared/mod.rs`.
- Wire in `game.rs`:
  ```rust
  app.add_plugins(RantzDmgPlugin)
      .register_dmgable::<Bolt>()
      .register_dmgable::<Wall>()
      .register_dmgable::<Breaker>()
      .register_dmgable::<Salvo>()
      .configure_sets(FixedUpdate, EffectV3Systems::Tick.before(DmgSystems::ApplyDamage));
  ```
  `Cell` NOT registered this wave (see §Scope note).
- Rename callers of the old `DeathPipelineSystems` set enum to `DmgSystems` (game-side references in `cells/plugin/system.rs` `.before(DeathPipelineSystems::DetectDeaths)` → `.before(DmgSystems::EmitKill)`, etc.). Detail review on plan-wave execution.
- Migrate `handle_breaker_death` (state/run) from `DeathPipelineSystems::HandleKill` to `DmgSystems::ApplyKill`.
- Verify `GameEntity` trait is gone from game code — every reference now resolves to `Dmgable`.

### W2 — Migrate DamageBoost / Vulnerable callers

Three parallel sub-waves; all land together.

**W2a — effect_v3 core** (~15 sites):
- `effect_v3/effects/damage_boost/config.rs`: rewrite `Fireable::fire` to insert into `DamageBoostStack` (`stack.add(SourceId::from(source), config.multiplier)` for persistent; `stack.add_one_shot(multiplier)` for Pattern B consume-on-use).
- `effect_v3/effects/damage_boost/config.rs`: rewrite `Reversible::reverse` and `reverse_all_by_source` to call `stack.remove_by_source(&SourceId::from(source))`.
- `effect_v3/effects/vulnerable/config.rs`: mirror changes for `VulnerableStack`.
- DELETE the `PassiveEffect` impls on `DamageBoostConfig` + `VulnerableConfig` (the `EffectStack<T>` generic container stays — still used by `SpeedBoost`, `SizeBoost`, etc.).
- Update `effect_v3/commands/remove.rs` + `effect_v3/commands/route.rs` call sites.

**W2b — bolt domain** (~6 sites):
- `bolt/systems/bolt_cell_collision/system.rs` reads `DamageBoostStack` + `VulnerableStack` where it currently reads `EffectStack<DamageBoostConfig>` + `EffectStack<VulnerableConfig>`. (Note: emission still bakes in multipliers. TODO #2 switches to raw base_damage emission.)
- `bolt/queries.rs`: update the query alias that currently names `EffectStack<DamageBoostConfig>`.
- `bolt/test_utils.rs::damage_stack` signature swap.

**W2c — test helpers + state/run + protocol** (~10 sites):
- Every test helper that directly constructs `EffectStack<DamageBoostConfig>` for a bolt fixture uses `DamageBoostStack::default()` + `.add(...)`.
- State/run test fixtures (chip_select + protocol dispatch tests) update likewise.

RON asset files (`breaker-game/assets/chips/standard/*.ron` — 9 files + 1 protocol + 1 example) **stay untouched** — the `EffectType::DamageBoost(DamageBoostConfig)` enum variant is preserved; only its implementation changes.

### W3 — Heal emitter set-tag sweep

Systems that currently write `HealDealt<T>` from inside `DmgSystems::ApplyHeal` move to `DmgSystems::EmitHeal`:
- Momentum heal emitter
- Sympathy heal-adjacent emitter
- Renewal periodic heal (if wired this way)

Pure `.in_set(...)` rename. Cross-set `.chain()` in the crate already ensures `EmitHeal` runs before `ApplyHeal`, so timing is preserved.

### W4 — `source_chip` → `source: Option<SourceId>`

Every `DamageDealt<T> { source_chip: Some("..."), .. }` or `HealDealt<T> { source: Some("..."), .. }` emitter updates to `Some(SourceId::from("..."))`. Every reader updates to compare / clone / display an `Option<SourceId>`. Mechanical — covered by the compiler after the struct field changes in W1 (prelude re-export).

### W5 — `SourceIdExt` builder + namespace normalization

W4 swapped the field type but left every construction as raw string literals and ad-hoc `format!()` calls. Format drift is silent: a typo in a protocol sentinel compiles, consumer-side `starts_with("chip:")` or `strip_prefix("hazard:diffusion:")` matches can miss without the compiler noticing. W5 introduces a **game-side typestate builder** that makes the valid formats the ONLY way to construct a `SourceId`.

#### Location

`breaker-game/src/shared/source_id_ext.rs`. Trait `SourceIdExt` is impl'd on `rantzsoft_dmg::SourceId` (the crate stays game-vocabulary-free — all `ProtocolKind` / `HazardKind` / `Rarity` references live game-side).

#### Entry point + typestate

```rust
SourceId::builder()           // → SourceIdBuilder (empty typestate)
    .chip("Piercing")         // → ChipBuilder      { .rarity(Rarity) | .build() }
    .protocol(ProtocolKind)   // → ProtocolBuilder  { .action(&'static str) | .build() }
    .hazard(HazardKind)       // → HazardBuilder    { .instance(u64) | .build() }
    .armed(inner_source_id)   // → ArmedBuilder     { .build() }
```

Each kind-selector returns a distinct typestate struct. `ChipBuilder` has no `.instance()`; `ProtocolBuilder` has no `.rarity()`; invalid combinations don't compile. `.build()` is the only way to consume.

#### Format pins

| Builder chain | Output string |
|---|---|
| `.chip("Piercing").build()` | `chip:Piercing` |
| `.chip("Piercing").rarity(Rarity::Common).build()` | `chip:Piercing:Common` |
| `.protocol(ProtocolKind::Burnout).build()` | `protocol:burnout` |
| `.protocol(ProtocolKind::Burnout).action("shockwave").build()` | `protocol:burnout:shockwave` |
| `.hazard(HazardKind::Tether).build()` | `hazard:tether` |
| `.hazard(HazardKind::Diffusion).instance(42).build()` | `hazard:diffusion:42` |
| `.armed(inner).build()` | `<inner>:armed` |

`ProtocolKind::kind_slug()` and `HazardKind::kind_slug()` are simple `match` helpers producing lowercase-underscore slugs. They're the single source of truth for protocol/hazard segment strings.

**Armed has no trailing number.** Every production callsite today uses `#armed[0]` (see `effect_v3/conditions/evaluate_conditions/system.rs`); the index was never materially used. W5 collapses to `:armed`.

#### Reader-side helpers on `SourceId`

```rust
fn is_armed(&self) -> bool;                                      // ends with ":armed"
fn unwrap_armed(&self) -> Option<SourceId>;                      // strip trailing ":armed"
fn starts_with(&self, prefix: &str) -> bool;                     // partial-match attribution
fn extract_hazard_instance(&self, kind: HazardKind) -> Option<u64>;  // colocated with .instance()
```

`extract_hazard_instance` and the `hazard().instance()` construction live in the same module so the format lives in exactly one place — closes the "silent break if format changes" risk called out by the diffusion reader in `hazard/hazards/diffusion/system.rs:291`.

#### Callsite migration — delete

Remove these scattered constants and predicates entirely:

- Protocol sentinel constants: `BURNOUT_SENTINEL`, `BURNOUT_SHOCKWAVE_SOURCE`, `DEBT_COLLECTOR_SENTINEL`, `IRON_CURTAIN_SENTINEL`, `RECKLESS_DASH_SENTINEL`, `MOMENTUM_SENTINEL`, `SYMPATHY_SENTINEL` (in each protocol/hazard's `system.rs`).
- Hazard format constants: `DIFFUSION_SOURCE_PREFIX`, `DIFFUSION_SOURCE_PREFIX_COLON`.
- Inline literal strings at `echo_strike/system.rs:216`, `tether/system.rs:306`, `volatility/system.rs:166`, `renewal/system.rs:154`, `cascade/system.rs:145`.
- Armed string plumbing: `is_armed_source` + its module `effect_v3/conditions/armed_source.rs`; every `format!("{source}#armed[0]")` callsite in `effect_v3/conditions/evaluate_conditions/system.rs`.

#### Callsite migration — replace

| Pattern before | Pattern after |
|---|---|
| `SourceId::from(BURNOUT_SENTINEL)` | `SourceId::builder().protocol(ProtocolKind::Burnout).build()` |
| `SourceId::from(BURNOUT_SHOCKWAVE_SOURCE)` | `SourceId::builder().protocol(ProtocolKind::Burnout).action("shockwave").build()` |
| `SourceId::from("hazard:tether")` | `SourceId::builder().hazard(HazardKind::Tether).build()` |
| `SourceId::from(format!("{DIFFUSION_SOURCE_PREFIX}:{}", p.instance_id))` | `SourceId::builder().hazard(HazardKind::Diffusion).instance(p.instance_id).build()` |
| `s.0.strip_prefix(DIFFUSION_SOURCE_PREFIX_COLON).and_then(parse::<u64>)` | `s.extract_hazard_instance(HazardKind::Diffusion)` |
| `chip.and_then(\|c\| c.0.clone()).map(SourceId::from)` | builder via `.chip(template).rarity(rarity).build()` — requires `EffectSourceChip` shape change |
| `SourceId::from(source.to_owned())` (ad-hoc) | route through builder with an explicit kind |
| `stack.remove(source, self)` — `source: &str` | `stack.remove(&source_id, self)` — `source_id: &SourceId` |
| `format!("{source}#armed[0]")` | `SourceId::builder().armed(source).build()` |
| `is_armed_source(&source_string)` | `source_id.is_armed()` |
| `SpawnedByEvolution(pub String)` | `SpawnedByEvolution(pub SourceId)` — constructed via `.chip(name).rarity(Evolution).build()` |

#### Sub-component shape changes

- **`EffectSourceChip`** currently holds `Option<String>` (raw display name). Migrates to carry either `Option<SourceId>` directly (simpler) or a typed `Option<ChipTag { template: String, rarity: Option<Rarity> }>` (more explicit). Decide during implementation — both are downstream of the same callsite sweep.
- **`SpawnedByEvolution(String)`** → `SpawnedByEvolution(SourceId)`. Read site at `bolt/systems/bolt_cell_collision/system.rs:388` drops its `SourceId::from(s.0.clone())` wrapping and just clones the `SourceId`.
- **`EffectStack<T>`** generic key changes from `&str` to `&SourceId` across every impl. ~10 effect configs touched (`damage_boost`, `vulnerable`, `speed_boost`, `size_boost`, `quick_stop`, `bump_force`, `ramping_damage`, `piercing`, `anchor`, `splinter`). Each `Reversible::reverse` / `reverse_all_by_source` replaces `SourceId::from(source.to_owned())` with `&source_id` directly.

#### Tests

- **Builder typestate** (compile-time): `compile_fail` doctests proving `.chip(...).instance(...)` and `.protocol(...).rarity(...)` don't compile.
- **Format pins**: one test per row of the Format pins table above. Exact string match.
- **Round-trip**: `hazard(Diffusion).instance(42)` → `.extract_hazard_instance(Diffusion) == Some(42)`; `armed(x).unwrap_armed() == Some(x)`.
- **Partial match**: `SourceId::builder().chip("Piercing").rarity(Rarity::Common).build().starts_with("chip:Piercing")` → true; same chain vs `"chip:Pulse"` → false.
- **Callsite coverage**: every migrated site keeps its existing behavioral test; those tests now assert the exact SourceId shape the builder produces (captures format pins at the behavioral layer too).

#### Risks

1. **`EffectStack<T>` key type change** is the largest surface. Compiler-guided but wide; ~10 effect configs + consumer code. Bundle the change in a single commit per effect to keep reviews focused.
2. **Evolution chips**: `ChipDefinition` has `template_name: Option<String>` with `None` for `Rarity::Evolution`. The builder needs a decision — either `.chip(evolution_name).rarity(Rarity::Evolution).build()` where `evolution_name` is the evolution's own name (not a template), OR a dedicated `.evolution(name)` shorthand. Decide during implementation; either works, shorthand is marginally cleaner.
3. **Diffusion instance format atomicity**: construction (`hazard.instance()`) and extraction (`extract_hazard_instance`) must land together. If the format changes mid-migration, ring messages stop routing to their visited-cell sets and diffusion double-emits or drops silently.
4. **Armed-number drop**: collapsing `#armed[n]` to `:armed` assumes nothing ever reads `n`. Verified — production only uses `n=0`. If future multi-armed semantics appear, re-extend the builder (`armed(inner).index(n)`) without format-breaking existing callers.

### W6 — `EffectCommandsExt` callsite audit

`EffectCommandsExt` (in `breaker-game/src/effect_v3/commands/ext/system.rs`) defines 8 methods that each wrap `self.queue(<Command>)`: `fire_effect`, `reverse_effect`, `route_effect`, `stamp_effect`, `stage_effect`, `remove_effect`, `remove_staged_effect`, `track_armed_fire`. Five production callsites bypass the extension by queueing the raw `Command` struct directly. Ordering is identical (the extension is just `self.queue(...)` underneath) — this is a style/consistency sweep, but it's prerequisite for W7 because W7 needs an exhaustive inventory of every damage-emission-capable dispatch path, and direct-queue callsites hide from a grep on the extension.

#### Callsites to migrate

| Site | Current | After |
|---|---|---|
| `effect_v3/walking/fire.rs:19` | `commands.queue(FireEffectCommand { entity, effect, source })` | `commands.fire_effect(entity, effect, source)` |
| `effect_v3/walking/sequence.rs:32` | `commands.queue(FireEffectCommand { entity, effect, source })` | `commands.fire_effect(entity, effect, source)` |
| `effect_v3/walking/sequence.rs:39` | `commands.queue(RouteEffectCommand { entity, name, tree, route_type })` | `commands.route_effect(entity, name, tree, route_type)` |
| `effect_v3/effects/circuit_breaker/systems/system.rs:36` | `commands.queue(FireEffectCommand { ... })` | `commands.fire_effect(...)` |
| `effect_v3/effects/circuit_breaker/systems/system.rs:48` | `commands.queue(FireEffectCommand { ... })` | `commands.fire_effect(...)` |

#### Second pass — other command variants

After the 5 `FireEffectCommand` / `RouteEffectCommand` sites migrate, grep for each remaining variant and repeat:

```
grep -rn 'commands\.queue(ReverseEffectCommand' breaker-game/src
grep -rn 'commands\.queue(StampEffectCommand'   breaker-game/src
grep -rn 'commands\.queue(StageEffectCommand'   breaker-game/src
grep -rn 'commands\.queue(RemoveEffectCommand'  breaker-game/src
grep -rn 'commands\.queue(RemoveStagedEffectCommand' breaker-game/src
grep -rn 'commands\.queue(TrackArmedFireCommand' breaker-game/src
```

Any production hits (not test fixtures constructing the struct directly for assertion purposes) get the same treatment. If a site has a genuine ordering reason to bypass the extension, leave it and add a doc comment explaining why. Default action is to migrate.

#### Tests

No new behavioral tests — the extension methods are identical wrappers. The existing tests for the affected systems (`walking/sequence.rs`, `walking/fire.rs`, `effects/circuit_breaker/*`) continue to pass unchanged, which is the pin.

One regression safeguard: add a `compile_fail` or `clippy::disallowed_methods` lint entry that forbids `commands.queue(<EffectCommand>)` forms going forward. Drives new code onto the extension automatically.

#### Risks

1. **Hidden callsites**: the 5 surfaced here are what grep shows; secondary command variants may reveal more. Do the second-pass grep BEFORE declaring W6 complete.
2. **Test-only struct construction**: tests that build `FireEffectCommand { ... }` as fixtures for assertion (e.g. verifying queued-command content) stay as-is — those aren't dispatch bypasses, they're fixture construction.

### W7 — `Fireable::fire` damage-emission refactor

The surface problem: two `Fireable::fire` impls emit `DamageDealt<Cell>` by calling `world.write_message(DamageDealt { ... })` directly inside imperative Commands-based `fire()` bodies. Those bodies execute at command-queue flush time, which is not a scheduled system and has no set membership. Consequences are detailed in the "Per-wave implementation approach" section and in the session's W9 explanation — the short version is these messages bypass `DmgSystems::EmitDamage` ordering, can't be tagged `.in_set(EmitDamage)`, don't get seen by same-tick MutateDamage mutators, and keep the W6 (raw base_damage emission) bug alive for Explode and PiercingBeam.

**Prerequisite**: W6 lands first so the dispatch surface is uniform. After W6, every effect trigger path flows through `EffectCommandsExt::fire_effect(...)` (or the other helpers). W7 surveys every `Fireable::fire` impl reached via that path, identifies the ones that emit damage inline, and splits each into a request + scheduled consumer.

#### Current inline-damage emitters

Exhaustive grep of `Fireable::fire` bodies that call `world.write_message(DamageDealt ...)`:

| File | Impl | Current behavior |
|---|---|---|
| `effect_v3/effects/explode/config.rs:22-80` | `Fireable for ExplodeConfig` | `fire()` queries quadtree for cells in radius, computes per-cell damage (pre-multiplied?), writes one `DamageDealt<Cell>` per hit via `world.write_message` |
| `effect_v3/effects/piercing_beam/config/config_impl.rs:22-85` | `Fireable for PiercingBeamConfig` | `fire()` queries cells along beam line, writes `DamageDealt<Cell>` per pierced cell |

No other Fireable impl in `effect_v3/effects/*/config.rs` or `config/config_impl.rs` currently calls `write_message(DamageDealt ...)` inline (confirmed by `grep -l "write_message(DamageDealt" breaker-game/src/effect_v3/effects/*/config.rs breaker-game/src/effect_v3/effects/*/config/config_impl.rs`). Post-W6 the audit MUST be re-run because the consolidated `fire_effect`-path callsites may reveal more.

#### Refactor pattern (per emitter)

Split each inline emitter into three pieces:

1. **Request message** (game-side, per emitter):
   ```rust
   // effect_v3/effects/explode/messages.rs
   #[derive(Message, Debug, Clone)]
   pub struct ExplodeEmissionRequested {
       pub center:      Vec2,
       pub radius:      f32,
       pub base_damage: f32,
       pub dealer:      Option<Entity>,
       pub source:      Option<SourceId>,
   }
   ```

2. **`fire()` body** writes the request only:
   ```rust
   impl Fireable for ExplodeConfig {
       fn fire(&self, entity: Entity, source: &str, world: &mut World) {
           let Some((center, _)) = world.get::<Position2D>(entity).map(|p| (p.0, ())) else {
               return;
           };
           let source_id = /* builder via SourceIdExt */ ;
           world.resource_mut::<Messages<ExplodeEmissionRequested>>()
               .write(ExplodeEmissionRequested {
                   center,
                   radius:      self.radius,
                   base_damage: self.base_damage,
                   dealer:      Some(entity),
                   source:      Some(source_id),
               });
       }
   }
   ```
   No world queries beyond grabbing the emitter's position; no damage math; no multi-entity iteration.

3. **Consumer system** lives in `DmgSystems::EmitDamage`, reads the request, does the spatial query, writes `DamageDealt<Cell>` per hit with **raw `base_damage`** (W6 rule — pipeline multiplies exactly once):
   ```rust
   // effect_v3/effects/explode/systems/apply_explode.rs
   pub(crate) fn apply_explode_damage(
       mut reader: MessageReader<ExplodeEmissionRequested>,
       quadtree: Res<RantzQuadtree>,
       cells: Query<Entity, (With<Cell>, Without<Dead>)>,
       mut writer: MessageWriter<DamageDealt<Cell>>,
   ) {
       for req in reader.read() {
           for candidate in quadtree.query_circle(req.center, req.radius) {
               if cells.get(candidate).is_err() { continue; }
               writer.write(DamageDealt::<Cell> {
                   dealer:        req.dealer,
                   attributed_to: None,
                   target:        candidate,
                   amount:        req.base_damage,
                   source:        req.source.clone(),
                   _marker:       PhantomData,
               });
           }
       }
   }
   ```
   Registered: `app.add_systems(FixedUpdate, apply_explode_damage.in_set(DmgSystems::EmitDamage))` in the explode plugin / effect_v3 plugin aggregator.

Same shape for `PiercingBeamConfig`:
- `PiercingBeamEmissionRequested { start, direction, length, base_damage, piercing_remaining, dealer, source }`
- `fire()` writes the request only
- `apply_piercing_beam_damage` scheduled consumer does the beam raycast + per-cell emission with raw `base_damage`

#### Files touched per emitter

| Explode | PiercingBeam |
|---|---|
| `effect_v3/effects/explode/messages.rs` (new) | `effect_v3/effects/piercing_beam/messages.rs` (new) |
| `effect_v3/effects/explode/systems/apply_explode.rs` (new) | `effect_v3/effects/piercing_beam/systems/apply_piercing_beam.rs` (new) |
| `effect_v3/effects/explode/config.rs` (modify `fire()`) | `effect_v3/effects/piercing_beam/config/config_impl.rs` (modify `fire()`) |
| `effect_v3/effects/explode/plugin.rs` (register message + system) | `effect_v3/effects/piercing_beam/plugin.rs` (register message + system) |
| `effect_v3/effects/explode/tests.rs` (update fixtures) | `effect_v3/effects/piercing_beam/tests/*` (update fixtures) |

#### Tests per emitter

Behavioral pins (new or updated):

1. **Fire emits request only**: `fire()` writes exactly one `<Emitter>EmissionRequested` with the expected fields; NO `DamageDealt<Cell>` is emitted by `fire()` itself on the same tick.
2. **Consumer emits raw base_damage**: one tick later, `DamageDealt<Cell>` messages carry `amount == self.base_damage` (pre-multiplication). The pipeline then applies boosts/vuln.
3. **End-to-end damage match**: with no boosts/vuln, pre-refactor and post-refactor `Hp.current` on hit cells are identical (behavioral parity).
4. **Boost applied once**: with a single `DamageBoostStack` entry on the dealer (multiplier 2.0), the final `Hp` delta equals `base_damage * 2.0` — NOT `base_damage * 4.0`. This catches the pre-W6-style double-application that would re-appear if `fire()` pre-multiplied.
5. **Source propagation**: the `source: Some(SourceId)` carried by the request reaches every emitted `DamageDealt<Cell>` unchanged.
6. **Scheduling pin**: `system_in_set(app, apply_explode_damage, DmgSystems::EmitDamage) == true`. Same for piercing_beam.
7. **Invulnerable pass-through**: cells with `Invulnerable` get a message (filter zeroes it in ApplyDamage — consistent with W7).
8. **Consumer handles multi-request-per-tick**: two `fire()` calls on the same tick produce two requests → consumer reads both → emits both sets of damage. No dropped requests.

Regression:
- **Queue-flush vs schedule-edge timing**: write an integration test that arms a Diffusion cell as primary, triggers explode that also hits the same cell, and asserts the Diffusion reducer sees the combined damage. Pre-refactor this would likely fail (inline-emitted explode damage arrives after MutateDamage runs); post-refactor it succeeds. This is the tangible payoff of the refactor — pin it with a test.

#### Risks

1. **`&mut World` → `Commands`-friendly signature shift**: `Fireable::fire` takes `&mut World` currently. The refactor still needs to write one message; keep the `&mut World` signature and use `world.resource_mut::<Messages<T>>()` to write. Don't change the trait — just change what each body does.
2. **Post-W6 re-inventory**: after W6 unifies dispatch through `EffectCommandsExt`, grep once more for `write_message(DamageDealt`, `world.resource_mut::<Messages<DamageDealt<Cell>>>()`, and any new patterns that W6 normalization might surface. Add newly-discovered emitters to this wave.
3. **Fireable impls that emit damage via a BOOST stack, not DamageDealt directly**: these are fine — `DamageBoostStack::add_one_shot` etc. is the correct non-emitting path and doesn't need the request+consumer split. Don't accidentally refactor those.
4. **Scenario ordering changes**: test that `cargo scenario -- --all` still passes. Specifically the Diffusion/Tether interaction with Explode (if any scenarios exercise it). Behavioral parity is expected but the timing shift from "inline during fire flush" to "next FixedUpdate's EmitDamage set" could expose latent ordering assumptions in scenarios.

### W8 — Pre-merge cleanup sweep

Seven items. Two (§A, §B) are pre-existing scenario failures that block the Full Verification Tier (`cargo scenario -- --all` must be green at the pre-merge gate per branch policy). Five (§C–§G) are reviewer findings surfaced during W5–W7 that need to land before merge so the branch arrives clean. All seven are disjoint files → safe to parallel-launch.

#### §A — Fix `cell_death_speed_burst` scenario (pre-existing)

- **Scenario**: `breaker-scenario-runner/scenarios/chaos/cell_death_speed_burst.scenario.ron`.
- **Diagnosis (from session-state Active Failures)**: 1-tick ordering mismatch between `apply_velocity_formula` (bolt speed-clamp system) and `EffectV3Systems::Death` (where `on_cell_destroyed` death bridges fire effects like speed-burst). The speed-burst multiplier is applied to bolt velocity on the tick the cell dies; `apply_velocity_formula` either ran already that tick (losing the burst) or runs next tick (applying the burst one frame late).
- **Investigation steps before writing the fix**:
  1. Read `breaker-game/src/bolt/systems/apply_velocity_formula.rs` — confirm current set-membership + ordering edges.
  2. Read `breaker-game/src/effect_v3/effects/speed_boost/*` — find the death-bridge that arms the speed-burst.
  3. Read `docs/architecture/ordering.md` (post-cleanup — may be the task #25 target) for the `EffectV3Systems::Death` authoritative ordering claim.
  4. Run the scenario with `cargo scenario -- -s cell_death_speed_burst --no-fail-fast` to inspect the violation message (position/velocity/frame) — decide whether the bolt is getting speed-clamped BEFORE the burst or the burst is arriving a frame late.
- **Likely fix shape**: add an explicit `.before(BoltSystems::IntegrateMotion)` or `.after(EffectV3Systems::Death)` edge on whichever side is wrong. No new components, no new messages.
- **Tests**:
  - Unit test in bolt domain: spawn bolt at known velocity, trigger death-bridge-driven speed burst, tick once, assert velocity has been multiplied AND clamped within the same tick (not on the tick after).
  - Scenario regression: the existing `cell_death_speed_burst.scenario.ron` must pass after the fix.
- **Risks**: speed-burst effects are currently applied via `EffectStack<SpeedBoostConfig>` reconciliation; the ordering fix might uncover a second latent bug where the stack reconciliation itself races with clamp. If that's the case, scope it as a separate item and flag before proceeding.
- **Commit**: `fix(bolt): apply speed-burst multiplier before velocity-formula clamp on same tick`.

#### §B — Fix `evolution_lifecycle` scenario (pre-existing scenario-runner infra bug)

- **Scenario**: `breaker-scenario-runner/scenarios/mechanic/evolution_lifecycle.scenario.ron`.
- **Diagnosis (from session-state Active Failures)**: scenario-runner concurrency infrastructure bug — not a game-code bug. The scenario fails intermittently under parallel execution (`cargo scenario -- --all` default 32 parallel jobs) but passes under `--serial`. Likely a race in how scenario instances share state across subprocess runs, or a flaky assertion sensitive to wall-clock timing.
- **Investigation steps**:
  1. Run `cargo scenario -- -s evolution_lifecycle` (single, in-process) 10× — confirm serial passes consistently.
  2. Run `cargo scenario -- --all --serial | grep evolution_lifecycle` — confirm pass under forced-serial mode.
  3. Read `breaker-scenario-runner/src/` for the parallel-execution driver — look for shared mutable state (static `Mutex`, global `HashMap`, etc.) that instances might fight over.
  4. Read the scenario's invariant checkers — look for assertions that depend on absolute wall time rather than simulation tick count.
- **Likely fix shape**: either (a) fix the scenario-runner parallel isolation (add per-instance workspace, use atomic counters, etc.), or (b) fix the scenario's assertion to be time-independent. Determine which during investigation.
- **Tests**: the scenario itself under `cargo scenario -- --all -p 32 --loop 10` should pass 100% of runs. No unit tests needed unless the fix is in scenario-runner infra (then add unit tests to `breaker-scenario-runner/src/`).
- **Risks**: this is scenario-runner code, not game code — bugs here can silently accept broken game behavior in future scenarios. If the fix is non-trivial, escalate to a standalone branch instead of bundling with this port merge.
- **Commit**: either `fix(scenario-runner): <specific infra issue>` or `fix(scenarios): make evolution_lifecycle assertions deterministic`.

#### §C — `debt_collector` zero-amount emission guard

- **Source**: reviewer-correctness finding during W6 execution. `debt_collector_on_impact` in `breaker-game/src/protocol/protocols/debt_collector/system.rs:222` writes `DamageDealt<Cell>` with `amount = /* some calculation */` — but there's no guard against `amount <= 0.0`. A zero-amount message propagates through the full pipeline and can cause a spurious `DamageDealt::<Cell>` record in tests/stats tracking.
- **Fix**:
  ```rust
  // Before writing the message:
  if amount <= 0.0 {
      continue; // or `return;` if this is a single-iteration path
  }
  damage_writer.write(DamageDealt { ... });
  ```
  Matches the guard already present in `tether_emit_partner` (see `hazard/hazards/tether/system.rs`) — use that as the reference.
- **Files**:
  - `breaker-game/src/protocol/protocols/debt_collector/system.rs` — add the guard.
  - `breaker-game/src/protocol/protocols/debt_collector/tests/` — add a test.
- **Tests**:
  - `debt_collector_skips_emission_when_amount_is_zero`: seed conditions that produce a zero amount (e.g. debt amount of 0, or a fully-paid target), tick, assert no `DamageDealt<Cell>` message emitted.
  - `debt_collector_skips_emission_when_amount_is_negative`: same shape, negative amount.
  - Existing positive-amount test continues to pass — pin.
- **Risks**: minor — the guard is subtractive. The one gotcha is making sure the zero-amount case actually CAN occur in the codebase today (otherwise the guard is defensive-only); document the callsite that can produce it in a comment.
- **Commit**: `fix(debt_collector): guard against zero-amount damage emissions`.

#### §D — `Hit` → `Impact` rename in `bolt_cell_collision` types

- **Source**: reviewer-quality finding during W6. `bolt_cell_collision` uses `Hit` / `HitOutcome` / `HitApplyState` / `resolve_bolt_cell_hit` naming. Game vocabulary per `docs/design/terminology/` uses **Impact** (or **Bump** depending on context) — `Hit` is not a canonical term. Also conflicts with the existing `BoltImpactCell` / `BoltImpactWall` message names.
- **Scope**: rename-only; no behavioral change.
- **Files**:
  - `breaker-game/src/bolt/systems/bolt_cell_collision/system.rs` — rename `HitOutcome` → `ImpactOutcome`, `HitApplyState` → `ImpactApplyState`, `resolve_bolt_cell_hit` → `resolve_bolt_cell_impact`, local `hit` bindings → `impact`.
  - `breaker-game/src/bolt/systems/bolt_cell_collision/tests/*.rs` — same rename across fixtures.
  - Any doc comments referencing the old names — sweep.
- **Tests**: existing tests pass unchanged (pure rename). No new tests needed.
- **Risks**: name collision with `BoltImpactCell` — be careful not to shadow. The message type is cross-domain (`effect_v3` consumes it); the internal `ImpactOutcome` type is crate-internal to `bolt_cell_collision`. Use module-qualified references (`collision::ImpactOutcome`) where needed.
- **Commit**: `refactor(bolt): rename Hit→Impact in bolt_cell_collision types for vocabulary consistency`.

#### §E — Extract protocol scheduling test helpers to shared module

- **Source**: reviewer-quality finding during W6. 6 protocols have scheduling tests (`burnout`, `debt_collector`, `echo_strike`, `iron_curtain`, `reckless_dash`, plus one more) that each reimplement the same `scheduling_test_app()` + cell-spawn + bolt-spawn + protocol-activation helpers. Duplication is ~200 lines per file × 6 files.
- **Fix**: extract the common helpers to `breaker-game/src/protocol/test_helpers/scheduling.rs` (new file) as `pub(crate) fn`s. Each protocol's `tests/scheduling.rs` imports from there.
- **Files**:
  - `breaker-game/src/protocol/test_helpers/mod.rs` — new, `#[cfg(test)] pub(crate) mod scheduling;`.
  - `breaker-game/src/protocol/test_helpers/scheduling.rs` — new, holds `scheduling_test_app()`, `spawn_test_cell()`, `spawn_test_bolt_with_base_damage()`, `activate_protocol()`, `tick_n()`, etc.
  - 6 protocol `tests/scheduling.rs` files — remove duplicated helpers, `use crate::protocol::test_helpers::scheduling::*`.
  - `breaker-game/src/protocol/mod.rs` — add `#[cfg(test)] mod test_helpers;` declaration.
- **Tests**: all 6 protocol scheduling test suites continue to pass unchanged.
- **Risks**: cross-file refactor — compiler-guided. One specific risk: each protocol's test file may have helper variants with subtly different parameters (e.g. one spawns a cell with Hp=100, another with Hp=50). Parameterize the shared helper to accept an `Hp` argument rather than having a default — avoid behavioral drift.
- **Commit**: `refactor(protocol): extract shared scheduling test helpers`.

#### §F — Add `echo_strike` HP-delta scheduling test

- **Source**: reviewer-completeness finding during W5. Five protocols (`burnout`, `debt_collector`, `iron_curtain`, `reckless_dash`, plus one more) have scheduling tests asserting end-to-end Hp delta under the full pipeline (Harness B). `echo_strike` has scheduling tests that only assert message emission — not the resulting Hp change. Gap: a future regression that breaks the sibling-emission amount-pass-through could land without catching it in echo_strike's tests.
- **Fix**: add one scheduling test to `breaker-game/src/protocol/protocols/echo_strike/tests/scheduling.rs`:
  ```rust
  #[test]
  fn echo_strike_sibling_applies_expected_hp_delta_end_to_end() {
      let mut app = echo_strike_scheduling_app();          // full pipeline
      let primary = spawn_vuln_cell_with_hp(&mut app, Vec2::ZERO, 100.0);
      let echo_a  = spawn_vuln_cell_with_hp(&mut app, Vec2::new(60.0, 0.0), 100.0);
      let echo_b  = spawn_vuln_cell_with_hp(&mut app, Vec2::new(-60.0, 0.0), 100.0);
      // Arm echo_strike, register network
      // Write primary DamageDealt<Cell> amount=20 targeting primary
      // Tick the full pipeline
      // Assert primary.Hp = 80, echo_a.Hp = 90 (half-fraction), echo_b.Hp = 90
      // (exact numbers depend on echo_strike's fraction config)
  }
  ```
  Exact expected values derive from `EchoStrikeConfig` — use the canonical tuning.
- **Files**:
  - `breaker-game/src/protocol/protocols/echo_strike/tests/scheduling.rs` — add the test.
  - If §E lands first, the helpers come from the shared module. If §F lands first, inline the helpers here and the §E cleanup will deduplicate.
- **Tests**: the new test passes.
- **Risks**: low — additive test coverage.
- **Commit**: `test(echo_strike): add end-to-end Hp-delta scheduling pin`.

#### §G — Rename placeholder `group_*.rs` test files

- **Source**: reviewer-file-length finding during the 2026-04-23 file-split pass. 36 test files across the codebase are named `group_a.rs`, `group_b.rs`, `group_c.rs`, etc. — placeholders from sub-split operations that never got renamed to describe their actual behavior content.
- **Fix**: for each file, read the test function names, identify the behavioral grouping, and rename the file to match. Example: a file full of `*_boundary_*` test names becomes `boundary_edges.rs`. Update the containing `mod.rs` to reflect the rename.
- **Inventory** (to be generated at the start of W8 §G; expected ~36 files):
  ```
  find breaker-game/src -name 'group_*.rs' -type f
  find rantzsoft_dmg/src -name 'group_*.rs' -type f
  ```
- **Files** (one commit per sub-directory of files to keep commits focused):
  - Each sub-directory with `group_*.rs` files gets its own commit.
  - Each commit: rename files + update `mod.rs` declarations.
- **Tests**: all tests continue to pass (pure rename; no content changes).
- **Risks**: low. One watch-out: if two sibling files would rename to the same behavioral name, merge them OR sub-split differently and name both descriptively.
- **Commit**: one per sub-directory, `refactor(tests): rename group_*.rs placeholders in <directory> to behavioral names`.

#### W8 parallel launch strategy

All seven items touch disjoint files. Launch writer-code in parallel per item (respecting the cargo serialization — each runner-tests invocation after each writer-code completes waits for the prior to release). Ordering preferences:

- **§A and §B first** (blocking Full Verification Tier — no other work matters until these are green).
- **§C, §D, §F in parallel** (small, localized).
- **§E, §G last** (wider file touch; better to land after the smaller items have settled).

Each item gets its own commit to keep reviewable chunks. After all 7 land: Full Verification Tier runs, then merge.

## Per-wave implementation approach

All waves use the TDD pipeline (writer-tests → reviewer-tests → RED gate → writer-code → GREEN gate → reviewer-correctness → commit). Each wave's Basic Verification Tier must be clean before commit; Standard tier at commit gate; Full tier at branch finish.

W1 is mostly mechanical (file move + re-exports + `impl` additions + enum rename). Spec a small RED suite confirming the crate is actually consumed and the old types are gone.

W2 is the most complex — four test sub-suites (one per sub-wave + a cross-migration integration).

W3 and W4 are narrow.

## Verification

### Per-wave Basic Tier
- `cargo all-dclippy` clean
- `cargo all-dtest` clean
- `cargo dmgtest` clean (the crate itself must still pass; we aren't modifying it)

### Standard Tier (commit gate)
- `reviewer-correctness` on the diff
- `reviewer-architecture` on W1 (boundary — no game code leaks into crate)
- `reviewer-completeness` on every wave

### Full Tier (pre-merge)
- `cargo scenario -- --all` — every scenario still passes
- `guard-game-design` — damage numbers identical before/after (no mechanic behavior change)
- `guard-docs` — TODO index updated
- `guard-dependencies` — `breaker-game` now depends on `rantzsoft_dmg`

### Spot-checks
- `grep -rn 'death_pipeline\|GameEntity\|DeathPipelineSystems' breaker-game/` → zero matches
- `grep -rn 'EffectStack<DamageBoostConfig>\|EffectStack<VulnerableConfig>' breaker-game/` → zero matches
- `cargo dev` boots; short playthrough damage numbers unchanged

## Risks

1. **Cell-on-legacy-path interim.** During W1–W4, Cell damage still goes through `apply_damage_to_cells` (the legacy cells-domain system), not through crate-owned `apply_damage::<Cell>`. `DamageBoostStack` on the bolt IS used by `bolt_cell_collision` to pre-bake the amount before emitting. `VulnerableStack` on the cell IS read by `apply_damage_to_cells`. No double-application risk as long as Cell stays unregistered. TODO #2 closes the interim.
2. **W1 delete-then-replace ordering.** `breaker-game/src/shared/death_pipeline/` deletion and `rantzsoft_dmg` prelude re-export must land in the same commit/PR; interim would fail to compile. W1 tests drive the atomic swap.
3. **`source_chip` → `source` field rename** is a breaking change across many files. Compiler-guided; mechanical.
4. **Heal emitter set rename** is trivial but must not break scenario tests (Sympathy regression suite specifically).

## Reference files (game-side targets)

- Prelude: `breaker-game/src/prelude/death_pipeline.rs`
- Components needing `impl Dmgable`: `breaker-game/src/{bolt, cells, walls, breaker}/components.rs`, `breaker-game/src/cells/behaviors/survival/salvo/components.rs`
- Game-plugin composition: `breaker-game/src/game.rs`
- W2 call sites: `breaker-game/src/effect_v3/effects/{damage_boost, vulnerable}/config.rs`, `effect_v3/commands/{remove, route}.rs`, `effect_v3/dispatch/*.rs`, `effect_v3/walking/*.rs`, `effect_v3/types/effect_type.rs`, `effect_v3/types/reversible_effect_type.rs`, `effect_v3/stacking/effect_stack/*.rs`, `bolt/systems/bolt_cell_collision/system.rs`, `bolt/queries.rs`, `bolt/test_utils.rs`, `state/run/chip_select/systems/*/tests.rs`, `protocol/systems/dispatch_protocol_selection/tests.rs`
- W3 call sites: whichever systems currently write `HealDealt<T>` from `ApplyHeal` — inventory during W3 planning

## TODO entry

```
1. **[BLOCKED by #0]** Port `breaker-game` to `rantzsoft_dmg` — register Bolt/Wall/Breaker/Salvo,
   delete `shared/death_pipeline/`, migrate `EffectStack<DamageBoostConfig>` /
   `EffectStack<VulnerableConfig>` callers to new stacks, heal emitter sweep, `source_chip`
   → `source: Option<SourceId>` rename. Cell stays on legacy path; TODO #2 migrates it.
   — [detail](detail/port-to-rantzsoft-dmg.md)
```
