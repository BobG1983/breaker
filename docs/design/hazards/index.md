# Hazards — Design Catalog

Hazards are the negative upgrade system — stackable debuffs the player must choose from during infinite play (tier 9+). The model is **choose-your-poison**: at each post-node hazard-select screen, the player is shown 3 random hazards and MUST pick one. Selection is timed — on expiry, a hazard auto-picks at random.

The goal is bragging rights — "I got to tier 16" because by that point you've layered multiple stacking debuffs against your god-tier build. Unstoppable force meets immovable object.

## Hard Rules

Every hazard in the pool obeys these rules. Violating any is grounds for a proposal to be killed.

1. **NO disabling** chips, Bumping, dashing, or any player capability. The player always has the tools; the hazard makes the tools harder to apply.
2. **Nothing that feels cheaty** — the player should feel overwhelmed, not robbed.
3. Hazards are **escalatingly negative** — pure difficulty that stacks.
4. Every hazard must be **readable and telegraphed** — the player can see and respond.
5. Hazards must create a **new mechanical dynamic** — something to read, respond to, and decide about. If the description is just "X goes up" or "X goes down", it's not interesting enough.
6. Hazards are **problems to solve**, not handicaps to endure. Every hazard has a strategy to play around it. A master barely notices it; a novice gets crushed. The hazard is a mirror of the player's skill.

## Pool (16)

All 16 hazards live in a single flat pool — no severity tiers. Each entry links to its canonical design doc. Tuning numbers are the shipped RON targets; see each doc for the full formula.

| Hazard | One-liner | Stacking target |
|--------|-----------|-----------------|
| [Cascade](cascade.md) | Destroyed cells heal nearby cells. | +10 HP base + 5 HP/level |
| [Decay](decay.md) | Node timer drains faster. | 15% base + 5%/level |
| [Diffusion](diffusion.md) | Incoming damage shared with nearby cells (concentric rings). | 20% + 10%/level, +1 ring depth per 5 levels |
| [Drift](drift.md) | Wind pushes the bolt in a telegraphed direction, changes ~8s. | Base force + force/3 per level |
| [Echo Cells](echo_cells.md) | Destroyed cells leave a ghost after 1.5s. | 1 HP, doubling per level |
| [Erosion](erosion.md) | Breaker shrinks over time; Bumps restore width. | Shrink rate scales linearly; floor at 35% |
| [Fracture](fracture.md) | Destroyed cells spawn 1-HP debris at offsets. | 2 debris base + 1/level, capped at 4 |
| [Gravity Surge](gravity_surge.md) | Destroyed cells spawn short-lived gravity wells. | 2s + 1s/level duration; sqrt-dim strength |
| [Haste](haste.md) | Bolt speed multiplier, multiplicative with existing speed. | +20% + 10%/level |
| [Momentum](momentum.md) | Non-lethal hits give cells HP; 2× splits into 2 at 1×. | +10 HP + 10 HP/level per non-lethal hit |
| [Overcharge](overcharge.md) | Bolt accumulates speed per kill between Bumps; resets on Bump. | 5% + 3%/level per kill, multiplicative |
| [Renewal](renewal.md) | Cells regen to pristine on a countdown; timer shrinks per stack. | 10s base * (1 - 20%/level) |
| [Resonance](resonance.md) | Rapid kills fire dodgeable slow-waves at the breaker. | Window 0.5s + 0.3s/level; log-dim slow |
| [Sympathy](sympathy.md) | Damage to a cell heals nearby cells; target still takes full damage. | 25% + 5%/level, +1 ring depth per 5 levels |
| [Tether](tether.md) | Nearby cell pairs linked; damage bleeds to the partner. | 25% + 10%/level damage; 40% + 10%/level coverage |
| [Volatility](volatility.md) | Cells gain HP when not being hit; cap at 2× starting. | +1 HP per 5s, interval shrinks per stack |

## Trap Synergies (Player-Knowledge Rewards)

The best hazard designs look manageable in isolation but combine devastatingly. Experienced players learn which combinations to avoid; novices take the "safe pick" and get punished.

- **Echo Cells + Volatility** — ghosts look free (1 HP) but grow rapidly if not cleared immediately.
- **Tether + Cascade** — Tether spreads non-lethal damage to partners; Cascade heals their neighbors on kill. Damage you spread feeds the heal loop.
- **Diffusion + Sympathy** — both gain ring depth every 5 levels; at depth 2+, clusters become nearly impenetrable damage sponges.
- **Erosion + Haste + Overcharge** — catch a 2× bolt with a 40% Breaker while the bolt accelerates per kill.
- **Fracture + Momentum** — splits create empty cells = room for more Momentum splits; Volatility makes the debris grow.
- **Fracture + Volatility** — "easy cleanup" 1-HP debris becomes 3-HP debris if you don't clear it fast.
- **Decay + Renewal** — timer drains faster while cells regenerate on their own countdown. Double clock pressure.
- **Gravity Surge + Drift** — two forces on the bolt simultaneously; both readable, both demanding.
- **Overcharge + Haste** — bolt speed compounds multiplicatively between Bumps.
- **Resonance + Echo Cells** — clearing ghost clusters triggers resonance slow-waves.
- **Momentum + Diffusion** — Diffusion bleeds damage to neighbors (can't one-shot); Momentum punishes non-lethal hits with HP growth.

## Why Flat Pool (No Severity Tiers)

Severity tiers (Taxing / Punishing / Terminal) add artificial structure that doesn't serve gameplay. A flat pool with emergent synergies is more roguelite — the "obvious safe pick" is a trap if you don't know the meta. Difficulty comes from:

1. **Stack count** — more hazards = harder, period.
2. **Synergy combos** — some pairs are WAY worse together than their parts suggest.
3. **Player knowledge** — experienced players learn which combinations are deadly.

This is the Balatro model: depth through emergent interaction, not through explicit difficulty labels.

## Why Choose-Your-Poison + Stacking

The choose-your-poison + stacking model creates decisions at two levels:

1. **Immediate**: "Which of these 3 is least bad for my current build + existing hazards?"
2. **Strategic**: "This looks easy now, but what happens when I stack it with what I already have?"

Stacking is what makes deep infinite runs impressive — not variety of hazards but **intensity** of repeated ones. By tier 14+, the player's build is wild (multi-pierce, chain lightning, AoE, maxed speed). Hazards are the counterweight.

## Killed Hazard Proposals (Why Not)

Design proposals that were considered and rejected, and why:

- **Frenzy** (cells fire faster) — just a number modifier, no mechanical dynamic.
- **Barrage** (cells gain spread shot) — just more projectiles, no decision-making.
- **Dim** (brightness reduced) — visual impairment isn't fun.
- **Density** (more cells) — just a number, not a mechanic.
- **Blackout** — that's how HP already works visually.
- **Magnetism** — cheaty as hazard (now a cell type instead).
- **Turbulence** — cheaty.
- **Silence** — disables chip. HARD NO.
- **Lockdown** — disables dash. HARD NO.
- **Mirror** — inverted controls feel cheaty.
- **Echo** (angle corruption) — per-hit angle corruption, can't respond meaningfully.
- **Entropy** — erases cell type design, anti-variety.
- **Volatile Revenge** — replaced by Resonance (more interesting).
- **Warp** — feels cheaty/random; cells rearranging is unreadable.
- **Ablation** — just Volatile Revenge with extra steps.
- **Fortress** — interesting but better as a cell type.
- **Ricochet** (as hazard) — doesn't make sense; it's a protocol instead.
- **Aftershock** — hurts you for winning, punishes the core loop.
- **Surge** (bolt grows) — number modifier, no decision.
- **Undertow** (breaker drift) — fights player's direct controls.
- **Tremor** (cells shift down) — arbitrary rule change.
- **Phase Shift** (cells shift directionally) — too close to rejected Warp.
- **Backlash** (steeper angles on damaged cells) — too narrow, only one build feels it.
- **Scorch** (AoE kills burn floor) — not great.
- **Convergence** (cells drift to impact) — visual complexity concern.
- **Resonance Field** (clusters fire projectiles) — no.
- **Anchor** (slow zones from kills) — visual overlap with Gravity Surge.

## Meta-Progression

Unlike protocols, hazards do NOT grow via meta-progression — the pool is fixed at 16. Replay depth comes from stacking + synergy, not from pool expansion. Phase 7 may target 30 hazards total, but additions will be evaluated against the Hard Rules individually.

## Related

- [Protocols](../protocols/index.md) — the positive upgrade system.
- [Terminology: Core](../terminology/core.md) — Breaker, Bolt, Cell, Bump, Rebound.
- [Design pillars](../pillars/) — the frame every hazard is evaluated against.
