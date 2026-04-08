# Protocol: Tier Regression

## Category
`custom-system`

## Game Design

**Behavior change**: You WANT to retreat instead of advance.

**Mechanic**: Drop back 1 tier of difficulty. Replay easier tier's nodes for extra chip offerings. Can only appear once per run. In infinite mode: regresses difficulty but keeps hazard stack.

**Origin**: R1 brainstorm.

**Key constraint**: This protocol depends on the node sequencing refactor (todo #7). The exact mechanism for "drop back 1 tier" requires `NodeSequence` to support tier manipulation at runtime. Details below document the design intent; the concrete implementation will be finalized after todo #7 lands.

## Config Resource

Extracted from `ProtocolTuning::TierRegression` at activation time:

```rust
// protocol/protocols/tier_regression.rs

/// Tuning values for Tier Regression, extracted from ProtocolTuning at activation time.
#[derive(Resource, Debug, Clone)]
pub(crate) struct TierRegressionConfig {
    pub tiers_back: u32,
}
```

**Activation**:

```rust
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::TierRegression { tiers_back } = tuning else { return };
    commands.insert_resource(TierRegressionConfig {
        tiers_back: *tiers_back,
    });
}
```

## Components
None. Tier Regression modifies the `NodeSequence` resource directly — it does not add per-entity components.

## Messages

**Reads (existing)**:
- `ProtocolSelected { kind: ProtocolKind::TierRegression }` — handled by `dispatch_protocol_selection`, which calls `activate` to populate the config resource

**Sends (new, TBD)**:
- The exact message or system call to modify `NodeSequence` depends on the node sequencing refactor (todo #7). Options:
  - Direct write to `ResMut<NodeSequence>` (if the protocol domain is granted write access to `NodeSequence` as a cross-domain exception)
  - New `RegressTier { tiers_back: u32 }` message owned by the `run/node` domain (preferred — message-driven cross-domain communication)

## Systems

### `apply_tier_regression`

- **Schedule**: Runs once, immediately after activation. Could be an `OnEnter` system triggered by a state transition, or a one-shot system triggered by the `ProtocolSelected` message.
- **Run condition**: `protocol_active(ProtocolKind::TierRegression)` — though this system runs only once at activation, not per-frame.
- **What it does**:
  1. Reads `TierRegressionConfig` to get `tiers_back` (default: 1)
  2. Modifies `NodeSequence` to insert a regressed tier's worth of nodes at the current position
  3. The regressed nodes use the difficulty parameters of the lower tier
  4. In infinite mode (tier 9+): regresses difficulty but the hazard stack (`ActiveHazards`) is NOT cleared — you replay easier cells with harder hazards
- **Ordering constraints**: Must run after `dispatch_protocol_selection` (config resource must exist). Must run before `advance_node` picks the next node.

**TBD (blocked on todo #7)**:
- How `NodeSequence` represents tiers and whether it supports runtime insertion of nodes
- Whether regression replays the exact same nodes or generates new nodes at the lower tier's difficulty
- Whether the regression is a "detour" (return to current tier after) or a permanent setback
- How chip offerings work during regressed nodes (the design says "extra chip offerings" — does each regressed node offer chips?)

### Cleanup

`TierRegressionConfig` is removed at run end by `protocol::protocols::cleanup()`. No per-frame runtime system exists — the effect is a one-time `NodeSequence` mutation.

## Effect Tree
N/A

## Cross-Domain Dependencies

| Domain | Resource/Message | Access |
|--------|-----------------|--------|
| `run/node` | `NodeSequence` resource | Write (cross-domain — needs message or accepted exception) |
| `run/node` | Tier definitions / difficulty parameters | Read (to know what the regressed tier looks like) |
| `hazard` | `ActiveHazards` resource | Read-only (verify hazards persist through regression in infinite mode) |
| `protocol` | `ActiveProtocols` resource | Read (gating via `protocol_active`) |
| `protocol` | `TierRegressionConfig` resource | Read (tiers_back value) |

**Hard dependency**: Node sequencing refactor (todo #7) must land before this protocol can be implemented. The current `NodeSequence` may not support the runtime tier manipulation that Tier Regression requires.

## Expected Behaviors (for test specs)

1. **Tier index decreases by 1 on activation**
   - Given: Player at tier 5, Tier Regression selected
   - When: Protocol activates
   - Then: Next nodes are at tier 4 difficulty

2. **Can only appear once per run**
   - Given: Tier Regression already offered this run (whether taken or not)
   - When: Protocol offering is generated for subsequent tiers
   - Then: Tier Regression does not appear in the offering pool

3. **Regressed nodes offer chip selections**
   - Given: Tier Regression activated, playing regressed tier 4 nodes
   - When: A regressed node is completed
   - Then: Player gets a chip offering (extra chance to power up before returning to harder tiers)

4. **Hazard stack persists through regression in infinite mode**
   - Given: Tier 12, 4 hazards active (e.g., Decay x2, Drift x1, Haste x1), Tier Regression selected
   - When: Protocol activates, tier regresses to 11
   - Then: All 4 hazard stacks remain active and at their current intensity

5. **Cell difficulty matches regressed tier**
   - Given: Tier Regression activated, regressed from tier 5 to tier 4
   - When: Regressed nodes are generated
   - Then: Cell HP, layouts, and node parameters match tier 4 definitions

6. **Protocol activation is immediate**
   - Given: Player selects Tier Regression on chip select screen
   - When: Chip select screen closes
   - Then: The very next node uses regressed tier difficulty

7. **Minimum tier floor**
   - Given: Player at tier 1, Tier Regression selected
   - When: Protocol activates (tiers_back: 1)
   - Then: Tier cannot go below tier 0 (or tier 1, depending on design — TBD). Regression is clamped.

## Edge Cases

- **Tier 0 / tier 1 regression**: What happens if the player is at the lowest possible tier? The regression should be clamped — either no regression occurs (waste of protocol pick) or the protocol should not be offered at tier 0/1. Design decision needed.
- **Infinite mode tier calculation**: In infinite mode (tier 9+), tiers may be procedurally generated. Regression needs to know what "1 tier back" means when tiers are generated on the fly. This is a todo #7 design question.
- **Interaction with hazard selection**: After regressing, does the player still get hazard offerings at the regressed tier (if tier 9+)? Likely yes — the hazard stack grows regardless of tier regression. This makes regression a mixed blessing: easier cells but more hazards.
- **Protocol re-offering**: Tier Regression can only appear once per run. The offering system must check `ActiveProtocols` or a separate `offered_protocols` set to prevent re-offering.
- **Node sequence integrity**: Modifying `NodeSequence` mid-run must not corrupt the sequence or create invalid state. The regression must cleanly insert or replace nodes without breaking the `advance_node` → `resolve_post_chip_state` flow.
- **Return to normal tier**: After replaying the regressed tier's nodes, does the player return to the tier they were at, or advance from the regressed tier? Design intent suggests returning to current progression — the regression is a "bonus" detour, not a permanent setback. This needs a design decision.
- **Run seed determinism**: The regressed nodes must be deterministic from the run seed. If `NodeSequence` is pre-generated from the seed, inserting regression nodes must also be seeded deterministically.
