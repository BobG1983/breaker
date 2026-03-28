---
name: Arcwelder Evolution Design
description: TetherBeam evolution — free-moving bolts connected by damage-dealing energy beam
type: project
---

## Decision: Arcwelder (ChainBolt Evolution)

**Why:** ChainBolt tether constrains bolt movement. The evolution removes the constraint, adds persistent beam damage between bolts. Creates geometry-as-weapon — bolt positions define a damage line.

**New effect: TetherBeam**
- Parameters: damage_per_tick (3.0), beam_width (16.0), tick_rate (0.1s)
- fire(): Spawn free bolt (no DistanceConstraint), create TetherBeam component. System raycasts between both bolt positions, damages intersecting cells.
- reverse(): Despawn tethered bolt and beam.

**Evolution: "Arcwelder"**
- Trigger: PerfectBumped -> Impacted(Cell) -> Do(TetherBeam)
- Ingredients: Tether x2 + Piercing Shot x2
- Visual: neon electric arc crackling between bolts, sparks on cell contact

**How to apply:**
- Skill ceiling: master positions both bolts to maximize beam diagonal coverage across cell clusters
- Risk: losing either bolt kills the beam; two bolts to track with no tether safety
- Key synergies: DamageBoost (scale beam damage), Magnetism (sweep through clusters), SpeedBoost (wider sweep), Cascade (beam kills trigger shockwaves)
- Note: "Voltchain" name already taken by Chain Hit + Damage Boost evolution
