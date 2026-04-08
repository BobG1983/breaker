# Protocol: Anchor

## Category
`effect-tree`

## Game Design

**Behavior change**: You WANT to commit to a position and predict where the bolt will be, rather than chasing reactively.

**Mechanic**: Stand still for a brief delay, then become "planted." While planted: better bump force, wider perfect bump window, and Piercing. Start moving to unplant.

**Origin**: Promoted from Anchor evolution (Quick Stop x2 + Bump Force x2). Delete the evolution entry. Anchor evolution RON is removed; Anchor becomes a protocol.

**Tuning values (design targets, TBD exact numbers)**:
- Plant delay: 0.3s of standing still
- Bump force multiplier: 2x
- Perfect window multiplier: 1.5x
- Piercing amount: TBD (likely Piercing(1) or Piercing(2))

**Rival to Burnout**: Both reward stillness but with different patterns — Anchor is positional commitment (stay put, predict bolt path), Burnout is heat rhythm (move to build heat, stop to drain + boost).

## Config Resource
None — tuning values are in the effect tree RON.

## Components
None. The "planted" state and its effects are managed through the effect system. The existing `AnchorPlanted` component (or equivalent) may be reused if the effect system needs a marker, but this is owned by the effect system's `Until`/`During` scoping, not by the protocol module.

## Messages
**Reads (existing)**:
- Breaker movement state (the effect system needs to detect "standing still for N seconds" and "started moving")
- `BumpPerformed` — the effect system's bump bridge applies the enhanced bump force and wider perfect window during planted state

**Sends**: None.

## Systems
No runtime systems — effects dispatched through the effect system.

The effect tree is installed on the breaker entity at protocol activation time via `dispatch_protocol_selection`. The new effect system's `During(Condition, ...)` scoping manages the planted/unplanted state transition. Effect cleanup happens via `SourceId`-based removal at run end.

## Effect Tree

The RON effect tree uses the new effect system's primitives (todo #2 dependency). Conceptual structure:

```ron
// assets/protocols/anchor.protocol.ron
(
    name: "Anchor",
    description: "Stand still to plant. While planted: stronger bumps, wider perfect window, and piercing.",
    unlock_tier: 0,
    tuning: Anchor(
        effects: [
            Route(Breaker, During(StillFor(0.3), [
                Do(BumpForceBoost(factor: 2.0)),
                Do(PerfectWindowBoost(factor: 1.5)),
                Route(Bolt, [
                    Do(Piercing(amount: 1)),
                ]),
            ])),
        ],
    ),
)
```

**Notes**:
- `Route(Breaker, ...)` targets the breaker entity.
- `During(StillFor(0.3), ...)` activates when the breaker has been stationary for 0.3s and deactivates when the breaker starts moving.
- `BumpForceBoost` and `PerfectWindowBoost` modify the breaker's bump parameters while planted.
- `Route(Bolt, [Do(Piercing(...))])` applies piercing to bolts while the breaker is planted. This nesting may need design iteration — the bolt receives piercing while the breaker is planted, and loses it when the breaker unplants.
- The `During` primitive is part of the new effect system (todo #2). It provides enter/exit scoping that replaces the current `Until` pattern for state-based conditions.
- Exact values are TBD — current evolution values (0.3s, 2x, 1.5x) are design targets. Piercing amount needs design decision.

## Cross-Domain Dependencies

| Domain | Resource/Message | Access |
|--------|-----------------|--------|
| `breaker` | Breaker entity (movement state, position) | Effect system reads to detect "standing still" |
| `breaker` | Bump force, perfect window parameters | Effect system writes via `BumpForceBoost`, `PerfectWindowBoost` |
| `bolt` | Bolt entities (piercing component) | Effect system writes via `Piercing` |
| `effect` (new) | `dispatch_effects`, `SourceId` cleanup, `During` scoping | Effect tree installation, state-based scoping, and removal |

## Expected Behaviors (for test specs)

1. **Breaker plants after standing still for 0.3s**
   - Given: Breaker at position (0.0, -200.0), velocity (0.0, 0.0), Anchor protocol active
   - When: 0.3s elapses with no movement input
   - Then: Breaker enters planted state (effects activate)

2. **Planted breaker has 2x bump force**
   - Given: Breaker is planted, base bump force 500.0
   - When: Bolt bumps the breaker
   - Then: Bump force applied is 1000.0 (2x)

3. **Planted breaker has 1.5x perfect bump window**
   - Given: Breaker is planted, base perfect window 0.2s
   - When: Bolt enters bump zone
   - Then: Perfect window is 0.3s (1.5x)

4. **Bolts gain piercing while breaker is planted**
   - Given: Breaker is planted, bolt has no piercing
   - When: Anchor piercing effect activates
   - Then: Bolt has Piercing(1) — passes through cells instead of bouncing off

5. **Moving unplants immediately**
   - Given: Breaker is planted (standing still for 0.5s)
   - When: Player inputs movement (any direction)
   - Then: Planted state ends immediately — bump force returns to normal, perfect window returns to normal, bolt loses piercing

6. **Re-planting requires standing still again for 0.3s**
   - Given: Breaker just unplanted (started moving)
   - When: Breaker stops moving
   - Then: After 0.3s of stillness, breaker re-plants

7. **Piercing removed from bolt on unplant**
   - Given: Breaker planted, bolt has Piercing(1) from Anchor
   - When: Breaker starts moving (unplants)
   - Then: Bolt's Piercing from Anchor is removed (bolt may still have piercing from chips)

## Edge Cases

- **Multiple bolts**: All bolts receive piercing while breaker is planted, all lose it on unplant. Each bolt tracked independently if it was spawned during planted vs unplanted state.
- **Bolt spawned while planted**: A bolt spawned (e.g., from Fission) while breaker is planted should receive Anchor's piercing immediately.
- **Interaction with existing piercing**: Anchor's piercing stacks additively with chip-granted piercing. If a bolt has Piercing(1) from a chip and Piercing(1) from Anchor, it has Piercing(2) total.
- **Dash while planted**: Dashing counts as movement — unplants the breaker immediately. This creates a tradeoff: stay planted for power or dash for positioning.
- **Node end while planted**: All Anchor effects are cleaned up at node end via `SourceId("protocol:anchor")`.
- **Interaction with Burnout**: Both reward stillness. Anchor plants after 0.3s for bump power; Burnout drains heat while still (1.5s for full drain + speed boost). A player with both protocols gets planted quickly (0.3s) and then benefits from heat drain over the next 1.2s — different timing windows that complement rather than conflict.
- **Bump during plant delay**: If the bolt bumps the breaker during the 0.3s plant delay (before planting completes), the bump uses normal force and window. Only bumps after planting is complete get the boost.
