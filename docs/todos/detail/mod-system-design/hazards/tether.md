# Hazard: Tether

## Game Design

Adjacent cell pairs are linked with visible beams. Damage to one cell deals a percentage to its partner. Coverage determines what fraction of eligible pairs are linked. Sounds helpful -- isn't. Spreads non-lethal chip damage that feeds Cascade/Renewal synergies. Masters find chain-collapse sequences where Tether links cause cascading kills.

**Stacking formula**:
- Damage share: `25% + 10% * (stack - 1)`
- Link coverage: `40% + 10% * (stack - 1)` of eligible pairs

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct TetherConfig {
    pub base_damage_percent: f32,      // 25.0
    pub damage_per_level_percent: f32, // 10.0
    pub base_coverage_percent: f32,    // 40.0
    pub coverage_per_level_percent: f32, // 10.0
}
```

## Components

```rust
/// Marks a cell as part of a tether pair. Points to the partner cell entity.
#[derive(Component, Debug)]
pub(crate) struct TetherLink {
    pub partner: Entity,
}
```

Each cell in a linked pair has a `TetherLink` pointing to the other. When one partner is destroyed, the surviving cell's `TetherLink` is removed (the link breaks).

## Messages

**Reads**: None directly for damage redirect — the cells domain's `apply_damage::<Cell>` reads `TetherConfig` + `TetherLink` components.
**Sends**: None for damage redirect.

The cell damage system (`apply_damage::<Cell>`) handles tether redirection:
1. For each `DamageDealt<Cell>`, checks if the target has a `TetherLink` component
2. Computes redirect: `damage * damage_percent / 100.0`
3. Sends additional `DamageDealt<Cell>` for the partner
4. Original target takes FULL damage (Tether adds extra damage to the partner, does not reduce incoming)

## Systems

The hazard domain provides link management systems. Damage redirect is handled by the cells domain.

1. **`establish_tether_links`** (hazard domain)
   - Schedule: `OnEnter(NodeState::Playing)` or first frame of Playing
   - Run if: `hazard_active(HazardKind::Tether)`
   - Behavior:
     1. Query all cell entities with grid positions
     2. Build list of all eligible adjacent pairs (horizontally and vertically adjacent)
     3. Compute coverage: `coverage_percent = base + per_level * (stack - 1)`, clamped to 100%
     4. Randomly select `coverage_percent` of eligible pairs using seeded RNG
     5. Insert `TetherLink` component on both cells in each selected pair
   - Ordering: After cell spawn systems complete

2. **`cleanup_broken_tether_links`** (hazard domain)
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Tether)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_damage` (cells may die)
   - Behavior:
     1. For each entity with `TetherLink`, check if the partner still exists and is alive
     2. If partner is dead/despawned, remove `TetherLink` from the surviving cell

## Stacking Behavior

| Stack | Damage % to partner | Link coverage |
|-------|--------------------:|-------------:|
| 1     | 25%                 | 40%          |
| 2     | 35%                 | 50%          |
| 3     | 45%                 | 60%          |
| 5     | 65%                 | 80%          |
| 7     | 85%                 | 100%         |

**Note**: Coverage caps at 100% (all eligible pairs linked). Damage percent is uncapped -- at stack 9 it's 105%, meaning the partner takes MORE damage than the original hit. This is intentional for high-stack punishment.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Queries cell grid positions for adjacency | Direct query |
| `cells` | Adds damage to partner cells | `DamageDealt<Cell>` (sends new entries) |
| `cells` | Reads cell alive/dead state for link cleanup | Direct query |

**Visual**: Tether beams are rendered between linked cells. This is an FX concern -- the `fx` domain reads `TetherLink` components to draw beam sprites. FX details are out of scope for the hazard system design.

## Expected Behaviors (for test specs)

1. **Tether links are established on node start at stack=1**
   - Given: 10 eligible adjacent pairs, stack=1 (coverage=40%)
   - When: `establish_tether_links` runs
   - Then: 4 pairs have mutual `TetherLink` components (8 total components)

2. **Damage redirects to partner at stack=1**
   - Given: Cell A linked to Cell B, bolt deals 100 damage to A
   - When: `tether_redirect_damage` runs
   - Then: A receives 100 damage (full), B receives 25 damage (25% redirect)

3. **Damage scales with stack count at stack=3**
   - Given: Cell A linked to Cell B, stack=3, bolt deals 80 damage to A
   - When: `tether_redirect_damage` runs
   - Then: A receives 80 damage, B receives 36 damage (45% of 80)

4. **Broken links are cleaned up when partner dies**
   - Given: Cell A linked to Cell B, Cell B is destroyed
   - When: `cleanup_broken_tether_links` runs
   - Then: Cell A's `TetherLink` component is removed

5. **No redirect for cells without TetherLink**
   - Given: Cell A has no `TetherLink`, bolt deals 50 damage to A
   - When: `tether_redirect_damage` runs
   - Then: A receives 50 damage, no additional `DamageDealt<Cell>` sent

## Edge Cases

- **Tether + Cascade synergy**: Tether spreads non-lethal damage to partners. When a partner eventually dies, Cascade heals its neighbors. This creates a feedback loop -- Tether feeds Cascade. The systems are independent (both read/write `DamageDealt<Cell>`), but the interaction is emergent.
- **Tether + Diffusion ordering**: If both are active, Diffusion runs first (shares/reduces damage), then Tether runs on the modified damage amounts. This means Diffusion reduces what Tether can redirect.
- **Bidirectional tether damage**: If A is linked to B and B is linked to A, damage to A redirects to B, and damage to B redirects to A. If both take damage in the same frame, each redirect fires independently. This is correct -- no infinite loop because redirected damage does NOT trigger another redirect (only original `DamageDealt<Cell>` entries from the bolt/chip system trigger Tether, not Tether's own output).
- **Coverage randomness**: Link selection uses seeded RNG for deterministic replay. The seed should be derived from the run seed + node index.
- **Restacking mid-run**: When a second Tether stack is added, coverage increases. New links should be established for the new coverage amount. Option A: re-run `establish_tether_links` on each new node (simplest, links refresh per node). Option B: add links mid-node. Recommend Option A -- links are per-node anyway.
- **Cleanup**: `TetherLink` components are on cell entities. When the node ends and cells are despawned, `TetherLink` is automatically cleaned up. `TetherConfig` resource removed at run end.
