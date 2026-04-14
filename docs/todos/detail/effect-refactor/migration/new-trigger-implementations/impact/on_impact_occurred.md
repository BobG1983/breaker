# Name
on_impact_occurred

# Reads
All 6 collision messages:
- `BoltImpactCell`
- `BoltImpactWall`
- `BoltImpactBreaker`
- `BreakerImpactCell`
- `BreakerImpactWall`
- `CellImpactWall`

# Dispatches
`ImpactOccurred(EntityKind)` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext::Impact { impactor, impactee }` — populated with both participant entities so On(Impact(Impactor)) and On(Impact(Impactee)) can resolve within global trigger trees.

Each collision fires two global sweeps (one per participant kind), both with the same context:

**BoltImpactCell:**
- Sweep 1: `ImpactOccurred(Cell)`, context: `Impact { impactor: bolt, impactee: cell }`
- Sweep 2: `ImpactOccurred(Bolt)`, context: `Impact { impactor: bolt, impactee: cell }`

**BoltImpactWall:**
- Sweep 1: `ImpactOccurred(Wall)`, context: `Impact { impactor: bolt, impactee: wall }`
- Sweep 2: `ImpactOccurred(Bolt)`, context: `Impact { impactor: bolt, impactee: wall }`

**BoltImpactBreaker:**
- Sweep 1: `ImpactOccurred(Breaker)`, context: `Impact { impactor: bolt, impactee: breaker }`
- Sweep 2: `ImpactOccurred(Bolt)`, context: `Impact { impactor: bolt, impactee: breaker }`

**BreakerImpactCell:**
- Sweep 1: `ImpactOccurred(Cell)`, context: `Impact { impactor: breaker, impactee: cell }`
- Sweep 2: `ImpactOccurred(Breaker)`, context: `Impact { impactor: breaker, impactee: cell }`

**BreakerImpactWall:**
- Sweep 1: `ImpactOccurred(Wall)`, context: `Impact { impactor: breaker, impactee: wall }`
- Sweep 2: `ImpactOccurred(Breaker)`, context: `Impact { impactor: breaker, impactee: wall }`

**CellImpactWall:**
- Sweep 1: `ImpactOccurred(Wall)`, context: `Impact { impactor: cell, impactee: wall }`
- Sweep 2: `ImpactOccurred(Cell)`, context: `Impact { impactor: cell, impactee: wall }`

# Source Location
`src/effect_v3/triggers/impact/bridges.rs`

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, with `run_if(in_state(NodeState::Playing))`.
Per-system ordering:
- `on_impact_occurred_bolt_cell` after `BoltSystems::CellCollision`
- `on_impact_occurred_bolt_wall` after `BoltSystems::WallCollision`
- `on_impact_occurred_bolt_breaker` after `BoltSystems::BreakerCollision`

# Behavior
Implemented as 6 separate systems (one per collision message type). Each system:

1. Read each collision message of its type.
2. Build `TriggerContext::Impact { impactor, impactee }` from the message fields.
3. Walk all entities with BoundEffects/StagedEffects with `ImpactOccurred(impactee_kind)` and the context.
4. Walk all entities again with `ImpactOccurred(impactor_kind)` and the same context.

Each bridge does NOT:
- Modify any entities
- Send any messages
- Handle game logic or damage calculation
- Skip triggers based on entity state
