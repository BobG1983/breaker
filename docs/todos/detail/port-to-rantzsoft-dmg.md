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
