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

### Entropy Engine
**Ingredients**: Cascade + Flux

Counter-gated random effect — every 5th cell destroyed, roll from weighted pool. Cascade provides the trigger domain, Flux provides the randomness mechanic, evolution adds the counter gate.

### Chain Reaction
**Ingredients**: Cascade x3 + Splinter x2 + Piercing x3

Recursive bolt spawning with effect inheritance. Destroyed cells spawn temporary bolts that inherit the parent's effects — including piercing and cascading shockwaves. Creates exponential chain reactions that spread across the field.

**Design notes**: This is the evolution that most dramatically changes the feel of the game. A well-built Chain Reaction setup can clear an entire field from a single cell destruction. The inheritance mechanic means every chip the player has invested in amplifies the chain.

### Feedback Loop
**Ingredients**: TBD

3 perfect bumps trigger bolt spawn + shockwave. Counter-gated burst that rewards consistent precision over time rather than single-hit spikes.

### Nova Lance
**Ingredients**: Damage Boost x2 + Bolt Speed x2

Perfect bumps unleash devastating shockwaves on cell impact.

### Voltchain
**Ingredients**: Chain Hit x2 + Damage Boost x2

Destroying cells unleashes chain lightning to nearby targets.

### Phantom Breaker
**Ingredients**: Wide Breaker x2 + Bump Force x2

Successful bumps summon a phantom breaker that mirrors your moves.

### Supernova
**Ingredients**: Piercing Shot x3 + Surge x1

Perfect bumps trigger chain explosions — cells destroyed spawn bolts and shockwaves.

### Dead Man's Hand
**Ingredients**: Damage Boost x3 + Last Stand x1

Losing a bolt triggers a shockwave and boosts all remaining bolts.

**Redesign note (C7)**: Planned rework to Pulse effect — all bolts shockwave on bolt lost, creating a distributed damage response across the field rather than a single point-source shockwave.

### Railgun
**Ingredients**: Piercing Shot x3 + Bolt Speed x4

Perfect bumps fire a devastating piercing beam through all cells in the bolt's path.

### Gravity Well
**Ingredients**: Bolt Size x2 + Magnetism x2

Destroying cells creates gravity wells that pull bolts toward the destruction point.

### Second Wind
**Ingredients**: Wide Breaker x3 + Breaker Speed x3

Invisible wall that bounces the bolt once per node. Cheat death once.

**Redesign note (C7)**: Now uses `Once` node type instead of a custom one-shot mechanism. The invisible wall effect is placed on the breaker's `BoundEffects`, not as a global resource. Fires exactly once per node, then the `Once` node is permanently consumed from the chain.
