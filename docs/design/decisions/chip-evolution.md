# Chip Evolution

**Decision**: Two chips at minimum stacks + boss kill = evolved chip.

## Model

- Evolution recipes are pre-defined: Chip A + Chip B -> Evolved Chip C
- Both ingredient chips must be at a minimum stack threshold (defined per recipe in RON)
- Evolving consumes both ingredient chips and replaces them with the evolved form
- Evolution is offered as a boss node reward — beat the boss, and if you qualify for any evolution, you choose one
- If you don't qualify for any evolution, the boss offers alternative rewards (chips, stat boosts, etc.)

## Scope

Phase 4 (vertical slice) targets 3-4 evolutions across mixed chip combinations:
- Passive + Passive -> evolved chip
- Passive + Triggered -> evolved chip (cross-type)

This proves the architecture handles same-type and cross-type evolutions.

## Rationale

- **Knowledge-gated power** — players who know the recipes can plan builds around them (Pillar 7: Discovery is the Long Game)
- **Investment required** — both chips need minimum stacks, preventing cheap/accidental evolutions
- **Boss node purpose** — gives boss fights a reward beyond "gate to next tier" (Pillar 1: The Escalation)
- **Inspired by Vampire Survivors** evolution system, but requires stacking investment rather than just maxing a single weapon

## Design Space

Evolution recipes create a discovery layer: players experiment to find which chip pairs evolve, and plan runs around recipes they know. The wiki effect — community knowledge-sharing — is a longevity multiplier.

## Evolution Recipes

### Shock Chain
**Ingredients**: Chain Reaction x1 + Aftershock x2 + Cascade x2

**Design notes**: Destroyed cells trigger recursive shockwaves — each shockwave kill spawns another shockwave. Combines the Chain Reaction template's destruction-chaining with Aftershock's area damage and Cascade's propagation.

**Effect**: `On(Bolt) → When(CellDestroyed) → Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))`

### Feedback Loop
**Ingredients**: TBD

**Design notes**: Counter-gated burst. Every 3rd perfect bump fires a bolt swarm + shockwave. Rewards consistent precision over time rather than single-hit spikes. Uses `EntropyEngine` internally as a counter mechanism with deterministic output instead of random selection.

**Effect**: `When(OnPerfectBump, [Do(EntropyEngine(3, [(0.5, Do(SpawnBolts { count: 2 })), (0.5, Do(Shockwave(base_range: 96.0, ...)))]))])`

See `docs/design/chip-catalog.md` for the full evolution catalog and `docs/design/evolutions.md` for design principles.
