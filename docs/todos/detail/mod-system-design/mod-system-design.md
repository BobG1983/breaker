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
- **Effect refactor dependency**: Protocol implementation waits for todo #2. Effect-tree protocols use `ValidDef` types from the new system.

## Implementation Order

This todo is large. Suggested implementation waves:

1. **Infrastructure**: `ProtocolKind`/`HazardKind` enums, `ProtocolRegistry`/`HazardRegistry` registries, `ActiveProtocols`/`ActiveHazards` resources, plugin shells, RON loading
2. **Legendary removal + Anchor migration**: Remove `Legendary` rarity, retune 13 chips as Rare, delete Anchor evolution, create Anchor protocol RON
3. **Protocol offering integration**: `ChipOffering::Protocol` variant, `generate_chip_offerings` protocol slot, `handle_chip_input` protocol branch, `ProtocolSelected` message, protocol card rendering
4. **Effect-tree protocols**: Deadline, Ricochet Protocol, Anchor, Kickstart (depends on effect refactor — todo #2)
5. **Custom-system protocols**: Debt Collector, Echo Strike, Siphon, Fission, Burnout, Conductor, Afterimage, Reckless Dash, Greed, Iron Curtain, Tier Regression (several depend on effect refactor for `Killed` trigger; Tier Regression depends on node sequencing — todo #7)
6. **Hazard state flow**: `RunState::HazardSelect`, `HazardSelectState`, `resolve_post_chip_state` dynamic route, hazard select UI
7. **Hazard systems**: All 16 hazards (depends on node sequencing for tier 9+ — todo #7)
8. **Scenarios**: Invariant checkers + adversarial scenarios for all protocols and hazards

**Hard dependencies**:
- Waves 4-5 depend on **todo #2** (effect system refactor) for `SourceId`, `Killed(KillTarget)`, `During(Condition)`, `Route`/`Stamp`/`Transfer`. Protocol implementation waits for the refactor — `ProtocolDefinition` effect-tree variants use the new system's `ValidDef` types, not `RootEffect`.
- Waves 5 (Tier Regression), 6, 7 depend on **todo #7** (node sequencing refactor) for extended tiers and tier 9+ gating
- Waves 1-3 can proceed independently

**Design decisions**:
- Protocol offering is random from seeded `GameRng` (deterministic from run seed)
- Picking a protocol closes the chip select screen (protocol OR chip, not both)
- Hazard select screen is timed — on expiry, a hazard is auto-picked at random

## Status
`[NEEDS DETAIL]` — technical design complete, needs implementation wave breakdown into concrete specs before `/implement`
