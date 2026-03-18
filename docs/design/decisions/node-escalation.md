# Node Escalation & Difficulty

**Decision**: Procedural node sequence from seed with tier-based escalation.

## Model

Runs progress through tiers of increasing difficulty. Each tier is a set of nodes followed by a boss:

- **Tier 1**: 5 passive nodes -> boss node
- **Tier 2**: 5 tougher nodes (some active) -> boss node
- **Tier 3**: Active nodes mixed in more frequently -> boss node
- **Tier N**: Eventually all active nodes -> boss node

## Escalation Axes

Difficulty increases along multiple axes simultaneously:

1. **Node type mix**: Passive -> Active -> Boss ratio shifts toward harder types
2. **Cell mechanics**: Simple cells early, Lock Cells / Tough Cells / Portal Cells later
3. **Timer compression**: Timer drops by a fraction after each boss kill
4. **Cell HP scaling**: Cells get tougher (more HP) as tiers progress

## Data Structure

- Node layouts defined in RON per node type (passive, active, boss)
- Tier escalation parameters defined in a difficulty curve RON
- Procedural sequence generated from run seed + difficulty params
- Same seed = same node sequence

## Rationale

Difficulty comes from mechanics, not just stats. "Tougher" means new cell types with new behaviors, not just more HP. The tier/boss cadence creates natural pacing milestones (Pillar 1: The Escalation) and boss kills are evolution opportunities (see chip-evolution.md).
