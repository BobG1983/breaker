---
name: Protocol Brainstorm Evaluation
description: 15 protocol designs generated for the mod system; 2 legendaries promoted (Deadline, Ricochet Protocol), 11 kept as chips; design archetypes and hazard counter-play mapped
type: project
---

## Protocol Brainstorm -- 15 Designs

Generated 15 protocols for the protocol & hazard system. Written to `docs/todos/detail/mod-system-design/research/protocol-brainstorm.md`.

### The 15 Protocols
1. Deadline (promoted from legendary) -- timer pressure = bolt power
2. Ricochet Protocol (promoted from legendary) -- wall-bank 3x damage
3. Convergence Engine -- cell kills pull survivors toward impact
4. Undertow -- delayed bolt Y-inversion creates rapid bump rhythm
5. Overburn -- bolt aura damages nearby cells along path
6. Debt Collector -- whiffs/bolt-loss stack damage for next hit
7. Bloodline -- bolt gains permanent damage from killed cells' HP
8. Fission -- every 8th cell destroyed splits a bolt
9. Iron Curtain -- bolt-lost spawns shield wall + damages all cells
10. Kickstart -- 3s of 2x speed + 2x damage at node start
11. Echo Strike -- perfect bumps record positions, cell impact replays phantom damage
12. Gravity Lens -- bolt curves toward dense cell clusters, scales with speed
13. Siphon -- AoE kills add time to node timer
14. Overclock Protocol -- stacking speed+size on perfect bump, whiff resets
15. Tier Regression -- drop back 1 tier for extra chip offerings

### Key Design Decisions
- **2 legendaries promoted**: Deadline and Ricochet Protocol become protocols (run-defining effects that change strategy, not just stats)
- **11 legendaries kept as chips**: Glass Cannon, Desperation, Whiplash, Singularity, Gauntlet, Chain Reaction, Feedback Loop, Parry, Powder Keg, Death Lightning, Tempo
- **Tempo vs Overclock Protocol**: Tempo stays as chip; Protocol 14 adds the size dimension that elevates the concept to protocol-worthy
- **Every protocol has at least one hazard synergy and one hazard anti-synergy** -- mapped in the document
- **Breaker preferences mapped** -- each protocol has best/worst breaker pairings

### Protocol Archetypes
Timer Manipulators (Deadline, Siphon, Kickstart), Angle Masters (Ricochet, Gravity Lens), Controlled Chaos (Convergence, Fission), Failure Alchemists (Debt Collector, Iron Curtain), Rhythm Players (Undertow, Overclock Protocol), Scaling Engines (Bloodline, Echo Strike), Strategic (Tier Regression)

**Why:** Protocols are the Balatro coupon/joker analog -- one per run, replaces a chip pick, must change HOW you play. These 15 designs cover distinct playstyle archetypes while synergizing with the existing chip catalog and counterplaying the 16 hazards.

**How to apply:** These are design proposals. The designer picks which to implement first. Tuning values are targets, not final. Each protocol needs implementation specs and testing.
