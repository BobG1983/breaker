# Creating a Mutator (Protocol or Hazard)

A **mutator** is a run-scoped game modifier — either a positive selectable upgrade (**protocol**, chosen at chip-select) or a stacking challenge (**hazard**, chosen during infinite play at tier 9+). Both share structural patterns and live under `breaker-game/src/mutators/`.

This how-to walks through adding a new mutator end-to-end. Use it whenever you're adding a new protocol or hazard. Examples reference existing mechanics — read those alongside.

---

## 1. Pick a directory

Decide whether the mechanic is a **protocol** or a **hazard**:

- **Protocol** — positive, selectable at chip-select, typically one-shot per run. Mostly visible-to-player upgrades. Examples: `burnout`, `debt_collector`, `iron_curtain`.
- **Hazard** — negative, stackable, chosen during infinite-tier play. Difficulty knob. Examples: `diffusion`, `momentum`, `tether`, `volatility`.

Create the directory at `mutators/protocols/<mechanic>/` or `mutators/hazards/<mechanic>/`.

---

## 2. Lay out the mechanic directory

Every mechanic has the same structural skeleton:

```
<mechanic>/
├── mod.rs              # `pub fn wire(app: &mut App)` + module declarations
├── config/             # (or `config.rs`) — RON-loaded tuning + defaults
│   ├── mod.rs
│   └── config_impl.rs  # struct + Default impl + RON drift-guard test
├── messages.rs         # mechanic-specific messages (if any)
├── system.rs           # (or `system/`) — production code: `wire(app)`, the systems themselves, activate(), helper fns
└── tests/              # (only if 400+ test lines) — split by behavior into named files
```

Use the directory form (`config/`, `system/`, `tests/`) once a single file exceeds 400 lines (see `.claude/rules/file-splitting.md`). Until then, prefer single files.

---

## 3. Add the kind variant

Add the mechanic's variant to the appropriate `Kind` enum:

- Protocols → `mutators/protocols/definition.rs::ProtocolKind`
- Hazards → `mutators/hazards/definition.rs::HazardKind`

Also add a `Tuning` variant in the same file (`ProtocolTuning::<NewMech> { ... }` or `HazardTuning::<NewMech> { ... }`). The tuning fields mirror the RON-loaded shape from step 5.

---

## 4. Implement `wire(app: &mut App)`

`wire` is the per-mechanic registration entry point. The fan-out (`mutators/protocols/mod.rs::wire(app)` / `mutators/hazards/mod.rs::wire(app)`) calls every mechanic's `wire(app)` exactly once at plugin build time.

```rust
pub(crate) fn wire(app: &mut App) {
    // Resources owned by this mechanic
    app.init_resource::<MyConfig>();
    // Systems this mechanic owns
    app.add_systems(FixedUpdate, my_system.run_if(protocol_active(ProtocolKind::MyMech)));
    // Cleanup on node end (per-mechanic responsibility — there is no central cleanup driver)
    app.add_systems(OnExit(NodeState::Playing), my_cleanup_node);
}
```

Then add `pub mod <mechanic>;` to `mutators/protocols/mod.rs` (or `hazards/mod.rs`) and call `<mechanic>::wire(app)` from the parent `wire(app)` fan-out.

---

## 5. Author the RON config

Place the RON file under `breaker-game/assets/`:

- Protocols → `assets/protocols/<mechanic>.ron`
- Hazards → `assets/hazards/<mechanic>.ron`

The fields must match the `ProtocolTuning::<NewMech>` / `HazardTuning::<NewMech>` variant. The mechanic's `activate(tuning, commands)` function (in `system.rs`) parses the tuning enum and inserts a `<MechName>Config` resource with derived parameters.

---

## 6. Activate the mechanic

`activate(tuning: &HazardTuning, commands: &mut Commands)` (or the protocol variant) is the bridge from "selected at chip-select / hazard-select" to "running this run." It pattern-matches on the kind-specific tuning variant, derives the working config, and inserts it as a `Resource`.

```rust
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::MyMech { foo, bar } = *tuning else {
        warn!("MyMech::activate received non-MyMech tuning");
        return;
    };
    commands.insert_resource(MyMechConfig { foo: foo * 100.0, bar });
}
```

Wire `activate` into the `dispatch_<protocol|hazard>_selection` system in `mutators/<sub>/systems/dispatch_<sub>_selection.rs`.

---

## 7. Run-condition gating

Use `protocol_active(ProtocolKind::MyMech)` / `hazard_active(HazardKind::MyMech)` as `run_if` guards on systems that should only fire when the mechanic is active. Both run-conditions live in `mutators/<sub>/resources.rs`.

```rust
app.add_systems(
    FixedUpdate,
    my_system
        .run_if(protocol_active(ProtocolKind::MyMech))
        .run_if(in_state(NodeState::Playing)),
);
```

---

## 8. Damage chain participation

If the mechanic emits, mutates, or reacts to `DamageDealt<Cell>` in a cross-mechanic-ordered way, **register it centrally** in `mutators/plugin/system.rs::wire_damage_chain`, NOT in the mechanic's `wire(app)`.

`wire_damage_chain` is the single source of truth for the ordering of:

- `DmgSystems::MutateDamage` participants (e.g., `diffusion_reduce_primary`)
- `DmgSystems::PostApplyDamage` participants chained as `diffusion_emit_rings → tether_emit_partner → echo_strike_emit_siblings`

Adding a new chain participant means **editing `wire_damage_chain`**. Make the system `pub(crate)` from your mechanic's `system.rs`/`mod.rs` so the plugin module can reach it.

---

## 9. `DamageDealt<T>` writers MUST live in `DmgSystems::EmitDamage`

Any game-side system that writes `DamageDealt<T>` messages directly MUST be tagged `.in_set(DmgSystems::EmitDamage)`, except in two specific cases:

- **Inside `EffectV3Systems::Tick`** — these are transitively `.before(DmgSystems::EmitDamage)` via the set-level edge configured in `EffectV3Plugin::build`. Tagging them in `EmitDamage` creates a cycle. Don't. Examples: `apply_pulse_damage`, `apply_chain_lightning_damage`, `apply_tether_damage`, `apply_shockwave_damage`, `apply_explode_damage`, `apply_piercing_beam_damage`.
- **`bolt_cell_collision` itself** — `BoltSystems::CellCollision` is transitively before `EffectV3Systems::Bridge` (via sibling `bolt_wall_collision` / `bolt_breaker_collision` ordering), so tagging `bolt_cell_collision` in `EmitDamage` creates a cycle. The transitive chain `CellCollision → Bridge → Tick → EmitDamage` is sufficient.

For everything else (protocol amplifiers, hazard impact emitters, salvo turret fire), tag with `.in_set(DmgSystems::EmitDamage)` and drop any `.before(EffectV3Systems::Bridge)` edges — they're now redundant.

---

## 10. Tests

Tests live in `<mechanic>/tests/`. Group by behavior, not alphabetically. Common test files:

- `activate.rs` — `activate(tuning, commands)` config-parsing tests
- `wire.rs` (or `wire_tests.rs`) — `wire(app)` schedule placement + run-if gating
- `cleanup_node.rs` — `OnExit(NodeState::Playing)` cleanup behavior
- `<system_name>.rs` — per-system behavioral pins
- `ron_asset.rs` — drift-guard test that the canonical config matches the RON file
- `helpers.rs` — shared test fixtures with `pub(super)` visibility

For chain-participating mechanics: tests that need the central chain wired must call both `<mechanic>::wire(&mut app)` AND `wire_damage_chain(&mut app)`. The fn lives at `mutators::plugin::wire_damage_chain` via a `pub(crate)` re-export gated `#[cfg(test)]` in `mutators/plugin/mod.rs`. Import it with:

```rust
use crate::mutators::plugin::wire_damage_chain;
```

The cross-mechanic ordering pins live centrally in `mutators/plugin/tests/damage_chain.rs` — don't duplicate them in per-mechanic test files.

---

## Reference mechanics by pattern

| Pattern | Reference |
|---------|-----------|
| Simple amplifier (reads + amplifies `DamageDealt<Cell>`) | `protocols/burnout`, `protocols/reckless_dash` |
| One-shot bump-triggered effect | `protocols/echo_strike`, `protocols/conductor` |
| State-mutating overlay (writes `Hp.max` etc.) | `hazards/momentum`, `hazards/volatility` |
| BFS ripple emitter (chain participant) | `hazards/diffusion` |
| Linked-pair partner emitter (chain participant) | `hazards/tether` |
| Sibling-emit on death | `protocols/echo_strike`, `hazards/sympathy` |
| Run-end resource swap | `protocols/tier_regression` |

When in doubt, copy the closest pattern and adjust.
