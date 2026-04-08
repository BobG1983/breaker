# RNG Usage Map — Brickbreaker

**Date:** 2026-04-08  
**Bevy version:** 0.18.1 (confirmed from `breaker-game/Cargo.toml`)  
**rand version:** 0.9 (both crates)  
**RNG backend:** `ChaCha8Rng` (game), `SmallRng` (scenario runner)

---

## Summary of Findings

The game uses a single global `GameRng(ChaCha8Rng)` resource for all gameplay
randomness. There are no `thread_rng` or `OsRng` call sites other than the one
intentional path in `reset_run_state` for unseeded runs. The RNG is a Bevy
`Resource` accessed via `ResMut<GameRng>`, which means Bevy's borrow checker
serializes any two systems that both hold it — they cannot run in parallel.
However, ordering *within* a schedule stage between consuming systems is not
always explicit, which creates determinism sensitivity.

The scenario runner has its own completely separate RNG (`SmallRng`) for input
injection and never shares state with the game's `GameRng`. Scenarios always
force a known seed (default `0`) by writing to `RunSeed`.

The current architecture is a **flat single-stream design**: every RNG consumer
draws from the same sequential stream in the order Bevy schedules them. There is
no domain-partitioned seeding.

---

## All RNG Call Sites

| # | File | System / Function | Schedule | What It Randomizes | Draws Per Call | Determinism Risk |
|---|------|-------------------|----------|--------------------|----------------|------------------|
| 1 | `state/run/loading/systems/reset_run_state.rs` | `reset_run_state` | `OnExit(MenuState::Main)` | Reseeds `GameRng` from `RunSeed` or `from_os_rng()` | — (reseed) | LOW — runs alone, explicit ordering via `.after(reset_run_state)` on #2 |
| 2 | `state/run/loading/systems/generate_node_sequence/system.rs` | `generate_node_sequence_system` | `OnExit(MenuState::Main)` | Tier node counts (`random_range`) and per-tier shuffle | 1 per `TierNodeCount::Range` tier + 1 shuffle per tier | LOW — ordered `.after(reset_run_state)`, no other consumers in this schedule slot |
| 3 | `state/run/loading/systems/capture_run_seed.rs` | `capture_run_seed` | `OnEnter(NodeState::Loading)` | Generates the `RunStats.seed` from RNG (only on node 0 when `RunSeed` is `None`), then **reseeds GameRng** from that value | 1 `random::<u64>()` | MEDIUM — runs alongside `setup_run` and `reset_highlight_tracker` in the same `OnEnter(NodeState::Loading)` set; no explicit order relative to #4 |
| 4 | `state/run/systems/setup_run/system.rs` | `setup_run` | `OnEnter(NodeState::Loading)` | Random bolt launch angle on nodes > 0 | 1 `random_range(-spread..=spread)` | MEDIUM — shares schedule slot with `capture_run_seed` (#3); no ordering between them |
| 5 | `state/run/node/systems/reset_bolt/system.rs` | `reset_bolt` | `OnEnter(NodeState::Loading)` | Random bolt launch angle on nodes > 0 (per-bolt) | 1 per non-extra bolt | MEDIUM — shares schedule slot with #3 and #4; no explicit ordering between them |
| 6 | `bolt/systems/launch_bolt/system.rs` | `launch_bolt` | `FixedUpdate` (Playing) | Player-triggered bolt launch angle | 1 per serving bolt when Bump pressed | LOW — gated by `InputActions.active(Bump)`, single consumer |
| 7 | `bolt/systems/bolt_lost/system.rs` | `bolt_lost` | `FixedUpdate` (Playing) | Respawn angle for baseline (non-extra) lost bolts | 1 per lost bolt | MEDIUM — runs `.after(clamp_bolt_to_playfield).after(EnforceDistanceConstraints)`; no relative ordering with chain_lightning tick (#8) which also draws from GameRng |
| 8 | `effect/effects/chain_lightning/effect.rs` | `tick_chain_lightning` | `FixedUpdate` (Playing) | Next chain target selection (`choose`) per jump | 1 per `Idle` chain per tick | HIGH — runs unordered relative to `bolt_lost` (#7); both draw from GameRng in the same FixedUpdate tick; relative schedule position depends on Bevy's archetype walk order |
| 9 | `effect/effects/chain_lightning/effect.rs` | `fire` (called via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Initial chain target selection | 1 `choose` | HIGH — called during effect dispatch, ordering relative to other GameRng consumers in same tick is implicit |
| 10 | `effect/effects/spawn_bolts/effect.rs` | `fire` (via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Launch angle per spawned bolt | 1 per bolt count | MEDIUM — same concerns as #9 |
| 11 | `effect/effects/spawn_phantom/effect.rs` | `fire` (via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Launch angle for phantom bolt | 1 | MEDIUM — same concerns as #9 |
| 12 | `effect/effects/chain_bolt/effect.rs` | `fire` (via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Launch angle for chain bolt | 1 | MEDIUM — same concerns as #9 |
| 13 | `effect/effects/tether_beam/effect.rs` | `spawn_tether_bolt` (called from `fire`) | `FixedUpdate` (Playing) via effect bridge | Launch angle per tether bolt | 1 per bolt (2 in standard mode) | MEDIUM — same concerns as #9 |
| 14 | `effect/effects/random_effect/system.rs` | `fire` (via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Weighted selection of sub-effect | 1 | MEDIUM — same concerns as #9 |
| 15 | `effect/effects/entropy_engine/effect.rs` | `fire` (via effect bridge) | `FixedUpdate` (Playing) via effect bridge | Multiple weighted sub-effect selections (scales with cells destroyed) | 1 per effect fired (up to `max_effects`) | MEDIUM — same concerns as #9; draw count varies per call |
| 16 | `state/run/chip_select/systems/generate_chip_offerings.rs` | `generate_chip_offerings` | `OnEnter(GameState::ChipSelect)` | Weighted chip selection (via `chips/offering/system.rs`) | 1 per draw per slot (up to `offers_per_node`) | LOW — runs alone in its schedule slot, single consumer |
| 17 | `state/run/node/lifecycle/systems/spawn_highlight_text/system.rs` | `spawn_highlight_text` | `Update` (Playing) | Horizontal jitter for popup text position | 1 per popup spawned | LOW — visual only, purely cosmetic; does NOT affect gameplay outcomes |
| 18 | `state/transition/system.rs` | `spawn_transition_out` | `OnEnter(NodeState::TransitionOut)` or equivalent | Flash vs Sweep transition style | 1 `random_range(0..2)` | LOW — visual only, no gameplay effect |
| 19 | `state/transition/system.rs` | `spawn_transition_in` | `OnEnter(NodeState::TransitionIn)` or equivalent | Flash vs Sweep transition style | 1 `random_range(0..2)` | LOW — visual only, no gameplay effect |

### Scenario Runner RNG (separate, never shared with game)

| # | File | Driver | What It Randomizes | RNG Type |
|---|------|--------|--------------------|----------|
| SR-1 | `breaker-scenario-runner/src/input/drivers.rs` | `ChaosDriver` | Random gameplay action injection per frame | `SmallRng` seeded from scenario `seed` field |
| SR-2 | `breaker-scenario-runner/src/input/drivers.rs` | `HybridInput` | Random actions after scripted warmup phase | `SmallRng` (delegates to inner `ChaosDriver`) |
| SR-3 | `breaker-scenario-runner/src/lifecycle/systems/perfect_tracking.rs` | `PerfectDriver` | Breaker X offset relative to bolt position | `SmallRng` seeded from scenario `seed` field |
| SR-4 | `breaker-scenario-runner/src/lifecycle/systems/perfect_tracking.rs` | `PerfectDriver` (Random mode) | Random bump grade (`choose` from Early/Perfect/Late) | same `SmallRng` |

---

## Current Seed Flow Diagram

```
Player Menu Action (Enter)
        │
        ▼
handle_run_setup_input  (Update, MenuState::Selecting)
  └─ writes RunSeed(Some(n)) or RunSeed(None)
        │
        ▼ OnExit(MenuState::Main) — ordered
reset_run_state
  ├─ RunSeed(Some(n)) → GameRng = ChaCha8Rng::seed_from_u64(n)
  └─ RunSeed(None)    → GameRng = ChaCha8Rng::from_os_rng()
        │
        ▼ OnExit(MenuState::Main) — after reset_run_state
generate_node_sequence_system
  └─ draws from GameRng → NodeSequence resource
        │
        ▼ OnEnter(NodeState::Loading) — unordered set containing #3, #4, #5
 ┌──────┴───────────────────────────────────┐
 │                                          │
capture_run_seed                      reset_bolt / setup_run
  └─ if seed==0: draws u64 from GameRng        └─ draws angle from GameRng
     then reseeds GameRng with that value           (node > 0)
        │
        ▼ FixedUpdate (Playing) — concurrent draws, implicit ordering
  launch_bolt, bolt_lost, chain_lightning::fire,
  chain_lightning::tick, spawn_bolts, spawn_phantom,
  chain_bolt, tether_beam, random_effect, entropy_engine
        │
        ▼ OnEnter(ChipSelect)
generate_chip_offerings
  └─ weighted draws from GameRng → ChipOffers
        │
        ▼ OnEnter(Transition states) — visual only
spawn_transition_out / spawn_transition_in
  └─ 1 draw each for Flash vs Sweep
        │
        ▼ Update (Playing) — visual only
spawn_highlight_text
  └─ 1 draw per popup for X jitter

Scenario Runner (entirely separate):
  ScenarioDefinition.seed ──► SmallRng ──► ChaosDriver / PerfectDriver
  bypass_menu_to_playing forces RunSeed(Some(scenario.seed.unwrap_or(0)))
  so GameRng gets seed 0 by default in all scenarios
```

---

## Identified Risks

### Risk 1: Ambiguous ordering on `OnEnter(NodeState::Loading)`

`capture_run_seed`, `setup_run`, and `reset_bolt` all run in the same
`OnEnter(NodeState::Loading)` set and all draw from `GameRng`. Bevy serializes
them because `ResMut<GameRng>` is exclusive, but the execution order among these
three systems is determined by Bevy's internal scheduling and may vary across
Bevy versions or when other systems are added to the same set.

Specifically: `capture_run_seed` on the first node with `RunSeed(None)` draws
a `u64` *and* reseeds `GameRng` mid-run. If `setup_run` or `reset_bolt` runs
first, it draws from the OS-entropy-seeded state. If `capture_run_seed` runs
first, the other two draw from the deterministically re-derived ChaCha8 state.
This is the most significant ordering ambiguity in the current design.

**Observed mitigation:** The `BoltPlugin` registers `reset_bolt` with
`.after(BreakerSystems::Reset)` and `setup_run` is in the same set alongside
`capture_run_seed`. There is no `.after(capture_run_seed)` on either.

### Risk 2: Effect bridge fires draw from GameRng with no cross-effect ordering

Effects are dispatched through the bridge pattern during `FixedUpdate`. When
multiple effects that draw from `GameRng` fire in the same tick (e.g. `chain
lightning` + `spawn_bolts` + `random_effect` from a multi-chip setup), the
order of their RNG draws depends on entity iteration order inside the bridge
dispatch. Entity order in Bevy ECS is determined by archetype and storage
layout, which is affected by entity spawn order and component insertion history.
This is stable within a session but could differ between sessions with the same
seed if entity spawn ordering changes (e.g. after a Bevy upgrade or refactor).

### Risk 3: `tick_chain_lightning` and `bolt_lost` share FixedUpdate without mutual ordering

Both `tick_chain_lightning` (in `chain_lightning::register`) and `bolt_lost`
(in `BoltPlugin`) run in `FixedUpdate` with `Playing` guard. Both hold
`ResMut<GameRng>`. Bevy serializes them because of the exclusive borrow, but
the schedule order between them is not pinned. `tick_chain_lightning` is ordered
`.after(PhysicsSystems::MaintainQuadtree)`; `bolt_lost` is ordered
`.after(EnforceDistanceConstraints).after(clamp_bolt_to_playfield)`. There is no
`before`/`after` relationship between these two, so their draw order is
implementation-defined within any given Bevy version.

### Risk 4: Visual systems draw from gameplay RNG

`spawn_highlight_text` (Update) and `spawn_transition_out/in` (OnEnter)
consume `ResMut<GameRng>` for cosmetic randomness (popup jitter, transition
style). These draws shift the stream position for subsequent gameplay draws.
If either visual system fires in the same frame as a gameplay draw, it affects
what the gameplay system sees. For the transition systems this is unlikely to
matter, but the highlight popup system runs in `Update` concurrently with
`FixedUpdate` gameplay, creating potential for inter-schedule RNG interleaving
when the game runs at variable frame rates.

### Risk 5: `capture_run_seed` mid-run reseed is fragile

When `RunSeed` is `None`, `capture_run_seed` draws a `u64` from the
OS-seeded `GameRng` and then reseeds `GameRng` from that value to create a
deterministic sub-stream. The intent is to give the run a stable seed even
when the player didn't specify one. But the value of that initial draw depends
on whatever state `GameRng` was in when `capture_run_seed` ran — which is after
`reset_run_state` (OS entropy) and potentially after `generate_node_sequence`
(which drew from the same stream). So the "stable" seed is actually derived from
an uncontrolled sub-sequence.

### Risk 6: No seed visible to player until after node 0 starts

The player-visible seed (`RunStats.seed`) is only captured by `capture_run_seed`
at `OnEnter(NodeState::Loading)` — after the run starts. The `SeedEntry` UI
resource holds the typed seed, but `RunSeed` is written only at confirmation and
`RunStats.seed` is only written after the run begins. This means there is a
window where `RunStats.seed` is `0` (its default). For seeded runs (seed sharing)
this is fine — the value is identical to the input. For unseeded runs the player
cannot learn the seed until they are already in node 0.

---

## Recommendations for Hierarchical Seed Architecture

The planned node sequencing refactor intends to introduce hierarchical seed
derivation: `run_seed → tier_seed → node_seed → slot_seed → cell_seed`.
The current RNG landscape is compatible with this direction but requires
the following structural changes.

### Recommendation 1: Derive all seeds deterministically from `run_seed`

Replace the flat single-stream model with derived sub-seeds. The run seed
should be fixed before any gameplay randomness occurs:

```
run_seed (captured from RunSeed or from os_rng before anything else)
  ├─ node_sequence_seed  = hash(run_seed, 0x01)  → used by generate_node_sequence
  ├─ tier_seed[i]        = hash(run_seed, 0x10 | i)
  ├─ node_seed[i]        = hash(run_seed, 0x20 | i)
  │    ├─ slot_seed[j]   = hash(node_seed, j)
  │    └─ cell_seed[k]   = hash(node_seed, 0x100 | k)
  └─ bolt_launch_seed    = hash(run_seed, 0x30)  → bolt angle RNG per node
```

This eliminates Risk 5 entirely: the run seed is determined once, upfront,
before any draws.

### Recommendation 2: Separate cosmetic RNG from gameplay RNG

Create a second resource, `FxRng`, for all visual-only draws (highlight jitter,
transition style). This eliminates Risk 4 and keeps the gameplay RNG stream
clean for replay.

### Recommendation 3: Pin ordering between `capture_run_seed` and `setup_run`/`reset_bolt`

Until hierarchical seeding is in place, add explicit `.after(capture_run_seed)`
ordering on both `setup_run` and `reset_bolt` in their `OnEnter(NodeState::Loading)`
registrations. This resolves Risk 1.

### Recommendation 4: One RNG per effect domain, not one global

For effect-bridge draws (chain lightning, spawn bolts, phantom, tether beam,
random effect, entropy engine), derive a per-event RNG from the node seed and
the event index. This eliminates Risk 2 and makes the draw sequence independent
of entity spawn order and archetype layout.

Concrete approach: `EffectRng = ChaCha8Rng::seed_from_u64(hash(node_seed, event_counter))`
where `event_counter` is a `Resource` incremented each time an effect fires.
The counter is reset at node start.

### Recommendation 5: Capture run seed before `generate_node_sequence`

In the new architecture, move seed capture to `OnExit(MenuState::Main)` before
`generate_node_sequence`. This makes the seed always known before any gameplay
draws occur and resolves Risk 6 (player can see the seed before the first node
renders).

Ordering: `capture_run_seed` → `reset_run_state` → `generate_node_sequence`

### Recommendation 6: Document what the run seed covers

For the seed-sharing feature to work correctly, players need to know what is
and is not covered by the seed. The current design covers:
- Node sequence (type/tier ordering, Active/Passive/Boss distribution)
- Chip offerings at each chip select
- Bolt launch angles (when node starts)
- Bolt respawn angles (when bolt is lost)

It does NOT reproducibly cover:
- Effect-bridge draw order (Risk 2)
- The initial OS-entropy draw when `RunSeed(None)` (by design)
- Highlight popup positions (visual only, but consumes RNG)
- Transition style (visual only, but consumes RNG)

For full run replay, all of the above must be covered. The hierarchical seed
architecture addresses this if FxRng is separated (Recommendation 2) and
effect draws are indexed by event counter (Recommendation 4).

---

## Key Files

- `breaker-game/src/shared/rng.rs` — `GameRng(ChaCha8Rng)` definition and `from_seed(u64)` constructor
- `breaker-game/src/shared/resources.rs` — `RunSeed(Option<u64>)` resource (player-visible seed)
- `breaker-game/src/state/run/loading/systems/reset_run_state.rs` — reseeds `GameRng` at run start; the only `from_os_rng()` call site
- `breaker-game/src/state/run/loading/systems/capture_run_seed.rs` — captures `RunStats.seed` and re-derives deterministic stream for unseeded runs (Risk 5)
- `breaker-game/src/state/run/loading/systems/generate_node_sequence/system.rs` — first major gameplay draw; uses `random_range` + `shuffle`
- `breaker-game/src/state/run/chip_select/systems/generate_chip_offerings.rs` — chip selection weighted draw via `chips/offering/system.rs`
- `breaker-game/src/effect/effects/chain_lightning/effect.rs` — `fire()` and `tick_chain_lightning()` both draw from `GameRng`; highest draw frequency during gameplay
- `breaker-game/src/effect/effects/entropy_engine/effect.rs` — variable-draw-count effect (1 to `max_effects` per cell hit)
- `breaker-game/src/state/run/plugin.rs` — schedule registration for run-domain systems
- `breaker-game/src/bolt/plugin.rs` — schedule registration including `bolt_lost` and `launch_bolt` ordering
- `breaker-game/src/state/transition/system.rs` — cosmetic `GameRng` draw (transition style picker)
- `breaker-game/src/state/run/node/lifecycle/systems/spawn_highlight_text/system.rs` — cosmetic `GameRng` draw in `Update` schedule
- `breaker-scenario-runner/src/input/drivers.rs` — `ChaosDriver` + `PerfectDriver` using `SmallRng`; completely separate from game RNG
- `breaker-scenario-runner/src/lifecycle/systems/menu_bypass.rs` — forces `RunSeed(Some(0))` for all scenarios
- `breaker-scenario-runner/src/types/definitions/scenario.rs` — `ScenarioDefinition.seed` field controls both game seed and driver seed
