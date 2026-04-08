# Protocol: Deadline

## Category
`effect-tree`

## Game Design

**Behavior change**: You WANT to slow-play 75% of the node, then explode in the danger zone.

**Mechanic**: When node timer drops below 25%, all bolts get 2x speed + 2x damage until node ends.

**Origin**: Promoted from legendary chip.

## Config Resource
None — tuning values are in the effect tree RON.

## Components
None. All effects are dispatched through the effect system onto existing bolt components.

## Messages
**Reads (existing)**:
- `NodeTimerThresholdOccurred` — the effect system's bridge fires when the node timer crosses the 25% threshold

**Sends**: None.

## Systems
No runtime systems — effects dispatched through the effect system.

The effect tree is installed on the breaker entity at protocol activation time via `dispatch_protocol_selection`. The effect system's existing `NodeTimerThreshold` trigger bridge evaluates the tree when the timer crosses the configured threshold. Effect cleanup happens automatically via `SourceId`-based removal at node/run end.

## Effect Tree

The RON effect tree uses the new effect system's primitives (todo #2 dependency). Conceptual structure:

```ron
// assets/protocols/deadline.protocol.ron
(
    name: "Deadline",
    description: "When node timer drops below 25%, all bolts get 2x speed and 2x damage.",
    unlock_tier: 0,
    tuning: Deadline(
        effects: [
            Route(Bolt, When(NodeTimerThreshold(0.25), [
                Do(SpeedBoost(factor: 2.0)),
                Do(DamageBoost(factor: 2.0)),
            ])),
        ],
    ),
)
```

**Notes**:
- The `Route(Bolt, ...)` directive targets all bolt entities.
- `When(NodeTimerThreshold(0.25), ...)` gates on the node timer reaching 25% remaining.
- Both `SpeedBoost` and `DamageBoost` apply as multiplicative factors on top of existing chip effects.
- Effects persist until node end (the threshold fires once per node, effects last until cleanup).
- Exact `EffectKind` variant names and `Route`/`When` syntax depend on the new effect system (todo #2). The above is the design intent — concrete types will match the new system's `ValidDef` format.

## Cross-Domain Dependencies

| Domain | Resource/Message | Access |
|--------|-----------------|--------|
| `bolt` | Bolt entities (velocity, damage components) | Effect system writes via `SpeedBoost`, `DamageBoost` |
| `run/node` | `NodeTimer` (remaining time) | Effect system's `NodeTimerThreshold` bridge reads to detect threshold crossing |
| `effect` (new) | `dispatch_effects`, `SourceId` cleanup | Effect tree installation and removal |

## Expected Behaviors (for test specs)

1. **Bolt speed doubles when timer crosses 25%**
   - Given: Node timer at 26% remaining, bolt velocity magnitude 400.0
   - When: Timer ticks to 24% remaining (crosses 25% threshold)
   - Then: Bolt velocity magnitude becomes 800.0 (2x), direction unchanged

2. **Bolt damage doubles when timer crosses 25%**
   - Given: Node timer at 26% remaining, bolt base damage 10.0
   - When: Timer ticks to 24% remaining
   - Then: Bolt effective damage becomes 20.0 (2x)

3. **Effects stack multiplicatively with chip effects**
   - Given: Bolt has existing DamageBoost(1.5x) from a chip, node timer crosses 25%
   - When: Deadline DamageBoost(2.0x) activates
   - Then: Effective damage = base * 1.5 * 2.0 = 3.0x base damage

4. **Effects persist until node end**
   - Given: Deadline activated (timer below 25%), bolt speed doubled
   - When: Node ends (NodeState transitions out of Playing)
   - Then: Deadline effects are cleaned up, bolt speed returns to pre-Deadline value

5. **Threshold fires only once per node**
   - Given: Timer drops below 25% and Deadline activates
   - When: Timer continues ticking (20%, 15%, 10%...)
   - Then: No additional effects are applied (already active)

6. **No effect if timer never reaches threshold**
   - Given: Deadline protocol is active, node timer starts at 100%
   - When: Node ends with timer still above 25% (fast completion)
   - Then: Deadline effects never fire

7. **Multiple bolts all receive the boost**
   - Given: 3 bolts in play, timer crosses 25%
   - When: Deadline activates
   - Then: All 3 bolts receive 2x speed and 2x damage

## Previous Legendary (reference only)

Previous legendary chip RON for reference. The protocol design above is the source of truth — this is preserved for memory/reference only, possibly relevant for tuning comparison, possibly not.

```ron
// Former deadline.chip.ron legendary: slot
effects: [
    On(target: Bolt, then: [
        When(trigger: NodeTimerThreshold(0.25), then: [
            Until(trigger: NodeEnd, then: [
                Do(SpeedBoost(multiplier: 2.0)),
                Do(DamageBoost(2.0)),
            ]),
        ]),
    ]),
]
```

## Edge Cases

- **Node starts below 25%**: If a node's timer is configured to start below 25% (unlikely but possible), Deadline fires immediately on node start.
- **Bolt spawned after threshold**: A bolt spawned after the 25% threshold was already crossed (e.g., via Fission split) should still receive the Deadline boost. This depends on how the new effect system handles late-spawned entities — may need `Route(Bolt, ...)` to re-evaluate on bolt spawn.
- **Bolt-lost during Deadline**: Losing a bolt during the Deadline window does not cancel Deadline for remaining bolts. Each bolt's effects are independent.
- **Node end cleanup**: All Deadline effects must be removed at node end via `SourceId("protocol:deadline")` cleanup, so the next node starts clean.
- **Interaction with Kickstart**: Both can be active in the same run. Kickstart fires at node start (first 3s), Deadline fires at timer < 25%. They don't overlap unless the node is extremely short. If they did overlap, effects stack multiplicatively (4x speed, 4x damage).
