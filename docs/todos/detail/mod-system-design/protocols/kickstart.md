# Protocol: Kickstart

## Category
`effect-tree`

## Game Design

**Behavior change**: You WANT to optimize explosive openers.

**Mechanic**: Node starts with 3s of 2x bolt speed, Piercing(2), and 2x damage. Timer starts on first bump.

**Origin**: R1 brainstorm.

## Config Resource
None — tuning values are in the effect tree RON.

## Components
None. All effects are dispatched through the effect system onto existing bolt/breaker components. The "first bump" trigger and 3s duration are handled by the new effect system's trigger and scoping primitives.

## Messages
**Reads (existing)**:
- `NodeStart` — the effect system's node start bridge fires, activating the Kickstart window
- `BumpPerformed` — triggers the 3s countdown timer (first bump starts the clock)

**Sends**: None.

## Systems
No runtime systems — effects dispatched through the effect system.

The effect tree is installed on the breaker entity at protocol activation time via `dispatch_protocol_selection`. The effect system's `NodeStart` bridge activates the effects. The `TimeExpires` trigger (or equivalent duration scoping) removes them after 3s from first bump. Effect cleanup also happens at node end via `SourceId`-based removal.

## Effect Tree

The RON effect tree uses the new effect system's primitives (todo #2 dependency). Conceptual structure:

```ron
// assets/protocols/kickstart.protocol.ron
(
    name: "Kickstart",
    description: "Each node opens with 3s of 2x speed, piercing, and 2x damage after first bump.",
    unlock_tier: 0,
    tuning: Kickstart(
        effects: [
            Route(Bolt, When(NodeStart, [
                Until(TimeExpiresAfter(Bump, 3.0), [
                    Do(SpeedBoost(factor: 2.0)),
                    Do(DamageBoost(factor: 2.0)),
                    Do(Piercing(amount: 2)),
                ]),
            ])),
        ],
    ),
)
```

**Notes**:
- `Route(Bolt, ...)` targets all bolt entities.
- `When(NodeStart, ...)` fires at the start of each node.
- `Until(TimeExpiresAfter(Bump, 3.0), ...)` means: effects are active immediately, and a 3s countdown begins on the first `Bump` trigger. When the countdown expires, effects are removed.
- `TimeExpiresAfter(Bump, 3.0)` is a conceptual primitive — the new effect system needs a "start timer on trigger" capability. If this exact primitive doesn't exist, the implementation may use `During(KickstartWindow, ...)` with a custom condition, or `Once(When(NodeStart, ...))` with timer-based reversal.
- Effects re-fire every node (each node gets a fresh Kickstart window).
- Piercing(2) means the bolt passes through up to 2 cells before bouncing normally on the 3rd.
- Exact syntax depends on the new effect system (todo #2).

## Cross-Domain Dependencies

| Domain | Resource/Message | Access |
|--------|-----------------|--------|
| `bolt` | Bolt entities (velocity, damage, piercing components) | Effect system writes via `SpeedBoost`, `DamageBoost`, `Piercing` |
| `run/node` | `NodeState::Playing` entry | Effect system's `NodeStart` bridge fires |
| `breaker` | `BumpPerformed` message | Used as the trigger to start the 3s countdown |
| `effect` (new) | `dispatch_effects`, `SourceId` cleanup, timer scoping | Effect tree installation, timed removal |

## Expected Behaviors (for test specs)

1. **Bolt speed doubles at node start**
   - Given: Kickstart protocol active, new node begins, bolt velocity magnitude 400.0
   - When: Node enters `NodeState::Playing`
   - Then: Bolt velocity magnitude becomes 800.0 (2x), direction unchanged

2. **Bolt damage doubles at node start**
   - Given: Kickstart protocol active, new node begins, bolt base damage 10.0
   - When: Node enters `NodeState::Playing`
   - Then: Bolt effective damage becomes 20.0 (2x)

3. **Bolt gains Piercing(2) at node start**
   - Given: Kickstart protocol active, new node begins, bolt has no piercing
   - When: Node enters `NodeState::Playing`
   - Then: Bolt has Piercing(2) — passes through up to 2 cells

4. **3s countdown starts on first bump**
   - Given: Kickstart effects active (node just started), no bump has occurred yet
   - When: First bump occurs at t=0
   - Then: 3s countdown begins; effects remain active until t=3.0

5. **Effects removed after 3s countdown**
   - Given: First bump occurred, 3s countdown running
   - When: 3.0s elapses after first bump
   - Then: SpeedBoost, DamageBoost, and Piercing from Kickstart are all removed

6. **Effects persist indefinitely until first bump**
   - Given: Kickstart effects active, no bump has occurred
   - When: 10s elapses with no bump (bolt bouncing off walls and cells without breaker contact)
   - Then: Effects still active — timer hasn't started

7. **Each node gets a fresh Kickstart window**
   - Given: Previous node's Kickstart expired, new node begins
   - When: New node enters `NodeState::Playing`
   - Then: Kickstart effects re-activate (2x speed, 2x damage, Piercing(2)), countdown reset

8. **Stacks with chip effects**
   - Given: Bolt has DamageBoost(1.5x) from a chip, Kickstart active
   - When: Node starts
   - Then: Effective damage = base * 1.5 * 2.0 = 3.0x base damage

## Edge Cases

- **Multiple bolts**: All bolts receive Kickstart effects at node start. The 3s countdown is shared (starts on first bump, not per-bolt). When the countdown expires, all bolts lose Kickstart effects simultaneously.
- **Bolt spawned during Kickstart window**: A bolt spawned after node start but before Kickstart expires (e.g., from Fission) should receive the remaining Kickstart effects. This depends on how the new effect system handles late-spawned entities with `Route(Bolt, ...)`.
- **Bolt-lost during Kickstart**: Losing the bolt during the Kickstart window does not cancel Kickstart for remaining bolts.
- **Interaction with Deadline**: Both can be active in the same run. Kickstart fires in the first 3s after first bump, Deadline fires at timer < 25%. They don't typically overlap. If they did (extremely short node), effects stack multiplicatively: 4x speed, 4x damage.
- **Node end before countdown expires**: If the node ends during the Kickstart window (all cells destroyed quickly), effects are cleaned up at node end regardless — no carryover to next node.
- **Piercing stacking**: Kickstart's Piercing(2) stacks additively with chip-granted piercing. A bolt with Piercing(1) from a chip + Piercing(2) from Kickstart = Piercing(3) during the window.
- **Bump whiff**: A bump whiff (breaker misses the bolt) does NOT start the countdown — only a successful bump (Early, Late, or Perfect) starts the 3s timer.
