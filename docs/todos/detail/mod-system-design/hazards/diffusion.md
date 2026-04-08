# Hazard: Diffusion

## Game Design

Incoming damage to a cell is shared with adjacent cells. The original target takes reduced damage. At base, depth is 1 (only immediate neighbors). Every 5 stacked levels, cascade depth increases by 1.

**Stacking formula**: `share_percent = 20% + 10% * (stack - 1)`. Target takes `(100% - share_percent)` of original damage; each adjacent cell within cascade depth receives `share_percent / adjacent_count` of original damage as a new `DamageDealt<Cell>`.

**Cascade depth**: `depth = 1 + floor((stack - 1) / 5)`. At stack 1-5, depth 1. At stack 6-10, depth 2. At stack 11-15, depth 3.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DiffusionConfig {
    pub base_share_percent: f32,      // 20.0
    pub share_per_level_percent: f32, // 10.0
    pub depth_increase_interval: u32, // 5
}
```

## Components

None. Diffusion is stateless -- it intercepts the damage pipeline per-frame with no per-entity tracking.

## Messages

**Reads**: None directly — the hazard domain only provides the `DiffusionConfig` resource.
**Sends**: None directly.

The cell damage system (`apply_damage::<Cell>` in the cells domain) reads `Res<DiffusionConfig>` + `Res<ActiveHazards>` and handles redistribution internally:
1. For each `DamageDealt<Cell>`, computes `share_percent` from config + stack count
2. Reduces original target's damage to `damage * (1.0 - share_percent / 100.0)`
3. Sends additional `DamageDealt<Cell>` messages for adjacent cells within cascade depth

## Systems

**No hazard-domain runtime systems for the damage redistribution itself.** The cells domain's `apply_damage::<Cell>` reads `DiffusionConfig` and handles it.

The hazard domain provides:
- `DiffusionConfig` resource (populated at hazard activation time)
- `ActiveHazards` resource (already provided by `HazardPlugin`)

The `apply_damage::<Cell>` system (cells domain, after effect refactor) gains a conditional branch:
```
if let Some(config) = diffusion_config {
    let stack = active_hazards.stacks(HazardKind::Diffusion);
    let share_percent = (config.base_share_percent + config.share_per_level_percent * (stack - 1) as f32).min(95.0);
    let depth = 1 + (stack - 1) / config.depth_increase_interval;
    // reduce original damage, send DamageDealt<Cell> for adjacent cells within depth
}
```

## Stacking Behavior

| Stack | Share % | Target receives | Cascade depth |
|-------|---------|-----------------|---------------|
| 1     | 20%     | 80%             | 1             |
| 2     | 30%     | 70%             | 1             |
| 3     | 40%     | 60%             | 1             |
| 5     | 60%     | 40%             | 1             |
| 6     | 70%     | 30%             | 2             |

**Cap consideration**: At stack 9, share_percent = 100% -- the original target would take 0 damage. Systems should clamp `share_percent` to a maximum (e.g., 95%) so the original target always takes some damage. This prevents the degenerate case where cells become immortal when surrounded.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell positions for adjacency lookup | Direct query (cells are in hazard's ECS world) |
| `cells` | Modifies damage pipeline | `DamageDealt<Cell>` (read + rewrite) |

**Adjacency**: Uses the node's grid layout to determine which cells are adjacent. The cell builder pattern establishes grid positions -- Diffusion queries `CellPosition` (or equivalent grid coordinate component) to find neighbors.

## Expected Behaviors (for test specs)

1. **Damage is shared to adjacent cells at stack=1**
   - Given: Cell A at (2,3) with 100 HP, adjacent cells B and C, bolt deals 50 damage to A
   - When: `diffusion_intercept_damage` runs
   - Then: A receives 40 damage (80% of 50), B receives 5 damage (20% / 2 adjacent), C receives 5 damage

2. **Share percentage scales with stack count at stack=3**
   - Given: Stack=3 (share=40%), Cell A with adjacent cells B, C, D (3 neighbors), bolt deals 60 damage to A
   - When: `diffusion_intercept_damage` runs
   - Then: A receives 36 damage (60% of 60), B/C/D each receive 8 damage (40% of 60 / 3)

3. **Cascade depth increases at stack=6**
   - Given: Stack=6 (depth=2), Cell A adjacent to B, B adjacent to C (but C not adjacent to A), bolt deals 100 damage to A
   - When: `diffusion_intercept_damage` runs
   - Then: A receives reduced damage, B receives ring-1 share, C receives ring-2 share (fraction of B's share)

4. **No diffusion when cell has no adjacent cells**
   - Given: Isolated cell A with no neighbors, bolt deals 50 damage
   - When: `diffusion_intercept_damage` runs
   - Then: A receives full 50 damage (nothing to share with)

5. **System does not run when hazard is inactive**
   - Given: Diffusion not in `ActiveHazards`
   - When: damage is dealt
   - Then: `diffusion_intercept_damage` does not run, damage passes through unmodified

## Edge Cases

- **Diffusion + Sympathy synergy**: Both operate on damage events. Sympathy heals adjacent cells based on damage dealt. If Diffusion reduces the target's damage, Sympathy should heal based on the ORIGINAL damage dealt (not the reduced amount) -- otherwise the two partially cancel. Spec must clarify ordering.
- **Diffusion + Tether synergy**: Tether also redirects damage. If both are active, the order matters. Diffusion should run first (shares damage outward), then Tether applies to whatever damage each cell actually receives.
- **Cascade depth overflow**: At very high stacks (stack=16, depth=4), the cascade could touch most of the grid. Each ring's damage is a fraction of the previous ring's, so it naturally attenuates. No explicit cap needed beyond the share_percent cap.
- **Already-destroyed cells**: Shared damage should not target cells that were destroyed by an earlier `DamageDealt<Cell>` in the same frame. Filter out dead cells during adjacency lookup.
- **Self-referential damage**: A cell must never receive its own shared damage back. Track visited cells per cascade to prevent loops.
- **Cleanup**: No per-entity state to clean up. Config resource removed at run end via hazard cleanup.
