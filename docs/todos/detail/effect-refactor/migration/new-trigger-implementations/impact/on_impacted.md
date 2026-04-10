# Name
on_impacted

# Reads
All 6 collision messages:
- `BoltImpactCell`
- `BoltImpactWall`
- `BoltImpactBreaker`
- `BreakerImpactCell`
- `BreakerImpactWall`
- `CellImpactWall`

# Dispatches
`Impacted(EntityKind)` trigger variant

# Scope
Local — walks only the two participant entities from each collision message.

# TriggerContext
`TriggerContext::Impact { impactor, impactee }` — the impactor is the entity that initiated the collision, the impactee is the entity that was hit.

**BoltImpactCell:**
- Walk bolt with `Impacted(Cell)`, context: `Impact { impactor: bolt, impactee: cell }`
- Walk cell with `Impacted(Bolt)`, context: `Impact { impactor: bolt, impactee: cell }`

**BoltImpactWall:**
- Walk bolt with `Impacted(Wall)`, context: `Impact { impactor: bolt, impactee: wall }`
- Walk wall with `Impacted(Bolt)`, context: `Impact { impactor: bolt, impactee: wall }`

**BoltImpactBreaker:**
- Walk bolt with `Impacted(Breaker)`, context: `Impact { impactor: bolt, impactee: breaker }`
- Walk breaker with `Impacted(Bolt)`, context: `Impact { impactor: bolt, impactee: breaker }`

**BreakerImpactCell:**
- Walk breaker with `Impacted(Cell)`, context: `Impact { impactor: breaker, impactee: cell }`
- Walk cell with `Impacted(Breaker)`, context: `Impact { impactor: breaker, impactee: cell }`

**BreakerImpactWall:**
- Walk breaker with `Impacted(Wall)`, context: `Impact { impactor: breaker, impactee: wall }`
- Walk wall with `Impacted(Breaker)`, context: `Impact { impactor: breaker, impactee: wall }`

**CellImpactWall:**
- Walk cell with `Impacted(Wall)`, context: `Impact { impactor: cell, impactee: wall }`
- Walk wall with `Impacted(Cell)`, context: `Impact { impactor: cell, impactee: wall }`

Both participants receive the same context — On(Impact(Impactor)) resolves to the impactor entity regardless of which participant is being walked.

# Source Location
`src/effect/bridges/impact.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, with `run_if(in_state(NodeState::Playing))`.
Per-system ordering:
- `on_impacted_bolt_cell` after `BoltSystems::CellCollision`
- `on_impacted_bolt_wall` after `BoltSystems::WallCollision`
- `on_impacted_bolt_breaker` after `BoltSystems::BreakerCollision`

# Behavior
Implemented as 6 separate systems (one per collision message type). Each system:

1. Read each collision message of its type.
2. Build `TriggerContext::Impact { impactor, impactee }` from the message fields.
3. Walk impactor with `Impacted(impactee_kind)` and the context.
4. Walk impactee with `Impacted(impactor_kind)` and the same context.

Each bridge does NOT:
- Modify any entities
- Send any messages
- Walk entities not involved in the collision
- Handle game logic or damage calculation
