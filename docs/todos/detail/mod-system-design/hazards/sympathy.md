# Hazard: Sympathy

## Game Design

Damage dealt to a cell heals each adjacent cell for a percentage of damage dealt. Depth 1 at base (only immediate neighbors). Every 5 stacked levels, cascade depth increases by 1 (same scaling as Diffusion). Unlike Diffusion, the target takes FULL damage -- Sympathy is purely additive healing to neighbors.

**Stacking formula**: `heal_percent = 25% + 5% * (stack - 1)`. Each adjacent cell within cascade depth is healed for `damage * heal_percent / 100.0`.

**Cascade depth**: `depth = 1 + floor((stack - 1) / 5)`. At stack 1-5, depth 1. At stack 6-10, depth 2.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct SympathyConfig {
    pub base_heal_percent: f32,        // 25.0
    pub heal_per_level_percent: f32,   // 5.0
    pub depth_increase_interval: u32,  // 5
}
```

## Components

None. Sympathy is stateless -- it reacts to damage events per-frame with no per-entity tracking.

## Messages

**Reads**: None directly — the cells domain's `apply_damage::<Cell>` reads `SympathyConfig` and handles healing.
**Sends**: None directly. The cells domain sends `HealCell` internally.

The cell damage system (`apply_damage::<Cell>`) handles Sympathy:
1. After applying damage to a cell, reads `Res<SympathyConfig>` + `Res<ActiveHazards>`
2. Computes `heal_percent = base + per_level * (stack - 1)`
3. Computes `depth = 1 + floor((stack - 1) / depth_increase_interval)`
4. Finds adjacent cells within `depth` using spatial adjacency query
5. Sends `HealCell` for each adjacent cell (ring 1: `damage * heal_percent / 100.0`, ring N: attenuating)
6. Does NOT heal the damaged cell itself — only neighbors

## Systems

**No hazard-domain runtime systems for the healing logic.** The cells domain's `apply_damage::<Cell>` reads `SympathyConfig` and handles it.

The hazard domain provides:
- `SympathyConfig` resource (populated at hazard activation time)
- `ActiveHazards` resource (already provided by `HazardPlugin`)

## Stacking Behavior

| Stack | Heal % per neighbor | Cascade depth | Example: 100 damage |
|-------|--------------------:|:-------------:|---------------------|
| 1     | 25%                 | 1             | Each adjacent cell healed 25 HP |
| 2     | 30%                 | 1             | Each adjacent cell healed 30 HP |
| 3     | 35%                 | 1             | Each adjacent cell healed 35 HP |
| 5     | 45%                 | 1             | Each adjacent cell healed 45 HP |
| 6     | 50%                 | 2             | Ring 1: 50 HP each; Ring 2: 25 HP each |
| 11    | 75%                 | 3             | Ring 1: 75 HP; Ring 2: 56.25 HP; Ring 3: 42.2 HP |

At high stacks with deep cascade, a single hit heals a wide area. Combined with Diffusion (which also gains cascade depth), clusters become nearly impenetrable.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell positions for adjacency lookup | Direct query |
| `cells` | Heals adjacent cells | `HealCell` message (send) |
| `cells` | Reads damage events | `DamageDealt<Cell>` (read) |

**Adjacency**: Same grid-based adjacency as Diffusion. Uses cell grid positions to find neighbors at each depth ring.

## Expected Behaviors (for test specs)

1. **Adjacent cells healed on damage at stack=1**
   - Given: Cell A at (2,3), adjacent cells B and C, bolt deals 100 damage to A, stack=1
   - When: `sympathy_heal_adjacent` runs
   - Then: B receives `HealCell { amount: 25.0 }`, C receives `HealCell { amount: 25.0 }`. A takes full 100 damage (no reduction).

2. **Heal percentage scales with stack at stack=3**
   - Given: Cell A with adjacent cells B, C, D, bolt deals 80 damage to A, stack=3 (heal=35%)
   - When: `sympathy_heal_adjacent` runs
   - Then: B, C, D each receive `HealCell { amount: 28.0 }` (80 * 0.35)

3. **Cascade depth increases at stack=6**
   - Given: Stack=6 (depth=2), Cell A adjacent to B, B adjacent to C (C not adjacent to A), bolt deals 100 damage to A
   - When: `sympathy_heal_adjacent` runs
   - Then: B (ring 1) receives `HealCell { amount: 50.0 }` (100 * 50%). C (ring 2) receives `HealCell { amount: 25.0 }` (50 * 50%).

4. **No heal when cell has no adjacent cells**
   - Given: Isolated cell A with no neighbors, bolt deals 50 damage
   - When: `sympathy_heal_adjacent` runs
   - Then: No `HealCell` messages sent

5. **Damaged cell itself is NOT healed**
   - Given: Cell A with adjacent cell B, bolt deals 100 damage to A, stack=1
   - When: `sympathy_heal_adjacent` runs
   - Then: Only B receives `HealCell`. A does NOT receive any healing from its own damage event.

6. **System does not run when hazard is inactive**
   - Given: Sympathy not in `ActiveHazards`
   - When: Damage is dealt
   - Then: No `HealCell` messages sent from Sympathy

## Edge Cases

- **Diffusion + Sympathy synergy**: Both gain cascade depth every 5 stacks. At depth 2+, Diffusion shares damage outward (reducing target damage) while Sympathy heals neighbors based on damage dealt. The combination makes clusters extremely resilient. At depth 3+ the effect is game-defining -- clusters become damage sponges. This is the intended "impenetrable cluster" trap.
- **Sympathy ordering with Diffusion**: Sympathy runs AFTER `apply_damage`. If Diffusion modifies damage amounts before `apply_damage`, Sympathy heals based on the modified (reduced) amounts. This is correct -- Sympathy heals based on actual damage dealt, not original damage.
- **Sympathy + Cascade interaction**: Cascade heals adjacent cells when a cell is DESTROYED. Sympathy heals adjacent cells when a cell is DAMAGED (even if not killed). Both produce `HealCell` messages. They are additive -- both effects apply. A cell taking damage heals its neighbors (Sympathy), and if it dies, Cascade heals them again.
- **Dead cell adjacency**: Do not heal cells that are already dead. Filter out destroyed cells during the adjacency lookup.
- **Self-referential healing**: A cell must never heal itself via Sympathy. The damaged cell is excluded from the neighbor set.
- **Zero damage events**: If a `DamageDealt<Cell>` carries 0 damage, Sympathy produces 0 healing. This is correct (no-op).
- **Heal past max HP**: `HealCell` should be capped by the cells domain at the cell's max HP (or 2x starting HP if Volatility's cap applies). Sympathy sends the raw heal amount; the cells domain clamps it.
- **Cleanup**: No per-entity state. `SympathyConfig` removed at run end via hazard cleanup.
