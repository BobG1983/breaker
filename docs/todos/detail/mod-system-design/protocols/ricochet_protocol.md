# Protocol: Ricochet Protocol

## Category
`effect-tree`

## Game Design

**Behavior change**: You WANT to aim for walls, not cells.

**Mechanic**: After wall bounce, bolt deals 3x damage until next cell impact.

**Origin**: Promoted from legendary chip.

## Config Resource
None — tuning values are in the effect tree RON.

## Components
None. All effects are dispatched through the effect system onto existing bolt components. The "until next cell impact" scoping is handled by the new effect system's `Until` / duration primitives.

## Messages
**Reads (existing)**:
- `BoltImpactWall` — the effect system's wall impact bridge fires, activating the damage boost
- `BoltImpactCell` — the effect system's cell impact bridge fires, consuming the boost

**Sends**: None.

## Systems
No runtime systems — effects dispatched through the effect system.

The effect tree is installed on the breaker entity at protocol activation time via `dispatch_protocol_selection`. The effect system's existing impact trigger bridges evaluate the tree. The `Until(Impact(Cell), ...)` scoping ensures the damage boost is consumed on the next cell hit. Effect cleanup happens via `SourceId`-based removal at run end.

## Effect Tree

The RON effect tree uses the new effect system's primitives (todo #2 dependency). Conceptual structure:

```ron
// assets/protocols/ricochet_protocol.protocol.ron
(
    name: "Ricochet Protocol",
    description: "After bouncing off a wall, deal 3x damage until hitting a cell.",
    unlock_tier: 0,
    tuning: RicochetProtocol(
        effects: [
            Route(Bolt, When(Impacted(Wall), [
                Until(Impacted(Cell), [
                    Do(DamageBoost(factor: 3.0)),
                ]),
            ])),
        ],
    ),
)
```

**Notes**:
- `Route(Bolt, ...)` targets the bolt that bounced off the wall.
- `When(Impacted(Wall), ...)` gates on per-bolt wall impact (targeted trigger, not global).
- `Until(Impacted(Cell), ...)` applies the damage boost and removes it when the bolt hits a cell.
- The 3x damage boost is multiplicative with existing chip damage boosts.
- Re-fires every wall bounce — each new wall hit restarts the cycle.
- Exact syntax depends on the new effect system (todo #2). The above is design intent.

## Cross-Domain Dependencies

| Domain | Resource/Message | Access |
|--------|-----------------|--------|
| `bolt` | Bolt entities (damage components) | Effect system writes via `DamageBoost` |
| `bolt` / `wall` | `BoltImpactWall` message | Effect system's wall impact bridge reads |
| `bolt` / `cells` | `BoltImpactCell` message | Effect system's cell impact bridge reads (consumes the boost) |
| `effect` (new) | `dispatch_effects`, `SourceId` cleanup, `Until` scoping | Effect tree installation, scoped application, and removal |

## Expected Behaviors (for test specs)

1. **3x damage on first cell hit after wall bounce**
   - Given: Bolt with base damage 10.0, Ricochet Protocol active
   - When: Bolt bounces off wall, then impacts a cell
   - Then: Cell receives 30.0 damage (3x)

2. **Damage boost consumed after cell impact**
   - Given: Bolt just hit a wall (3x boost active)
   - When: Bolt impacts cell A (boost consumed), then impacts cell B without hitting a wall
   - Then: Cell A receives 3x damage, cell B receives 1x (normal) damage

3. **Wall bounce resets the cycle**
   - Given: Bolt hit a cell (boost consumed, no active boost)
   - When: Bolt bounces off another wall
   - Then: 3x damage boost is re-armed for the next cell impact

4. **Multiple wall bounces before cell hit do not stack**
   - Given: Bolt bounces off wall, then bounces off another wall (no cell hit between)
   - When: Bolt impacts a cell
   - Then: Cell receives 3x damage (not 9x) — boost refreshes, does not stack

5. **Stacks multiplicatively with chip damage boosts**
   - Given: Bolt has existing DamageBoost(1.5x) from a chip, Ricochet Protocol active
   - When: Bolt bounces off wall then impacts a cell
   - Then: Effective damage = base * 1.5 * 3.0 = 4.5x base damage

6. **Each bolt tracked independently**
   - Given: Bolt A just bounced off a wall, Bolt B has not
   - When: Both bolts impact cells
   - Then: Bolt A's target takes 3x damage, Bolt B's target takes 1x damage

7. **Boost persists across multiple wall bounces without cell contact**
   - Given: Bolt bounces off wall (boost armed)
   - When: Bolt bounces off breaker, then another wall, then impacts a cell
   - Then: Cell receives 3x damage (boost was never consumed by the breaker bounce — only cell impact consumes it)

## Edge Cases

- **Piercing bolt**: If the bolt has Piercing and passes through a cell, that counts as a cell impact — the boost is consumed on the first cell contacted, even if the bolt continues through.
- **Bolt-lost**: If the bolt is lost after a wall bounce but before hitting a cell, the boost is simply lost with the bolt. No special handling needed.
- **Wall bounce at node start**: The first action in a node could be a wall bounce (bolt launched at an angle). Ricochet Protocol works immediately — no warmup needed.
- **Interaction with Deadline**: Both multiply damage independently. A wall-bounced bolt during Deadline deals base * 3.0 (Ricochet) * 2.0 (Deadline) = 6x damage.
- **Cell-type interactions**: The 3x damage applies to all cell types equally (armored, shielded, etc.) — it modifies bolt damage, not cell vulnerability.
