# Protocol & Hazard System Design

## Summary
Design the protocol (positive) and hazard (negative) systems — per-tier upgrades and stackable debuffs that shape run identity and infinite scaling.

## Context
Protocols and hazards are a new upgrade category distinct from chips. Protocols provide powerful, run-altering positive effects (Balatro coupon analog). Hazards are stackable debuffs that the player chooses from during infinite play (choose-your-poison model). The goal is bragging rights — "I got to tier 16" because by that point you've layered on multiple stacking difficulties against your god-tier build.

## Design

### Protocols (Positive)
- Each protocol can only appear **once** per run
- Only **1 protocol** is offered per tier
- Player picks a protocol **instead of a chip** — it's an opportunity cost trade (you give up a chip slot to take the protocol)
- Possibly costs something to buy (currency TBD)
- Once picked, remains visible but greyed out and crossed out
- Should interact with chip system for synergy possibilities

### Hazards (Negative — Tier 9+)
Starting around tier 9 (infinite play — name TBD, "infinity mode" placeholder), the player is shown **3 random hazards** and **must pick one** (choose-your-poison). Hazard selection happens on its own dedicated UI screen **after** the chip/protocol selection screen, only at tier 9+.

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

### Hazard Start
Hazards start at **tier 9** — right after completing the 8-tier structured run. This is the "infinity mode" boundary.

### Meta-Progression
Protocol pool grows via **meta-progression** across runs. Players unlock new protocols over time (like unlocking new Jokers in Balatro). This gives long-term progression beyond a single run.

### Protocol Design Approach
- Target **15 protocols initially** (to match 16 hazards), **30 each by Phase 7** (content & variety phase)
- Source inspiration from **current legendary chips** — legendaries may be better as protocols than chips (one-time, run-altering, powerful)
- Review the killed hazard proposals for ideas that were "too positive" or "better as a different category"
- Use **guard-game-design** agent for brainstorm generation — feed it the hazard design philosophy, the killed-hazard reasons, and the existing legendary chips
- Protocols are **code-implemented systems** (like hazards), with RON-tuned values

### Content Targets
| Category | Initial (this todo) | Phase 7 target |
|----------|-------------------|----------------|
| Protocols | 15 | 30 |
| Hazards | 16 (designed) | 30 |

## Scope
- In: protocol offering logic, hazard choose-your-poison UI (pick from 3 random), hazard stacking, hazard systems (code-implemented, RON-tuned), UI for protocols on chip select screen, protocol/hazard state tracking, meta-progression for protocol unlocks
- Out: chip system changes (chips are separate), tier system (separate todo)

## Dependencies
- Depends on: node sequencing refactor (tiers must exist)
- Depends on: chip selection UI (protocols display on chip select screen), hazard selection UI (separate screen after chip selection, tier 9+ only)
- Related: tier regression mechanic — likely a protocol (it's one-time, run-altering)

## Open Questions
- **Protocol designs**: need brainstorm session with guard-game-design. Check legendary chips as source. Generate 15 to start.
- Erosion shrink rate exact number
- Volatility growth rate floor

## Status
`[NEEDS DETAIL]` — hazard system fully designed, protocol designs need brainstorm session (use `/todo research 6` to run guard-game-design)
