# Protocol Decisions — Designer Review

Tracks decisions from the designer review session. This is the source of truth; agent brainstorms are inputs, not outputs.

## The Chip/Protocol Distinction

- **Chips** are **power**. Definite buffs — static or effect-driven. They don't change how you WANT to play. They synergize with evolutions, each other, and protocols. Chips make you stronger within your current playstyle.
- **Protocols** are **rule changes**. They fundamentally change how you WANT to play. Without the protocol, you play normally (dash sometimes, always perfect bump if possible). With it, you deliberately do something non-standard.
- **The test**: Does this make me play differently, or just play the same way but stronger?

## Legendary Rarity Removal

Delete the Legendary rarity entirely. Only Common, Uncommon, Rare, and Evolution remain.

### All 11 Legendaries → Rework as Rare Chips
Glass Cannon, Desperation, Whiplash, Singularity, Gauntlet, Chain Reaction, Feedback Loop, Parry, Powder Keg, Death Lightning, Tempo — all are POWER (they reward what you're already doing). Retune numbers downward for Rare tier. Remove `legendary:` blocks from RON files, add as standard chip entries with `rarity: Rare`. Update `max_taken` as appropriate.

### Anchor Evolution → Promote to Protocol
Anchor (Quick Stop x2 + Bump Force x2) promoted to protocol. Delete the evolution entry. As a protocol: plant breaker by standing still → better bumps + Piercing.

### Evolution Catalog (Phase 7)
Not all chip combinations need an evolution. Current evolutions are test placeholders. Phase 7 scope: replace all evolutions, design final chip catalog. Note this in Phase 7 detail file.

---

## Final Protocol Roster (15)

### 1. Deadline
- **Behavior change**: You WANT to slow-play 75% of the node, then explode in the danger zone.
- **Mechanic**: When node timer drops below 25%, all bolts get 2x speed + 2x damage until node ends.
- **Origin**: Promoted from legendary chip.

### 2. Ricochet Protocol
- **Behavior change**: You WANT to aim for walls, not cells.
- **Mechanic**: After wall bounce, bolt deals 3x damage until next cell impact.
- **Origin**: Promoted from legendary chip.

### 3. Fission
- **Behavior change**: You WANT to maximize destruction volume for bolt splits.
- **Mechanic**: Every 8th cell destroyed permanently splits one bolt into two. New bolt inherits parent's effects. Persists across nodes.
- **Origin**: R1 brainstorm.

### 4. Kickstart
- **Behavior change**: You WANT to optimize explosive openers.
- **Mechanic**: Node starts with 3s of 2x bolt speed, Piercing(2), and 2x damage. Timer starts on first bump.
- **Origin**: R1 brainstorm.

### 5. Tier Regression
- **Behavior change**: You WANT to retreat instead of advance.
- **Mechanic**: Drop back 1 tier of difficulty. Replay easier tier's nodes for extra chip offerings. Can only appear once per run. In infinite mode: regresses difficulty but keeps hazard stack.
- **Origin**: R1 brainstorm.

### 6. Debt Collector
- **Behavior change**: You WANT to deliberately do Early/Late bumps to build a damage multiplier, then cash out with a Perfect bump.
- **Mechanic**:
  - Early or Late bump: stack += 0.5x multiplier
  - Perfect bump: next cell impact deals `normal damage * (1 + stack)`. Stack resets to 0.
  - Stack does NOT persist across nodes.
  - Stack IS lost on bolt-lost (punishment for losing the bolt you were building on).
  - Scales multiplicatively with damage chips.
- **Origin**: R1 brainstorm, revised from whiff-stacking (mechanic abuse) to bump-grade stacking.

### 7. Iron Curtain
- **Behavior change**: Bolt-lost becomes an offensive event, not just a penalty.
- **Mechanic**:
  - On bolt-lost: damage wave spreads upward from breaker position across the full playfield.
  - Wave damage = 0.5x bolt base damage at origin.
  - Linear falloff with distance from breaker (small "no falloff" window near breaker so close hits feel impactful).
  - Wave dims visually through its life.
  - Does NOT prevent bolt-lost penalties (life loss, time penalty, etc.).
- **Origin**: R1 brainstorm, revised from shield wall to directional damage pulse.

### 8. Echo Strike
- **Behavior change**: You WANT to Perfect Bump into specific cells to build an echo network, then hit them all simultaneously.
- **Mechanic**:
  - Perfect Bump → cell impact: that cell becomes an echo (max 3 echoes, FIFO).
  - On subsequent Perfect Bump → cell impact: damages the current target AND all active echoes.
  - Echo damage falloff by age: newest ~50%, middle ~25%, oldest ~10% of the impact damage.
  - Echoes cleared on node end.
- **Origin**: R1 brainstorm, revised from position-recording to cell-recording.

### 9. Siphon
- **Behavior change**: You WANT to set up multi-kill chains to farm time off the clock.
- **Mechanic**:
  - Kill a cell → 2s streak window starts.
  - Each subsequent kill within the window: +0.5s added to node timer. Window resets to 2s.
  - First kill starts the streak but adds no time.
  - 2s without a kill → streak ends.
  - All kill sources count (bolt, AoE, chain lightning, explosions).
  - Values (window duration, time per kill) are tunable.
- **Origin**: R1 brainstorm, revised from AoE-only to kill streak mechanic.

### 10. Greed
- **Behavior change**: You WANT to skip chip offerings, gambling on better chips later.
- **Mechanic**:
  - On chip offering screen: option to skip (take no chip).
  - Each skip increases the chance of higher rarity chips in future offerings (e.g., +5% rarity boost per skip, tunable).
  - No immediate power gain. Pure gamble — trading certain power now for uncertain better power later.
  - Stacks per skip.
- **Origin**: R2, clarification agent's Greed concept refined.

### 11. Reckless Dash
- **Behavior change**: You WANT to be out of position and stretch your dash to the limit for a risky catch.
- **Mechanic**:
  - "Risky catch" = bolt contacts breaker in the last 10-30% of dash distance (you had to stretch to reach it — weren't in position).
  - Risky catch → next cell impact deals 4x damage.
  - If bolt is lost during a dash → bolt loss triggers twice (double penalty).
  - Non-risky catch (bolt contacts in early/mid dash, or while stationary) → normal bump, no bonus, no penalty.
  - Telegraph: TBD — needs visual indicator when bolt is in the "risky zone" near bottom.
- **Origin**: R2, refined from "constant dashing" concept.

### 12. Burnout
- **Behavior change**: You WANT to alternate frantic movement and deliberate stillness.
- **Mechanic**:
  - Heat gauge fills while moving (4s to full), drains while stationary (2s to empty).
  - Full heat: next bump deals 4x damage + fires a shockwave.
  - Standing still for 1.5s: instant full drain + 2s breaker speed boost.
  - Rhythm: move → build heat → stop → drain → speed boost → move → mega-bump.
- **Origin**: R2, clarification agent.

### 13. Anchor
- **Behavior change**: You WANT to commit to a position and predict where the bolt will be, rather than chasing reactively.
- **Mechanic**:
  - Stand still → plant after brief delay → planted state grants better bump force, wider perfect window, + Piercing.
  - Start moving → unplanted.
  - Exact values TBD (current evolution values: 0.3s plant delay, 2x bump force, 1.5x perfect window).
  - Rival to Burnout (both reward stillness, different patterns: positional commitment vs heat rhythm).
- **Origin**: Promoted from Anchor evolution. Delete evolution entry.

### 14. Conductor
- **Behavior change**: You WANT to catch the RIGHT bolt, not just any bolt. Multi-bolt target selection.
- **Mechanic**:
  - With multiple bolts, only the "Conducted" (primary) bolt gets your chip effects.
  - Perfect Bump a bolt → that bolt becomes Conducted (primary).
  - Non-primary bolt-loss doesn't count as bolt-lost.
  - Edge case: perfect bump bolt A while bolt B (current primary) is about to be lost → A becomes primary → B's loss doesn't trigger bolt-lost.
  - Pointless with 1 bolt — changes how multi-bolt plays.
- **Origin**: R3 brainstorm.

### 15. Afterimage
- **Behavior change**: You WANT to dash AWAY from where the bolt will be, not toward it.
- **Mechanic**:
  - Dash → phantom breaker appears at your starting position for 2s.
  - Bolt bounces off phantom with normal rebound physics.
  - Perfect Bump during a phantom bounce → bolt becomes Phantom for a limited duration.
  - Phantom bolt passes through cells (damaging them) instead of bouncing off. Still bounces off walls and real breaker normally.
  - When duration expires, bolt returns to normal.
  - Duration does NOT reset if triggered again while already Phantom — must wait for it to return to normal first.
- **Origin**: R3 brainstorm, refined with phantom bolt mechanic.

---

## Declined Protocols

| Name | Reason |
|------|--------|
| Convergence Engine | Moving cells off the grid feels weird |
| Undertow | Bolt reflections should be predictable, feels like a debuff |
| Bloodline | Wildly overpowered, outpaces HP growth over a run |
| Gravity Lens | Too similar to Gravity Well |
| Overclock Protocol | Boring stat increases |
| Overburn | POWER not rule change (bolt damage aura doesn't change how you play) |
| Polarity | No synergies with existing chip catalog |
| Momentum Flip | Either inverts controls (bad) or rewards what you already do (not a protocol) |
| Triage | Too complicated |
| Flashpoint | Too complicated |
| Gauntlet Run | Doesn't really change how you want to play |
| Siege | Changes how you play but invalidates bolt (an existing core mechanic) |
| Scavenger | Cell modifiers don't make sense on a bolt |
| Harvest | Mobile game mechanic |

---

## Implementation Constraint: RON Tunable + Hot Reloadable
ALL tuning values in every protocol AND every hazard must be RON-configurable and hot-reloadable. No hardcoded numbers — every threshold, multiplier, duration, window, and percentage loads from RON so we can twiddle values without recompiling. This applies to: damage multipliers, stack values, timer durations, streak windows, falloff curves, heat rates, plant delays, rarity boost amounts, distance thresholds, etc.

## Graphics Catalog Note
All protocol and hazard effects that need visual representation MUST be documented in the rendering catalog (`docs/design/graphics/catalog/`) so Phase 5 doesn't miss them. This includes: Debt Collector stack glow, Echo Strike echo markers, Iron Curtain damage wave, Siphon kill streak display, Burnout heat gauge, Anchor plant state, Conductor primary bolt glow, Afterimage phantom breaker + phantom bolt, and all 16 hazard visual effects.

## Open Questions

- Reckless Dash: telegraph design (visual indicator for bolt in risky zone)
- Anchor: exact values as protocol (plant delay, bump force multiplier, perfect window multiplier, piercing amount)
- Afterimage: phantom bolt duration
- Conductor: what happens to non-primary bolt chip effects mid-flight when primary swaps?
- Siphon: final tuning on streak window (2s? 1.5s?) and time per kill (0.5s? 0.3s?)
- Greed: rarity boost amount per skip
