# Evolution Design Principles

Evolutions are the payoff for maxing out specific chip combinations. They represent the pinnacle of the build-crafting system — the moment a collection of incremental upgrades transforms into something dramatically more powerful.

## What Makes a Good Evolution

### Ultimate Ability Feel

An evolution should feel like unlocking an ultimate ability, not applying another stat buff. The player should notice an immediate, dramatic change in how the game plays. If you can't put big VFX on it, it's not an evolution — it's a chip.

**Good**: OnPerfectBump — bolt permanently splits into 2 bolts with piercing and new vfx. Every perfect bump now produces a visible, lasting power escalation.

**Bad**: OnPerfectBump — SpeedBoost. This feels like an amp with extra steps.

### VFX-Worthy

Every evolution must have a clear, screen-readable visual moment when it triggers. Chain lightning arcing between cells, a burst of new bolts, a shockwave ripple, an energy explosion — these are evolution-tier effects. If the effect is invisible (stat bonus, timer adjustment), it belongs in the amp/augment system, not evolutions.

### Permanent or Dramatic

Evolution effects should either be permanent (bolt split stays for the rest of the run) or dramatically impactful when they fire (3 piercing bolts that explode on contact). Temporary minor buffs belong in the passive chip system, and weak triggered effects belong as triggered chips.

### Not a Stat Buff

Evolutions must introduce new interaction points — new ways for the player to engage with the game mechanics. "More damage" is not an evolution. "Destroying 5 cells triggers chain lightning" IS an evolution because it creates a new micro-goal (cluster your cell destruction) and a new spectacle moment.

## Pacing Constraint

At least 1 evolution (the lowest-powered one) must be achievable by the first boss. This means two chips that can be maxed out within the first 4 levels. The early evolution does not need to be balanced against late-game evolutions — it's fine for it to be weaker. Its purpose is to teach the player that evolutions exist and are worth pursuing.

## Design-First: Evolutions Drive Chips

Work backwards from the evolution fantasy to the component chips. If a cool evolution needs base chips that don't exist yet, create those chips. The evolution experience is the goal; chips are the building blocks.

Example: "Chain Lightning" evolution needs a chip that increases damage in an area and a chip that chains effects between cells. If those chips don't exist, design them to enable the evolution.

## Categories

| Category | Description | Examples |
|----------|-------------|---------|
| **Offensive** | Creates new damage sources or dramatically amplifies existing ones | Chain lightning, multi-bolt burst, piercing explosion |
| **Defensive** | Prevents or mitigates bolt loss in dramatic ways | Bolt respawn from random cell, temporary invincibility shield |
| **Utility** | Changes fundamental game mechanics | Permanent bolt split, bolt magnetism to cells |

## Damage Attribution

Each evolution tracks cumulative damage dealt across the run. At run-end, the evolution with the highest total damage is displayed as the "Most Powerful Evolution" highlight. This includes:
- Direct damage from bolts spawned by the evolution
- Area damage from evolution-triggered shockwaves
- Any other cell destruction attributable to the evolution's effects

## Integration with EffectNode

Evolutions are implemented as chips with `EffectNode` effect trees. The chip's effect nodes are pushed to the entity's `BoundEffects` when selected and evaluated by bridge systems on matching game events. Each evolution carries its chip name through the evaluation pipeline for damage attribution.

## Evolution Catalog

### Nova Lance
**Ingredients**: Impact x2 + Bolt Speed x3

Perfect bumps unleash a devastating piercing beam through all cells in the bolt's path. Beam appears at max width, shrinks over time.

### Voltchain
**Ingredients**: Chain Reaction x1 + Aftershock x2

Destroying cells unleashes enhanced chain lightning (6 arcs, large max jumps) to nearby targets.

### Phantom Bolt
**Ingredients**: Wide Breaker x2 + Bump Force x2

Successful bumps summon spectral phantom bolts (up to 5 active). Ghost bolts with translucent/phasing visual and afterimage trail.

### Supernova
**Ingredients**: Piercing Shot x3 + Surge x1

Perfect bumps trigger chain explosions — cells destroyed spawn inheriting bolts and shockwaves. Cascade spectacle emerges from overlapping base effects.

### Dead Man's Hand
**Ingredients**: Damage Boost x3 + Last Stand x1

Losing a bolt triggers a shockwave and boosts all remaining bolts. Mechanic rework pending — current payoff is underwhelming for an evolution.

### Gravity Well
**Ingredients**: Bolt Size x2 + Magnetism x2

Destroying cells creates gravity wells (up to 4 active, 5s duration, 160 radius) that pull bolts toward the destruction point.

### Second Wind
**Ingredients**: Wide Breaker x2 + Last Stand x1

Invisible wall that bounces the bolt once per node. Cheat death once.

### ArcWelder
**Ingredients**: Tether x2 + Amp x2

All active bolts connected by crackling neon beams in sequence (1→2→3→4). Beams damage everything they intersect. Chain repairs when bolts are lost.

### Entropy Engine
**Ingredients**: Cascade x2 + Flux x2

Random effect from weighted pool fires on every cell destroyed. Prismatic flash per trigger, bolt gains persistent shimmer.

### Split Decision
**Ingredients**: Splinter x2 + Piercing Shot x2

Destroyed cells spawn 2 permanent inheriting bolts. Fission visual — cell splits into bolt orbs.

### Shock Chain
**Ingredients**: Chain Reaction x1 + Aftershock x2 + Cascade x2

Destroyed cells trigger recursive shockwaves — each shockwave kill spawns another shockwave. Escalating intensity per generation depth.

### Circuit Breaker
**Ingredients**: Feedback Loop x1 + Bump Force x2

3 perfect bumps charge a counter. On completion: spawn bolt + large shockwave, reset. Triangle charge indicator near bolt.

### Mirror Protocol
**Ingredients**: Reflex x1 + Piercing Shot x2

Bolt impacts spawn a mirrored bolt at the geometric reflection point. Mirror axis depends on which side was hit (top/bottom flips X, left/right flips Y). Inherits the source bolt's effects.

### Anchor
**Ingredients**: Quick Stop x2 + Bump Force x2

Plant mechanic — while braking/stopped, bump force doubled and perfect window widened. Creates dash→plant→bump→dash rhythm.

### FlashStep
**Ingredients**: Breaker Speed x2 + Reflex x1

Dash reversal during settling becomes a teleport — breaker disintegrates and rematerializes instantly. Skips settling penalty.

### Resonance Cascade
**Ingredients**: Pulse x2 + Bolt Size x2

Bolt constantly emits damage pulses at a fixed interval (no trigger needed). Larger bolt = larger pulse radius. Changes the verb from "aim" to "navigate."
