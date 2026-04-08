# Protocol & Hazard System Design

## Summary
Design the protocol (positive) and hazard (negative) systems — per-tier upgrades and stackable debuffs that shape run identity and infinite scaling.

## Context
Protocols and hazards are a new upgrade category distinct from chips. Protocols provide powerful, run-altering positive effects (Balatro coupon analog). Hazards are stackable debuffs that the player chooses from during infinite play (choose-your-poison model). The goal is bragging rights — "I got to tier 16" because by that point you've layered on multiple stacking difficulties against your god-tier build.

## Design

### Protocols (Positive)
- Each protocol can only appear **once** per run
- Only **1 protocol** is offered per tier
- Protocol is an **extra entry** on the chip select screen — displayed below the 3 chip offerings (landscape orientation, spanning width). Player picks EITHER a chip OR the protocol — picking either **closes the screen**. Opportunity cost is giving up a chip to take the protocol
- Possibly costs something to buy (currency TBD)
- Once picked, remains visible but greyed out and crossed out (not selectable)
- Should interact with chip system for synergy possibilities

### Hazards (Negative — Tier 9+)
Starting around tier 9 (infinite play — name TBD, "infinity mode" placeholder), the player is shown **3 random hazards** and **must pick one** (choose-your-poison). Hazard selection happens on its own dedicated **timed** UI screen **after** the chip/protocol selection screen, only at tier 9+. On timer expiry, a hazard is auto-picked at random.

**Key mechanics:**
- **16 hazards** in a single flat pool (no severity tiers)
- **Hazards can stack** — the same hazard can be picked more than once, each stack increases intensity
- All hazards are roughly equally punishing in isolation
- Difficulty comes from **stack count** (more = harder), **synergy combos** (some pairs are WAY worse together), and **player knowledge** (experienced players learn which "easy-looking" picks become traps)
- "Per level" means per level after the first (level 1 = base amount)
- Hazards are **code-implemented systems** (likely complex behavior), not RON-authored. Tuning values (percentages, durations, thresholds) are loaded from RON so they can be adjusted without recompiling. Values below are design targets.

### Hard Rules for Hazards
- **NO disabling** chips, bumping, dashing, or any player capability
- **Nothing that feels cheaty** — the player should feel overwhelmed, not robbed
- Hazards are **escalatingly negative** — pure difficulty that stacks
- Every hazard must be **readable and telegraphed** — the player can see and respond
- Hazards should feel like the game got meaner, not like the game cheated
- Hazards must create a **new mechanical dynamic** — something you read, respond to, make decisions about. If the only description is "X goes up" or "X goes down," it's not interesting enough.
- Hazards are **problems to solve**, not handicaps to endure. Every hazard has a strategy to play around it. A master barely notices a hazard. A novice gets crushed. The hazard is a mirror of your skill.

### Hazard Pool (16)

1. **Decay** — node timer ticks faster. 15%+5%/level.
2. **Drift** — wind pushes bolt in a telegraphed direction, changes every ~8s. Force+force/3/level.
3. **Haste** — bolt speed increase (multiplicative with existing speed). 20%+10%/level.
4. **Echo Cells** — destroyed cells leave a ghost after 1.5s that must be cleared. Don't carry original cell rules (no re-unlocking etc). 1 HP, doubles per level (1/2/4/8...).
5. **Erosion** — breaker shrinks over time. Non-whiff bumps restore 25% of what was lost. Perfect bumps restore 50% of what was lost. Min width 35%. Also reduces bump window height proportionally. Shrink rate TBD (number that divides neatly into 100%/second).
6. **Cascade** — destroyed cell heals adjacent cells. +10 HP+5 HP/level.
7. **Fracture** — destroyed cells split into adjacent empty cells. 2+1/level.
8. **Renewal** — cells have countdown timer, regen to full HP on expiry, timer resets shorter. 10s-20%/level (diminishing returns).
9. **Diffusion** — incoming damage shared with adjacent cells (depth 1, doesn't cascade). Target takes less. 20%+10%/level. +1 cascade depth every 5 levels.
10. **Tether** — adjacent cell pairs linked with visible beams. Damage to one deals a percentage to its partner. 25%+10%/level. Link coverage 40%+10%/level of eligible pairs. Sounds helpful — isn't (spreads non-lethal chip damage that feeds Cascade/Renewal). Masters find chain-collapse sequences.
11. **Volatility** — cells gain HP when not being hit. Cap at 2x starting HP. +1 HP/5s, -diminishing%/level (floor TBD). Neglect tax — must keep touching cells to suppress growth.
12. **Gravity Surge** — destroyed cells spawn short-lived gravity wells pulling the bolt. 2s+1s/level duration, pull strength+x%/level (diminishing returns).
13. **Overcharge** — bolt gains speed per cell destroyed within a bump cycle, resets on bump. 5%+3%/level per kill (multiplicative). Sounds like a buff. Isn't.
14. **Resonance** — every kill after the 2nd within a time window fires a slow-mo wave toward the breaker. Dodgeable, slow travel speed. 0.5s+0.3s/level window. Slow duration and strength have diminishing returns per level.
15. **Momentum** — non-lethal hits give the cell HP. Cell splits into 2 at 1x starting HP each when reaching 2x starting HP (into adjacent empty cells). +10 HP+10 HP/level per non-lethal hit.
16. **Sympathy** — damage dealt to a cell heals each adjacent cell for a percentage of damage dealt (depth 1, doesn't cascade). 25%+5%/level. +1 cascade depth every 5 levels.

### Why a Flat Pool (No Severity Tiers)
Severity tiers (Taxing/Punishing/Terminal) add artificial structure that doesn't serve gameplay. A flat pool with emergent synergies is more roguelite — the "obvious safe pick" is a trap if you don't know the meta. Difficulty comes from:
1. **Stack count** — more hazards = harder, period
2. **Synergy combos** — some pairs are WAY worse together than their parts suggest
3. **Player knowledge** — experienced players learn which combinations are deadly

This is the Balatro model: depth through emergent interaction, not through explicit difficulty labels.

### Trap Synergies (Player Knowledge Rewards)
The best hazard designs look manageable in isolation but combine devastatingly:

- **Echo Cells + Volatility** = ghosts look free (1 HP) but grow rapidly if not cleared immediately
- **Tether + Cascade** = Tether spreads non-lethal chip damage to partners, Cascade heals neighbors on kills — damage you spread via Tether feeds the Cascade heal loop
- **Diffusion + Sympathy** = both gain cascade depth every 5 levels; at depth 2+, clusters become nearly impenetrable damage sponges
- **Erosion + Haste + Overcharge** = catching a bolt at 2x speed with a 40% width breaker while it accelerates per kill
- **Fracture + Momentum** = splits create empty cells that are room for MORE Momentum splits; Volatility makes the debris grow
- **Fracture + Volatility** = "easy cleanup" 1-HP debris becomes 3-HP debris if you don't clear it fast
- **Decay + Renewal** = timer is ticking faster while cells are regenerating on their own countdown — double time pressure
- **Gravity Surge + Drift** = two forces on the bolt simultaneously, both readable but demanding to compensate for together
- **Overcharge + Haste** = bolt speed compounds multiplicatively within bump cycles
- **Resonance + Echo Cells** = clearing echo ghosts can trigger more resonance slow-waves
- **Momentum + Diffusion** = Diffusion bleeds damage to neighbors (can't one-shot), Momentum punishes non-lethal hits with HP growth

### Killed Hazard Proposals (with reasons)
- **Frenzy** (cells fire faster) — just a number modifier, no mechanical dynamic
- **Barrage** (cells gain spread shot) — just more projectiles, no decision-making
- **Dim** (brightness reduced) — visual impairment isn't fun
- **Density** (more cells) — just a number, not a mechanic
- **Blackout** — that's how HP already works visually
- **Magnetism** — cheaty as hazard (now a cell type instead)
- **Turbulence** — cheaty
- **Silence** — disables chip, HARD NO
- **Lockdown** — disables dash, HARD NO
- **Mirror** — inverted controls feel cheaty
- **Echo** (angle corruption) — per-hit angle corruption, can't respond meaningfully
- **Entropy** — erases cell type design, anti-variety
- **Volatile Revenge** — replaced by Resonance (more interesting)
- **Warp** — feels cheaty/random, cells rearranging is unreadable
- **Ablation** — just Volatile Revenge with extra steps
- **Fortress** — interesting but better as a cell type
- **Ricochet** — doesn't make sense
- **Aftershock** — hurts you for winning, punishes the core loop
- **Surge** (bolt grows) — number modifier, no decision
- **Undertow** (breaker drift) — fights player's direct controls
- **Tremor** (cells shift down) — arbitrary rule change
- **Phase Shift** (cells shift directionally) — too close to rejected Warp
- **Backlash** (steeper angles on damaged cells) — too narrow, only one build feels it
- **Scorch** (AoE kills burn floor) — not great
- **Convergence** (cells drift to impact) — visual complexity concern
- **Resonance Field** (clusters fire projectiles) — no
- **Anchor** (slow zones from kills) — visual overlap with Gravity Surge

### Why Choose-Your-Poison + Stacking
The choose-your-poison + stacking model creates decisions at two levels:
1. **Immediate**: "Which of these 3 is least bad for my current build + existing hazards?"
2. **Strategic**: "This looks easy now, but what happens when I stack it with what I already have?"

Stacking is what makes deep infinite runs impressive — not just variety of hazards but *intensity* of repeated ones. By tier 14+, the player's build is wild (multi-pierce, chain lightning, AoE, maxed speed) — hazards are the counterweight. Unstoppable force meets immovable object.

## Decisions

### Chip/Protocol Distinction
- **Chips** are **power** — buffs that don't change how you WANT to play
- **Protocols** are **rule changes** — fundamentally change how you WANT to play
- **The test**: Does this make me play differently, or just play the same way but stronger?

### Legendary Rarity Removal
Delete the Legendary rarity entirely. Only Common, Uncommon, Rare, and Evolution remain. All 11 existing legendaries → rework as Rare chips with tuned-down numbers. Anchor evolution → promote to protocol, delete evolution entry.

### Hazard Start
Hazards start at **tier 9** — right after completing the 8-tier structured run. This is the "infinity mode" boundary.

### Meta-Progression
Protocol pool grows via **meta-progression** across runs. Players unlock new protocols over time (like unlocking new Jokers in Balatro).

### RON Tunable + Hot Reloadable
ALL tuning values in every protocol AND every hazard must be RON-configurable and hot-reloadable. No hardcoded numbers.

### Content Targets
| Category | Initial (this todo) | Phase 7 target |
|----------|-------------------|----------------|
| Protocols | 15 (designed) | 30 |
| Hazards | 16 (designed) | 30 |

## Scope
- In: protocol offering logic (extra entry below chips on chip select screen, pick chip OR protocol), hazard choose-your-poison UI (pick from 3 random), hazard stacking, hazard systems (code-implemented, RON-tuned), UI for protocols on chip select screen, protocol/hazard state tracking, meta-progression for protocol unlocks, legendary rarity removal + retune as Rare
- Out: chip system changes (chips are separate), tier system (separate todo), evolution catalog redesign (Phase 7)

## Dependencies
- Depends on: node sequencing refactor (tiers must exist)
- Depends on: chip selection UI (protocols display on chip select screen), hazard selection UI (separate screen after chip selection, tier 9+ only)

## Design Files

| File | Contents |
|------|----------|
| [research/protocol-decisions.md](research/protocol-decisions.md) | **Source of truth** for all 15 protocol designs, legendary migration, declined protocols with reasons, graphics catalog note, open tuning questions |
| [research/protocol-brainstorm.md](research/protocol-brainstorm.md) | R1 agent brainstorm (15 initial candidates — input, not source of truth) |
| [research/protocol-candidates-r3.md](research/protocol-candidates-r3.md) | R3 agent brainstorm (8 candidates — input, not source of truth) |

## Technical Research Files

| File | Contents |
|------|----------|
| [research/interface-design.md](research/interface-design.md) | **Interface design** — concrete Rust types, traits, enums, struct layouts, RON formats, system patterns, cross-domain message inventory. Reviewed by architecture + idiom agents. |
| [research/chip-offering-flow.md](research/chip-offering-flow.md) | Full trace: ChipCatalog structure, offering algorithm, rarity weights, selection UI, effect dispatch, protocol integration point |
| [research/effect-system-architecture.md](research/effect-system-architecture.md) | Current effect system trace + planned new system primitives (Route/Stamp/Transfer/During/Killed), protocol-to-category mapping (A=effect tree, B=custom system) |
| [research/run-state-flow.md](research/run-state-flow.md) | Full state machine trace, node completion chain, chip select flow, HazardSelect insertion plan with 7 concrete wiring steps |
| [research/catalog-ron-patterns.md](research/catalog-ron-patterns.md) | SeedableConfig vs SeedableRegistry patterns, GameConfig derive macro, hot-reload wiring, ProtocolRegistry/HazardRegistry recommendation |
| [research/plugin-organization-patterns.md](research/plugin-organization-patterns.md) | Plugin structure survey, delegation pattern (effect/ model), cross-domain messages, scheduling patterns, recommended directory layout |
| [research/cross-domain-messages.md](research/cross-domain-messages.md) | New message struct definitions with full Rust code, ownership, producers, consumers |
| [research/registry-struct-patterns.md](research/registry-struct-patterns.md) | Exact code: BreakerRegistry, CellTypeRegistry, SeedableRegistry trait, ChipDefinition, GameConfig derive |
| [research/effect-item-patterns.md](research/effect-item-patterns.md) | Exact code: simple effect (damage_boost), complex effect (shockwave), trigger bridges, delegation pattern |
| [research/message-component-patterns.md](research/message-component-patterns.md) | Exact code: message derives, component patterns, per-run resources, enum patterns |

## What's Done
- **Game design**: 15 protocols designed with mechanics, behavior changes, synergies
- **Game design**: 16 hazards designed with stacking, trap synergies, killed proposals
- **Game design**: Chip/protocol distinction defined, legendary migration decided

## Technical Design

Research completed 2026-04-08. All concrete interface designs, struct layouts, and system patterns are in the referenced files below. Do NOT duplicate design details inline — read the source files.

| Document | What it covers |
|----------|---------------|
| [research/interface-design.md](research/interface-design.md) | **Primary reference**: all Rust types, enums, registries, resources, messages, system patterns, plugin structure, cross-domain rules |
| [research/cross-domain-messages.md](research/cross-domain-messages.md) | New message struct definitions: `HealCell`, `SpawnGhostCell`, `ApplyBoltForce`, `ApplyBreakerShrink`, `ApplyBreakerRestore` |
| [legendary-retuning.md](legendary-retuning.md) | Legendary removal + per-chip retuning plan (11 chips need Rare values — `[NEEDS DETAIL]`) |
| [protocols/](protocols/) | Per-protocol implementation guides (config, components, systems, behaviors, edge cases) |
| [hazards/](hazards/) | Per-hazard implementation guides (config, components, systems, stacking, behaviors, edge cases) |

### Key Technical Decisions (summary — details in interface-design.md)

- **Two plugins**: `ProtocolPlugin` + `HazardPlugin`, `effect/`-style delegation pattern
- **Tuning enum IS the kind**: `ProtocolTuning` / `HazardTuning` enum variants carry per-item fields, `kind()` derives the C-style `ProtocolKind` / `HazardKind`
- **Per-item config resources**: Tuning extracted at activation into typed config (e.g., `DebtCollectorConfig`). Systems read `Res<Config>` — zero enum matching at runtime.
- **`run_if` gating**: `protocol_active(kind)` / `hazard_active(kind)` closures prevent inactive systems from running
- **Message-driven cross-domain**: Hazards never mutate other domains' resources directly. Damage pipeline hazards (Diffusion, Tether, Sympathy, Momentum) are handled by the cell damage system reading hazard config resources.
- **Damage message**: After effect refactor (todo #2), `DamageCell` → `DamageDealt<Cell>`. The cell damage system (`apply_damage::<Cell>`) handles redistribution for Diffusion/Tether/Sympathy/Momentum.
- **Protocol offering**: Extra entry below chips. Pick protocol OR chip — either closes the screen. Random from seeded `GameRng`.
- **Hazard select**: Separate timed screen after chip select, tier 9+ only. Auto-picks at random on timer expiry.
- **Effect refactor**: Todo #2 completes before this todo starts. Effect-tree protocols use `ValidDef` types, `DamageDealt<Cell>`, `SourceId`, `Killed(KillTarget)` from the new system. No waiting needed.

## Implementation Order

This todo splits into 10 waves. Waves within a group can run in parallel. Hard dependencies gate between groups.

### Group A — No dependencies (can start now)

**Wave 1: Plugin infrastructure**
- Create `protocol/` module: `mod.rs`, `plugin.rs`, `definition.rs`, `resources.rs`, `messages.rs`
- Create `hazard/` module: same structure
- `ProtocolKind` (15 variants) + `ProtocolKind::ALL` const
- `HazardKind` (16 variants) + `HazardKind::ALL` const
- `ProtocolTuning` enum (all 15 variants with fields, `kind()`, `effects()`)
- `HazardTuning` enum (all 16 variants with fields, `kind()`)
- `ProtocolDefinition` + `HazardDefinition` asset structs
- `ProtocolRegistry` + `HazardRegistry` with `SeedableRegistry` impls
- `ActiveProtocols` + `ActiveHazards` + `UnlockedProtocols` resources
- `ProtocolOffer` + `HazardOffers` resources
- `ProtocolSelected` + `HazardSelected` messages
- `protocol_active()` + `hazard_active()` run_if helpers
- `ProtocolPlugin` + `HazardPlugin` shells (empty `protocols::register` / `hazards::register`)
- Wire registries into `defaults_plugin()`, plugins into `game.rs`
- Wire `ActiveProtocols.clear()` + `ActiveHazards.clear()` into `reset_run_state`
- Add terminology entries (done)
- **Files touched**: ~15 new files, 3 modified (game.rs, state/plugin/system.rs, reset_run_state.rs)
- **Detail**: [research/interface-design.md](research/interface-design.md) sections 1-8, 13

**Wave 2**: Removed — legendary removal is now a separate todo queued before this one. See [legendary-removal.md](../legendary-removal.md). By the time this todo starts, Legendary rarity is gone, chips are retuned, and Deadline/Ricochet Protocol/Anchor are ready for protocol RON creation.

### Group B — After Wave 1

**Wave 3: Protocol offering integration**
- `generate_protocol_offering` system: reads `ProtocolRegistry`, `ActiveProtocols`, `UnlockedProtocols`, `GameRng` → inserts `ProtocolOffer`
- Modify `spawn_chip_select`: spawn protocol card row below chip cards (landscape orientation)
- Modify `handle_chip_input`: up/down navigation between chip row and protocol row; protocol confirm sends `ProtocolSelected` + `ChangeState<ChipSelectState>`
- Modify `tick_chip_timer`: timer expiry skips protocol (auto-selects nothing for protocol)
- `dispatch_protocol_selection`: reads `ProtocolSelected`, inserts into `ActiveProtocols`, calls `protocols::activate()` (config resource insertion), dispatches effect trees via new effect system API
- Protocol cleanup system: removes config resources on `OnExit(MenuState::Main)`
- **Files touched**: `chip_select/systems/`, `chip_select/resources.rs`, `protocol/systems/`, `protocol/plugin.rs`
- **Detail**: [research/interface-design.md](research/interface-design.md) sections 7, 9, 10; [research/chip-offering-flow.md](research/chip-offering-flow.md)

**Wave 3b: Cross-domain messages** (parallel with Wave 3)
- Define `HealCell`, `SpawnGhostCell`, `ApplyBoltForce`, `ApplyBreakerShrink`, `ApplyBreakerRestore` messages in their owning domains
- Stub consuming systems (accept message, log, no-op) — real handlers come when hazards are implemented
- **Files touched**: `cells/messages.rs`, `bolt/messages.rs`, `breaker/messages.rs`, + consuming system stubs
- **Detail**: [research/cross-domain-messages.md](research/cross-domain-messages.md)

### Group C — After Wave 3

**Wave 4: Effect-tree protocols** (all 4 in parallel)
- Write RON effect trees for Deadline, Ricochet Protocol, Anchor, Kickstart using `ValidDef` format
- Wire `dispatch_protocol_selection` to use effect system dispatch API for effect installation
- **Files touched**: 4 `.protocol.ron` files, `dispatch_protocol_selection` effect branch
- **Detail**: [protocols/deadline.md](protocols/deadline.md), [protocols/ricochet_protocol.md](protocols/ricochet_protocol.md), [protocols/anchor.md](protocols/anchor.md), [protocols/kickstart.md](protocols/kickstart.md)

**Wave 5: Custom-system protocols** (all 10 in parallel, after Wave 3)

All custom-system protocols can run in parallel. Each gets: config resource, activate fn, runtime system(s), register(app), tests.

Simple batch (read existing messages only):
- **Greed**: `GreedStacks` resource, `ChipOfferSkipped` message, rarity boost in `generate_chip_offerings` — [protocols/greed.md](protocols/greed.md)
- **Siphon**: `SiphonStreak` resource, reads `CellDestroyedAt`, sends `ReverseTimePenalty` — [protocols/siphon.md](protocols/siphon.md)
- **Burnout**: `BurnoutHeat` component, heat gauge systems, mega-bump dispatch — [protocols/burnout.md](protocols/burnout.md)

`DamageDealt<Cell>` batch (send damage messages):
- **Debt Collector**: `DebtStack` component, bump grade tracking, cash-out bonus damage — [protocols/debt_collector.md](protocols/debt_collector.md)
- **Iron Curtain**: damage wave on bolt-lost — [protocols/iron_curtain.md](protocols/iron_curtain.md)
- **Echo Strike**: echo network, fractional echo damage — [protocols/echo_strike.md](protocols/echo_strike.md)
- **Reckless Dash**: risky zone detection, 4x damage boost — [protocols/reckless_dash.md](protocols/reckless_dash.md)
- **Afterimage**: phantom breaker spawn, phantom bolt piercing — [protocols/afterimage.md](protocols/afterimage.md)
- **Fission**: kill counter, bolt splitting — [protocols/fission.md](protocols/fission.md)

Conductor (chip effect filtering):
- **Conductor**: `Conducted` marker, primary swap, bolt-lost suppression, chip effect gating — [protocols/conductor.md](protocols/conductor.md)

### Group D — After Wave 1 (parallel with Group B/C)

**Wave 6: Hazard state flow**
- Add `RunState::HazardSelect` variant + `HazardSelectState` substate (5 variants)
- `resolve_post_chip_state` dynamic route: `tier_index >= HAZARD_TIER_THRESHOLD` → HazardSelect, else → Node
- `const HAZARD_TIER_THRESHOLD: u32 = 9` — a tier is a group of nodes (currently 4 nodes + 1 boss per tier). With the current 5-tier difficulty curve (tiers 0–4), `tier_index` never reaches 9, so the route never fires. When todo #7 extends the difficulty curve into infinite play (tiers 9+), hazards activate automatically with zero code changes.
- `HazardSelectPlugin`: `generate_hazard_offerings`, `spawn_hazard_select`, `handle_hazard_input`, `tick_hazard_timer`
- `dispatch_hazard_selection`: reads `HazardSelected`, increments `ActiveHazards`, calls `hazards::activate()`
- Wire routes in `register_routing()`
- Register `HazardSelectState` in `StatePlugin`
- **Files touched**: `state/types/run_state.rs`, new `state/types/hazard_select_state.rs`, `state/plugin/system.rs`, new `state/run/hazard_select/` module, `hazard/plugin.rs`
- **Detail**: [research/run-state-flow.md](research/run-state-flow.md) section 9; [research/interface-design.md](research/interface-design.md) section 10

**Wave 7: Hazard systems — simple batch** (parallel, after Wave 6)
These hazards send messages to other domains and don't touch the damage pipeline:
- **Decay**: `ApplyTimePenalty` — [hazards/decay.md](hazards/decay.md)
- **Drift**: `ApplyBoltForce` — [hazards/drift.md](hazards/drift.md)
- **Haste**: effect system `SpeedBoost` or message — [hazards/haste.md](hazards/haste.md)
- **Echo Cells**: `SpawnGhostCell` — [hazards/echo_cells.md](hazards/echo_cells.md)
- **Erosion**: `ApplyBreakerShrink` + `ApplyBreakerRestore` — [hazards/erosion.md](hazards/erosion.md)
- **Cascade**: `HealCell` — [hazards/cascade.md](hazards/cascade.md)
- **Fracture**: cell spawn — [hazards/fracture.md](hazards/fracture.md)
- **Renewal**: `HealCell` — [hazards/renewal.md](hazards/renewal.md)
- **Volatility**: `HealCell` — [hazards/volatility.md](hazards/volatility.md)
- **Gravity Surge**: `ApplyBoltForce` — [hazards/gravity_surge.md](hazards/gravity_surge.md)
- **Overcharge**: per-bolt speed tracking — [hazards/overcharge.md](hazards/overcharge.md)
- **Resonance**: wave entity spawning — [hazards/resonance.md](hazards/resonance.md)

**Wave 7b: Hazard systems — damage pipeline batch** (needs todo #2 for `DamageDealt<Cell>` + `apply_damage::<Cell>`)
These hazards modify how the cell damage system behaves:
- **Diffusion**: `DiffusionConfig` read by `apply_damage::<Cell>` — [hazards/diffusion.md](hazards/diffusion.md)
- **Tether**: `TetherConfig` + `TetherLink` components + link management — [hazards/tether.md](hazards/tether.md)
- **Momentum**: `MomentumConfig` read by `apply_damage::<Cell>` + split check — [hazards/momentum.md](hazards/momentum.md)
- **Sympathy**: `SympathyConfig` read by `apply_damage::<Cell>` — [hazards/sympathy.md](hazards/sympathy.md)

### Group E — Tier Regression (scaffolded now, completed after todo #7)

**Wave 8: Tier Regression protocol**
Implement everything except the actual `NodeSequence` mutation:
- `TierRegressionConfig { tiers_back: u32 }` resource, `activate()`, `register()`
- `ProtocolTuning::TierRegression` variant (already in Wave 1)
- `tier_regression.protocol.ron` with tuning values
- System that fires on activation: inserts `TierRegressionPending` resource/marker
- **Stub**: The system that would modify `NodeSequence` logs a warning and no-ops. When todo #7 lands, the stub is replaced with the actual tier manipulation (one system body change).
- Tests verify the scaffolding (config populated, marker inserted, offering works) but skip the actual regression behavior.
- **Detail**: [protocols/tier_regression.md](protocols/tier_regression.md)

### Group F — After all implementation

**Wave 9: Scenarios**
- New invariants: `ProtocolStateConsistent`, `HazardStackValid`, `HazardScalingBounded`
- Self-test scenarios for each invariant
- Adversarial chaos scenarios per custom-system protocol
- Hazard stacking stress scenarios for trap synergy pairs
- **Detail**: main detail file Scenario Coverage section

### Dependency Graph

```
                  ┌── Wave 3 ──┬── Wave 4 (effect-tree) ──────┐
                  │            └── Wave 5 (custom-system) ────┤
Wave 1 ──┬───────┤                                            │
          │       ├── Wave 3b                                  ├── Wave 9
          │       ├── Wave 6 ── Wave 7 ───────────────────────┤
          │       │         └── Wave 7b (damage pipeline) ────┤
          │       └── Wave 8 (scaffold, stub for todo #7) ────┘
Wave 2 ──┘
```

All external dependencies (todo #2 effect refactor) are complete before this todo starts. Todo #7 only blocks the Tier Regression system body (stubbed).

### Parallelism summary

| Phase | What runs | Blocked on |
|-------|-----------|------------|
| Start | Waves 1 + 2 in parallel | Nothing |
| After Wave 1 | Waves 3 + 3b + 6 + 8 in parallel | Wave 1 |
| After Wave 3 | Waves 4 + 5 in parallel (all 14 protocols) | Wave 3 |
| After Wave 6 | Waves 7 + 7b in parallel (all 16 hazards) | Wave 6 |
| After all waves | Wave 9 (scenarios) | Everything |

**Design decisions**:
- Protocol offering is random from seeded `GameRng` (deterministic from run seed)
- Picking a protocol closes the chip select screen (protocol OR chip, not both)
- Hazard select screen is timed — on expiry, a hazard is auto-picked at random

## Status
`ready` — game design (15 protocols, 16 hazards), technical design (interface-design.md), per-item implementation guides (31 files in protocols/ and hazards/), cross-domain messages defined, 10-wave implementation order with dependency graph. One sub-item `[NEEDS DETAIL]`: legendary retuning values (legendary-retuning.md).
