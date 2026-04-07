# Cell Modifiers

Cell "types" are modifiers — components added to a standard cell entity. Every cell is a standard cell with HP. Modifiers add behavior on top. A cell can have **multiple modifiers** (e.g., Armored + Sequence, Phantom + Volatile).

Assumes hazards and protocols already exist as implemented systems.

## Modifier Catalog

All modifiers follow the same pattern: a `CellBehavior` enum variant, a behavior folder under `cells/behaviors/`, and builder sugar on `Cell::builder()`.

### Existing Modifiers (already implemented or partially implemented)

| Modifier | Description | Builder API |
|----------|-------------|-------------|
| **Locked** | Must destroy specific key cells to unlock. Key cells defined in node layout RON (`locks` field), not cell type RON. Shielded cells auto-lock with orbit children as keys. | `.locked(key_entities)` |
| **Regen** | Regenerates HP over time. | `.regen(rate)` |
| **Shielded** | Orbit children that must be destroyed first. Auto-locks parent. | `.shielded(config)` |

### New Modifiers

| Modifier | Tier | Description | Builder API |
|----------|------|-------------|-------------|
| **Volatile** | 1 | Explodes on destruction, AoE damage to adjacent cells. Chain reactions. Easier than other modifiers. | `.volatile()` |
| **Sequence** | 3 | Numbered within a group. Must clear in order. Out-of-order hit repairs to full HP. Group length scales 3-6. | `.sequence(group_id, position)` |
| **Survival** | 4 | Bolt-immune. Attacks with pattern-based shots when clear path to bottom. Self-destruct timer (5-8s) starts on first shot, not spawn. Bump-vulnerable (perfect bump kills early). Never last standing in non-boss nodes. **Boss variant**: no self-destruct — permanent turrets (think guns on a cell spaceship). | `.survival(pattern, timer)` / `.survival_permanent(pattern)` |
| **Armored** | 5 | Weak point is always the back — rewards bouncing off walls/ceiling to get behind it. Piercing >= armor value bypasses direction but consumes that much Piercing. Armor scales 1-3. | `.armored(armor_value)` |
| **Phantom** | 6 | Flickers between solid/ghost on slow cycle (~3s). Telegraphs transition into ghost form. Starting phase varies per cell (determined by block placement — some start solid, some ghosted). | `.phantom(starting_phase)` |
| **Magnetic** | 7 | Pulls bolt within radius. Visible field lines. Destroying removes effect. Experts use as gravity assist. | `.magnetic(radius, strength)` |
| **Portal** | 5 (volatile only) | On death, spawns a portal entity at the cell's position. Bolt hitting the portal teleports bolt and breaker into a sub-level. Portal is destroyed when sub-level is cleared. BoltLost in a sub-level teleports bolt and breaker back to the parent level. See Portal Behavior below. | `.portal(sub_level_tier)` |

### Portal Behavior (detailed)
Portal is the most complex modifier. The lifecycle:

1. **Cell with Portal modifier is destroyed** → spawns a portal entity at the cell's position
2. **Portal entity exists on the field** — it's a collidable entity (not a cell). Required to clear the node.
3. **Bolt hits the portal** → bolt and breaker teleport into a sub-level (transition <200ms)
4. **Inside the sub-level**: timer continues from parent level. Sub-level is a small node at difficulty = parent tier - 2 (min tier 0). No portals within sub-levels (zero nesting).
5. **Sub-level cleared** → bolt and breaker teleport back to parent level. Portal entity is destroyed.
6. **BoltLost inside sub-level** → bolt and breaker teleport back to parent level. Portal remains (not cleared). Player must re-enter to try again.

Portal constraints:
- Max 4 portals per node (capped)
- Portals can be on cells that are also locked or guarded — the portal only spawns when the cell dies
- Sub-levels are small and speed-clearable
- Sub-levels exist to make the node harder, not to offer rewards

Portal count ramp across volatile tiers:
```
Early volatile:  0-1 portals per node
Mid volatile:    1-2 portals per node
Later volatile:  2-4 portals per node
Deep infinite:   max 4 portals per node
```

### Modifier Combinations
Cells can have multiple modifiers. Some interesting combos:
- **Volatile + Sequence** = "don't detonate near my sequence" (explosion repairs out-of-order)
- **Magnetic + Armored** = "the Defender" (magnet pulls bolt away from weak point)
- **Phantom + Sequence** = "the Timing Puzzle" (hit in order AND while visible)
- **Armored + Regen** = must get behind it AND kill it fast
- **Phantom + Magnetic** = gravity assist that flickers on/off
- **Portal + Locked** = must unlock before you can kill it and spawn the portal
- **Portal + Armored** = must get behind it to kill it and reach the portal

Not all combinations make sense — some should be disallowed or untested initially. But the system should support arbitrary combinations.

## Key Decisions

### Cell RON Files Dropped
No more `standard.cell.ron`, `lock.cell.ron`, `tough.cell.ron`, etc. Every cell is a standard cell. Modifiers are applied at generation time by the skeleton/block system (later) or directly via the builder (now). Current cell RON files should be removed.

### All Modifier Combos Are Valid
No combo is invalid — Survival is temporary immunity (self-destructs), so even Survival + Sequence or Survival + Portal work. Compile-time validation checks that required params are present, not that the combo is "sensible." Emergent interactions from unusual combos are a feature.

### Modifier Tuning Values
Initially: tuning values are in the builder code (hardcoded or from a config resource). When node sequencing is implemented, block RON files will specify modifiers with params per cell position. See node sequencing todo for block RON format.

### HP Model
HP is NOT defined per cell. Instead:
- A **toughness dimension** (tough/standard/weak) sets a base HP value
- A helper function `hp_for(toughness, tier, node_index) → f32` computes actual HP
- Per-tier multiplier (e.g., +50% per tier) + per-node multiplier (e.g., +10% per node within tier)
- Block positions will specify toughness (default: standard), not raw HP numbers
- "Tough" replaces the old `tough.cell.ron` — it's just a higher toughness dimension on a standard cell

### Visuals Are Modifier-Driven
Cell RON files no longer define `color_rgb` or damage visual params. Instead:
- Each modifier has a defined visual treatment (shader, sprite, overlay) — designed in Phase 5
- Toughness affects base color intensity (tougher = more saturated/brighter)
- No per-cell color authoring — visuals are systematic, not hand-painted

## Implementation Needs

### CellBehavior Enum Changes
The existing `CellBehavior` enum expands to cover all modifiers:
```rust
enum CellBehavior {
    Locked,  // key cells defined in node layout, not here
    Regen { rate: f32 },
    Shielded(ShieldBehavior),
    Volatile,
    Sequence { group_id: u32, position: u32 },
    Survival { pattern: AttackPattern, timer_secs: Option<f32> },
    Armored { value: u8 },
    Phantom { starting_phase: PhantomPhase },
    Magnetic { radius: f32, strength: f32 },
    Portal { sub_level_tier: u8 },
}
```

### Builder API
```rust
Cell::builder()
    .position(pos)
    .dimensions(w, h)
    .toughness(Toughness::Tough)  // or .standard() / .weak()
    .tier_hp(tier, node_index)     // computes HP from toughness + tier + node
    .volatile()
    .armored(2)
    .rendered(mesh, material)
    .spawn(commands);
```

### Toughness Enum
```rust
enum Toughness {
    Weak,      // low base HP
    Standard,  // default base HP
    Tough,     // high base HP
}
```

### Behavior Folders
Following the existing `cells/behaviors/` structure:
```
cells/behaviors/
    locked/         // lock check system (existing, already here)
    regen/          // HP regen system (existing, already here)
    shielded/       // orbit children (existing, already here)
    volatile/       // explosion AoE system
    sequence/       // ordering enforcement, repair system
    survival/       // attack patterns, self-destruct timer, bump-vulnerability
    armored/        // directional damage, piercing interaction
    phantom/        // phase cycling, visibility toggling
    magnetic/       // bolt attraction field
    portal/         // on-death portal spawn, sub-level transition, sub-level generation
```

### Graphics Needs (Phase 5)
Each modifier needs visual representation. Add to Phase 5 scope:
- **Locked**: lock icon overlay (existing, may need polish)
- **Regen**: regen pulse effect (existing, may need polish)
- **Shielded**: orbit children visuals (existing, may need polish)
- **Volatile**: glow/pulse effect, explosion VFX on death
- **Sequence**: visible number overlay, repair flash animation
- **Survival**: attack pattern indicators, self-destruct countdown display, turret visual
- **Armored**: directional armor plating visual, weak point indicator on back, piercing-through VFX
- **Phantom**: ghost form transparency, solid-to-ghost transition animation, phase telegraph
- **Magnetic**: field lines radiating from cell, bolt-pull particle trail
- **Portal**: portal spawn VFX on cell death, portal entity visual (dimensional tear?), sub-level transition effect, return transition effect

### Skeleton Integration (node sequencing — future)
Skeletons in the node generation system will specify modifiers + toughness per cell position. The node generator calls the builder API directly:
```rust
// Skeleton resolved to: Armored(2), Volatile, Tough, at tier 3 node 2
Cell::builder()
    .position(grid_pos)
    .dimensions(w, h)
    .toughness(Toughness::Tough)
    .tier_hp(3, 2)
    .volatile()
    .armored(2)
    .rendered(mesh, material)
    .spawn(commands);
```
Skeleton format and block RON details live in the node sequencing refactor todo.

## Needs Detail

### Modifier Designs (per-modifier)
- Component definitions for each new modifier
- System designs for each modifier's behavior
- How sequence groups are defined across blocks (group_id scoping)
- Survival attack pattern catalog
- Phantom phase timing and telegraph specifics
- Magnetic field physics (force curve, max pull, interaction with Drift hazard)
- Portal sub-level generation (how are sub-levels built? mini-frames?)
- Portal transition system (state management for parent/sub-level, entity lifecycle)
- Portal + BoltLost interaction (teleport back, portal persists)

### Builder & HP
- Toughness enum base values (what HP does Weak/Standard/Tough map to?)
- `hp_for()` function: exact formula for tier + node_index scaling
- Builder compile-time checks: what does "required params present" look like for each modifier?

### Migration
- Remove `standard.cell.ron`, `tough.cell.ron`, `lock.cell.ron`, `regen.cell.ron`
- Update `spawn_cells_from_grid` to use builder with toughness + tier HP
- Update node layout RON to not reference cell type aliases (currently `'S'`, `'T'`, `'L'`, `'R'`)

## Status
`[NEEDS DETAIL]` — design intent captured, needs component/system-level detail
