---
name: Wave 3 TetherBeam Chain + SpawnBolts Inherit Evaluation
description: Chain mode approved with spawn rate governor recommendation; inherit fix approved as correctness fix with no current gameplay change. Together they define the bolt-proliferation build archetype.
type: project
---

## Wave 3 Evaluation — 2026-03-30

### TetherBeam Chain Mode: APPROVED

Connects all active bolts in sequence with damaging beams. N bolts = N-1 beams. Excellent design:
- Creates new skill axis (indirect beam positioning via bolt trajectories)
- Rewards bolt-count builds with area damage scaling
- Tension dynamic: more bolts = more damage BUT more to track/lose
- Rich synergy web: Split Decision, Supernova, Reflex, Chain Reaction, Entropy Engine all feed it

**Flag: Split Decision + ArcWelder cascade risk.** Every cell kill spawns 2 bolts, new beams damage more cells, positive feedback loop. Recommend spawn rate governor (cap bolt spawns from effects at 4-6/s) to prevent single-frame cascade. Preserves fantasy, adds visual readability.

### SpawnBolts Inherit Fix: APPROVED

Spawned bolts now inherit from primary bolt, not fire entity. Pure correctness fix:
- No current gameplay change (all `inherit: true` chips are already bolt-bound)
- Forward-looking: prevents breaker-bound SpawnBolts from producing bolts with nonsensical breaker effects
- Strengthens synergy web: `inherit: true` becomes reliable build multiplier
- Affected chips: Supernova (no change), Split Decision (no change), Desperation (not affected — no SpawnBolts)

### Combined Impact: Bolt-Proliferation Archetype

These two changes together enable a build path: stack bolt-spawning chips + bolt-enhancing chips + ArcWelder. Spawned bolts carry your build forward (inherit fix), bolt count converts to area damage (chain mode). Peak Pillar 2.

### Remaining Blockers

1. **Spawn rate governor** for chain mode cascade (new — from this evaluation)
2. **BASE_BOLT_DAMAGE hardcoding** (existing — now more urgent because bolt-proliferation archetype needs beam damage to scale with EffectiveDamageMultiplier)

**Why:** Documents design approval for two Wave 3 mechanics and their interaction. Identifies the bolt-proliferation archetype as a validated build path.
**How to apply:** Reference spawn rate governor recommendation when implementing chain mode. Check BASE_BOLT_DAMAGE status before considering beam damage balanced.
