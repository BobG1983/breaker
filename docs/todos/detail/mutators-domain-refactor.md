# Consolidate `hazard/` and `protocol/` into a single `mutators/` domain

## Problem

Today's codebase has two parallel top-level domains — `breaker-game/src/hazard/` and `breaker-game/src/protocol/` — with near-identical shapes (per-mechanic directories, registries, RON tuning, activation flows, cleanup patterns, per-mechanic tests). The structural duplication is visible but tolerable; the real friction appears when systems from both domains need to participate in a shared chain (the `MutateDamage` damage-mutator chain introduced by the unified death pipeline crate). Chain ordering is cross-plugin, which pushes ordering wiring into whichever plugin "owns" the pipeline — a boundary violation.

This remediation consolidates both domains into a single `mutators/` domain with subdirectories that preserve the protocol/hazard semantic distinction, owned by a single `MutatorsPlugin`. The word "mutators" captures what both do: they mutate the rules of the run.

## Prerequisite

Depends on [TODO #1 — unified death pipeline crate](./unified-death-crate.md) landing first. The damage-mutator chain, `DamageBoostStack`/`VulnerableStack` components, and `SourceId` type all come from `rantzsoft_dmg`; this refactor assumes they're in place so `mutators/plugin.rs` can import them.

During the interim (crate landed, mutators refactor not yet done), the existing `hazard/` and `protocol/` plugins register chain members directly. Tests continue to pass; the architectural boundary is violated temporarily. Acceptable for the interim — this remediation closes it.

## Design

### File structure

```
breaker-game/src/mutators/
  mod.rs                      // pub mod plugin; pub mod protocols; pub mod hazards;
  plugin.rs                   // MutatorsPlugin — the ONLY plugin for this domain
  protocols/
    mod.rs                    // Re-exports per-protocol modules; registry + definition + resources
    registry.rs               // ProtocolRegistry (unchanged shape)
    definition.rs             // ProtocolDefinition, ProtocolKind, ProtocolTuning (unchanged)
    resources.rs              // ActiveProtocols (unchanged)
    systems/                  // dispatch_protocol_selection, generate_protocol_offering (unchanged)
    anchor/                   // each protocol unchanged internally
      mod.rs                  // pub fn wire(app: &mut App); pub use system::*;
      system.rs               // Systems — ONE system fn per file ideally
      tests/
        cleanup.rs            // per-mechanic cleanup test (moved from central suite)
        ...
    burnout/
    debt_collector/
    deadline/
    echo_strike/
    fission/
    iron_curtain/
    kickstart/
    reckless_dash/
    ricochet/
    tier_regression/
    ... (15 total)
  hazards/
    mod.rs                    // Re-exports per-hazard modules; registry + definition + resources
    registry.rs               // HazardRegistry (unchanged shape)
    definition.rs             // HazardDefinition, HazardKind, HazardTuning (unchanged)
    resources.rs              // ActiveHazards (unchanged)
    cascade/                  // each hazard unchanged internally
    diffusion/
    drift/
    echo_cells/
    erosion/
    fracture/
    gravity_surge/
    haste/
    momentum/
    overcharge/
    renewal/
    resonance/
    sympathy/
    tether/
    volatility/
    ... (16 total)
```

### `MutatorsPlugin` — the single plugin

```rust
// breaker-game/src/mutators/plugin.rs
pub struct MutatorsPlugin;

impl Plugin for MutatorsPlugin {
    fn build(&self, app: &mut App) {
        wire_protocols(app);        // Registry + definition + activation + per-protocol wire() calls
        wire_hazards(app);          // Registry + definition + tier progression + per-hazard wire() calls
        wire_damage_chain(app);     // Central chain tuple — source of truth for MutateDamage ordering
        wire_cleanup(app);          // Run-end cleanup driver for both subtrees
    }
}
```

No `ProtocolsPlugin`, no `HazardsPlugin`. `MutatorsPlugin::build` calls organizing sub-functions; all plugin work is done by this one plugin.

### Per-mechanic `wire(app)` exports non-chain systems only

Each mechanic exports:
- **System function(s)** as pub fns (not wired) — ready to be imported by the central chain assembly.
- **`pub fn wire(app: &mut App)`** that registers:
  - Activation hooks (OnEnter state, message reader for activation)
  - Timer ticks, stack managers, reactors (non-chain systems)
  - Cleanup (OnExit, message reader for run-end)
  - UI message wiring (if the mechanic emits a "show notification" message)
- **Does NOT register chain-participating systems.** Those are registered centrally in `wire_damage_chain`.

Example — Echo Strike:

```rust
// mutators/protocols/echo_strike/mod.rs
pub mod system;
pub use system::{echo_strike_emit_siblings, echo_strike_on_node_start, echo_strike_tick_primed};

pub fn wire(app: &mut App) {
    // Activation + timers — NOT the chain member.
    app.add_systems(FixedUpdate, (
        echo_strike_on_node_start,
        echo_strike_tick_primed,
    ).run_if(in_state(NodeState::Playing)));
    // Cleanup
    app.add_systems(OnExit(RunState::Finished), echo_strike_cleanup);
    // echo_strike_emit_siblings is registered in wire_damage_chain — NOT here.
}
```

### Central chain assembly — single source of truth

```rust
// mutators/plugin.rs
use crate::mutators::protocols::echo_strike::echo_strike_emit_siblings;
use crate::mutators::hazards::diffusion::diffusion_mutate_damage;
use crate::mutators::hazards::tether::tether_redirect;

fn wire_damage_chain(app: &mut App) {
    app.add_systems(FixedUpdate, (
        echo_strike_emit_siblings,
        diffusion_mutate_damage,
        tether_redirect,
        // ← new mutators added here; ordering decision lives in one place
    ).chain().in_set(DeathPipelineSystems::MutateDamage));
}
```

### Ordering rationale (game-side MutateDamage chain)

The crate (TODO #1) already chains the broader pipeline:

```
EmitDamage → ApplyDamageBoosts → MutateDamage → ApplyVulnerable → ApplyDamage → ...
```

`ApplyDamageBoosts` (crate-owned `apply_damage_boosts::<T>`) runs BEFORE `MutateDamage`, so every game-side mutator sees bolt-side amplified `msg.amount`. `ApplyVulnerable` (crate-owned `apply_vulnerable::<T>`) runs AFTER `MutateDamage` and BEFORE `ApplyDamage`, so every message — primary AND siblings emitted by the chain — gets its target's vulnerability applied before the pure applier deducts HP. `invulnerable_filter` (crate-owned) runs in `ApplyDamage` right before `apply_damage::<T>` and zeros messages targeting Invulnerable entities.

Within the game-side `MutateDamage` chain, the order of the three current members:

1. **`echo_strike_emit_siblings`** runs FIRST. It reads the amplified primary `DamageDealt<Cell>` and emits sentinel-tagged sibling `DamageDealt<Cell>` messages for each echo-network member (fraction-scaled). Running first means Echo Strike sees amplified damage for its fraction calc, AND its emitted siblings pass through Diffusion + Tether downstream (so an echoed cell that's in a Diffusion ring diffuses further; an echoed cell with a TetherLink redirects; an echoed cell that's Invulnerable gets zeroed by `invulnerable_filter` in `ApplyDamage`).

2. **`diffusion_mutate_damage`** runs SECOND. Reduces the primary message's `amount` and emits ring-sibling `DamageDealt<Cell>` messages for adjacent cells. Running after Echo Strike means Diffusion sees echo-emitted siblings and treats each as its own primary (each echoed cell's damage diffuses through its own ring). Running before Tether means Diffusion's ring siblings pass through Tether — a ring cell with a TetherLink redirects.

3. **`tether_redirect`** runs THIRD (last in `MutateDamage`). Reads each non-sentinel `DamageDealt<Cell>` whose target has a `TetherLink` and emits a sentinel-tagged sibling for the partner. Running last means Tether processes every message the chain has produced (primaries, Echo Strike siblings, Diffusion ring emissions). `tether_redirect` itself emits siblings but those don't get re-processed by upstream mutators — they flow straight to `ApplyVulnerable` → `ApplyDamage` next (where invulnerable zeroing still applies).

**Rules of thumb for future additions to the chain:**

- **Sibling emitters** (like Echo Strike) run EARLY so their emissions pass through downstream mutators that modify or redirect.
- **Primary mutators** (like Diffusion reducing the primary amount) run AFTER sibling emitters so siblings see the unreduced amount.
- **Pure redirect emitters** (like Tether) run LATE so they pick up siblings from upstream mutators.
- **Policy filters that zero amounts** (like `invulnerable_filter`) live in `ApplyDamage`, NOT `MutateDamage`, so they catch everything the chain produced.

When adding a new chain member, document in the PR description:
1. Which category it fits (emitter, primary mutator, redirect, policy).
2. Why it goes in the specific position in the tuple.
3. What it sees from upstream and what it emits to downstream.

### How authors add a new chain-participating protocol or hazard

1. Write the system fn in the mechanic's `system.rs`.
2. Export it from the mechanic's `mod.rs`.
3. Do NOT register it in the mechanic's `wire(app)`. Keep `wire(app)` for non-chain systems only.
4. Open `mutators/plugin.rs`, import the system, insert it at the correct position in the `wire_damage_chain` tuple.
5. Document the ordering decision (see rules of thumb above) in the PR description and in the mechanic's design doc §Ordering section.

The ordering decision is a design decision, not a clerical one — it MUST be central. The file is small, read-once, discoverable.

### `wire_protocols` / `wire_hazards`

Each calls its per-mechanic `wire(app)` functions:

```rust
fn wire_protocols(app: &mut App) {
    app.init_resource::<ProtocolRegistry>();
    app.init_resource::<ActiveProtocols>();
    app.add_message::<ProtocolActivated>();
    app.add_systems(FixedUpdate, (
        dispatch_protocol_selection,
        generate_protocol_offering,
    ));

    // Per-protocol wire() calls — registration of activation, timers, cleanup for each
    crate::mutators::protocols::anchor::wire(app);
    crate::mutators::protocols::burnout::wire(app);
    crate::mutators::protocols::debt_collector::wire(app);
    // ... all 15
}
```

Same shape for `wire_hazards` with `HazardRegistry`, `ActiveHazards`, `HazardActivated`, and all 16 hazard `wire(app)` calls.

### Separation maintained

Per-kind distinctions remain:
- `ProtocolRegistry` and `HazardRegistry` stay as separate resources
- `ActiveProtocols` and `ActiveHazards` stay as separate resources
- `ProtocolActivated` and `HazardActivated` stay as separate messages (always — per explicit user decision)
- `ProtocolTuning` and `HazardTuning` stay as separate enums (different RON structures)
- Activation flows: protocols via player selection, hazards via tier progression — unchanged

No unified `Mutator` enum discriminator, no unified registry. The split is semantic; consolidation is structural only.

### What UI moves (and what doesn't)

- Stays in `state/run/`: the actual selection screens.
  - `state/run/chip_select/` — shows 3 chips + 1 protocol; skip button (subsumes `greed-skip-button.md`)
  - `state/run/hazard_select/` — hazard reveal/notification UI
- Moves to `mutators/`: the dispatch systems that read selection messages and activate the chosen protocol/hazard.
  - `dispatch_protocol_selection` → `mutators/protocols/systems/dispatch.rs`
  - `generate_protocol_offering` → `mutators/protocols/systems/offering.rs`
  - Per-hazard tier-progression activation handlers → `mutators/hazards/systems/tier_activate.rs`

The UI layer calls into the mutators domain via messages (`ProtocolSelected { kind }`); mutators domain responds by activating and emitting `ProtocolActivated`. Message-driven, no direct coupling.

### What this remediation subsumes

Fold into this remediation (stop tracking separately):

- `audit/remediations/run-end-config-cleanup.md` — per-mechanic cleanup ownership naturally falls out of the subdirectory structure; each mechanic owns `wire_cleanup()` in its own `wire(app)`.
- `audit/remediations/greed-skip-button.md` — UI lives in `state/run/chip_select/` (not in mutators). The dispatch wire-up lands here because that's where `dispatch_protocol_selection` moves.
- Per-protocol / per-hazard cleanup tests — each mechanic's `tests/cleanup.rs` moves with its directory under the new tree.

Does NOT subsume (remain independent):

- The unified death pipeline crate (TODO #1) — prerequisite, not subsumed.
- Per-protocol / per-hazard BEHAVIORAL remediations (anchor piercing, burnout speed boost via effect stack, fission spawn rewrite, etc.) — those are mechanic-internal and land independently.
- Scenarios and invariant remediations — orthogonal.

### Architecture how-to (new deliverable)

Create `docs/architecture/creating-a-mutator.md` as the canonical walkthrough for adding a new protocol or hazard. Contents:

1. **Pick a directory** — `mutators/protocols/<name>/` or `mutators/hazards/<name>/` based on intent (helps player = protocol; makes run harder = hazard).
2. **Scaffold the files** — `mod.rs`, `system.rs`, `tests/`; what each file contains.
3. **Define the config** — `<Name>Config` resource, RON tuning variant, `activate()` function.
4. **Write the systems** — one system per file ideally; use `pub fn` and export from `mod.rs`.
5. **Write `wire(app)`** — register activation, timers, cleanup, UI hooks. Do NOT register chain-participating systems here.
6. **If the mechanic participates in the MutateDamage chain** — export the system fn, then edit `mutators/plugin.rs`'s `wire_damage_chain` to import and chain it. Decide carefully where it goes in the tuple.
7. **Register in `wire_protocols` or `wire_hazards`** — add the `crate::mutators::<kind>s::<name>::wire(app)` line.
8. **Add the mechanic to its registry** — `ProtocolRegistry` or `HazardRegistry`.
9. **Write tests** — `tests/` directory with cleanup test + behavioral tests.
10. **Wire the RON asset** — `breaker-game/assets/<kind>s/<name>.<kind>.ron`.

The how-to is ~1 page long, emphasizes the central-chain-assembly rule, links to this remediation and the crate detail, and is maintained alongside this refactor.

## Migration plan

Each step is a separate TDD-style commit. Steps can land sequentially on a feature branch.

### Step 1: Move directories

`git mv breaker-game/src/hazard breaker-game/src/mutators/hazards` and `git mv breaker-game/src/protocol breaker-game/src/mutators/protocols`.

Create `breaker-game/src/mutators/mod.rs` with:
```rust
pub mod plugin;
pub mod protocols;
pub mod hazards;
```

### Step 2: Sweep imports

`crate::hazard::*` → `crate::mutators::hazards::*` and `crate::protocol::*` → `crate::mutators::protocols::*`. Mechanical search-and-replace; compile until green.

### Step 3: Introduce `MutatorsPlugin`, retire `HazardPlugin` and `ProtocolPlugin`

- Write `mutators/plugin.rs` with `MutatorsPlugin` and stub `wire_protocols`/`wire_hazards`/`wire_damage_chain`/`wire_cleanup` functions.
- Move `HazardPlugin::build`'s contents into `wire_hazards`; delete `HazardPlugin`.
- Move `ProtocolPlugin::build`'s contents into `wire_protocols`; delete `ProtocolPlugin`.
- Top-level `game.rs` swaps `app.add_plugins((HazardPlugin, ProtocolPlugin))` for `app.add_plugins(MutatorsPlugin)`.

### Step 4: Refactor per-mechanic modules to `wire(app)` pattern

For each of 31 mechanics:
- Extract registration logic into `pub fn wire(app: &mut App)` in the mechanic's `mod.rs`.
- Call it from `wire_protocols` / `wire_hazards`.
- If the mechanic has a chain-participating system, REMOVE its registration from `wire(app)` and add to `wire_damage_chain`'s tuple.

### Step 5: Relocate dispatch + offering systems

Move `protocol/systems/dispatch_protocol_selection.rs` and `generate_protocol_offering.rs` into `mutators/protocols/systems/` if not already there. Same for hazard-side activation handlers.

### Step 6: Fold in `run-end-config-cleanup`

Each mechanic's cleanup test moves into its own `tests/cleanup.rs`. The centralized run-end-cleanup suite is deleted. `wire_cleanup` in `mutators/plugin.rs` dispatches to per-mechanic cleanup drivers (or each mechanic's `wire(app)` registers its own `OnExit(RunState::Finished)` system — prefer this for cohesion).

### Step 7: Fold in `greed-skip-button` (UI side)

The skip button itself goes in `state/run/chip_select/` (UI domain, not mutators). The dispatch that reads a "skip" message and activates a default protocol lives in `mutators/protocols/systems/dispatch.rs`. Land this step as part of the `state/run/chip_select/` UI work.

### Step 8: Write architecture how-to

Create `docs/architecture/creating-a-mutator.md`. Cross-link from `docs/architecture/plugins.md` and from this detail.

### Step 9: Verification

- Standard Verification Tier passes (lint + tests + reviewers)
- Full Verification Tier passes (scenarios + guards)
- No behavioral change — pre-refactor scenario pass rates equal post-refactor pass rates

## Tests

Mostly existing tests move with their files. New tests required:

- **MutatorsPlugin integration test**: headless app with `MutatorsPlugin`, verify both `ProtocolRegistry` and `HazardRegistry` populate, verify `ProtocolActivated` and `HazardActivated` messages register, verify the MutateDamage chain runs in declared order.
- **Per-mechanic cleanup tests**: move from the centralized `run-end-config-cleanup` suite into each mechanic's `tests/cleanup.rs`. Assert the mechanic's runtime resources (timers, stacks, per-node trackers) are removed on `OnExit(RunState::Finished)`; tuning resources (Config) survive.

## Cross-cutting implications

- `docs/architecture/plugins.md` — update to reflect the consolidated domain. The old entries for HazardPlugin and ProtocolPlugin merge into a single MutatorsPlugin entry.
- `docs/architecture/messages.md` — update cross-domain message table; remove Hazard<->Protocol cross-references, add Mutators<->State cross-references for UI dispatch.
- `docs/design/terminology/` — "mutator" becomes a domain term meaning "protocol or hazard." Keep "protocol" and "hazard" as kind-specific terms; add "mutator" as the umbrella.
- Scenarios that spawn hazards or protocols for testing continue to work — the activation API is unchanged.

## Ordering vs. TODO #1

TODO #1 (unified death pipeline crate) lands first. This refactor (TODO #2 — proposed) lands second. Rationale:

- Crate move is smaller and unblocks more downstream remediations.
- Mutators refactor is structural-only; it's valuable independent of when it lands.
- Doing the crate first means this refactor just imports the crate's types and sets; no re-migration.
- During the crate→mutators interim, the MutateDamage chain is wired from `hazard/plugin.rs` and `protocol/plugin.rs` directly. Ugly but works. Tests pass. This remediation closes that interim.

## Scope boundary

In scope:
- Directory restructure
- Single plugin consolidation
- Central chain assembly
- `wire(app)` pattern per mechanic
- UI dispatch relocation
- Run-end cleanup consolidation
- Architecture how-to authoring

Out of scope:
- Behavioral changes to any individual protocol or hazard
- Unification of registries, active-sets, or activation messages
- UI presentation changes (selection screens stay in `state/run/`)
- Evolution of the `MutateDamage` chain itself (new members land via their own remediations)

## TODO

Add to `docs/todos/TODO.md` as #2:

> **[ready]** Consolidate `hazard/` + `protocol/` into single `mutators/` domain — structural refactor with per-mechanic subdirectories, single `MutatorsPlugin`, central MutateDamage chain assembly, per-mechanic `wire(app)` pattern. Subsumes `run-end-config-cleanup.md` and `greed-skip-button.md` (dispatch side). Depends on #1. — [detail](detail/mutators-domain-refactor.md)
