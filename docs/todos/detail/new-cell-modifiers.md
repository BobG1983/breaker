# New Cell Modifiers

## Summary
Implement 7 new cell modifiers: Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal. Each adds unique behavior to standard cells via the CellBehavior enum and builder API.

## Context
The cell builder todo establishes the modifier pattern with existing modifiers (Locked, Regen, Shielded). This todo adds the new modifiers designed in [cell-modifiers.md](cell-builder-pattern/cell-modifiers.md). Each modifier gets its own `cells/behaviors/<name>/` folder with components and systems.

Modifiers are composable — a cell can have multiple modifiers (e.g., Armored + Volatile, Phantom + Sequence). All combos are valid.

## Scope
- In: 7 new CellBehavior enum variants (Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal)
- In: Components and systems for each modifier in `cells/behaviors/<name>/`
- In: Builder sugar for each modifier (`.volatile()`, `.armored(value)`, etc.)
- In: Modifier combination interactions
- In: `Impact(Projectile)` trigger variant for survival projectiles (or whatever replaces "bullet")
- In: Breaker RON forced bolt_lost and projectile_hit behaviors (enforced at definition level)
- In: Refactor existing attraction and gravity well effects to use inverse-square falloff (shared with Magnetic)
- Out: Cell builder typestate (done in builder todo)
- Out: Toughness / HP scaling (separate todo)
- Out: Visual effects for modifiers (Phase 5)
- Out: Node sequencing / skeleton integration (separate todo — but note portal hook-up needed there)

---

## Modifier Designs

### 1. Volatile (Tier 1)

**Concept**: Syntactic sugar for binding `When(Died, [Do(Explode)])` as a bound effect on the cell.

**Components**:
- No new components needed. The builder adds a `BoundEffects` entry with `Trigger::Died → Explode`.
- At spawn time, Volatile computes flat damage = 50% of median neighboring cell max HP. This value is baked into the Explode effect params.

**Systems**:
- No new systems. Uses existing `process_explode_requests` from `effect/effects/explode/`.
- Explode already does radius-based AoE + quadtree query + `DamageCell` messages.

**Chain reactions**: A volatile cell hit by an explosion can die, triggering its own Explode. The existing effect dispatch handles this naturally — `Died` trigger fires on death, which fires the Explode effect, which damages more cells.

**Builder**: `.volatile()` — no params. Damage computed at spawn from neighbors. Could take optional `damage_override: f32` for manual control.

**Edge cases**:
- Chain reaction depth: no explicit cap needed — finite cells means finite chain. But monitor for frame spikes in scenarios.
- Volatile + Sequence: explosion damages out-of-order sequence cells → repairs them to full HP. Intentional emergent combo.

---

### 2. Sequence (Tier 3)

**Concept**: Numbered cells within a group. Must be destroyed in order. Out-of-order hit repairs the target to full HP.

**Components**:
- `SequenceGroup(u32)` — which group this cell belongs to
- `SequencePosition(u32)` — this cell's position in the group (0-indexed)
- `SequenceActive` — marker: this cell is the current target in its group (next to be destroyed)

**Systems**:
- `check_sequence_order` — on `CellDamaged` (or `CellHit`): if the cell has `SequenceGroup` but NOT `SequenceActive`, repair to full HP instead of applying damage. If it has `SequenceActive`, allow damage through normally.
- `advance_sequence` — on cell death: if dead cell had `SequenceGroup + SequenceActive`, find the next cell in the group (by `SequencePosition + 1`) and add `SequenceActive` to it. If no next cell, group is complete.
- `init_sequence_groups` — at node spawn: for each sequence group, add `SequenceActive` to the cell with `SequencePosition(0)`.

**Node layout RON**:
```ron
sequences: {
    1: [(0,0), (1,1), (2,2)],   // group 1: destroy in this order
    2: [(0,2), (1,0)],           // group 2: two cells
},
```
At spawn, `spawn_cells_from_grid` resolves grid coordinates to entity IDs, adds `SequenceGroup(group_id)` and `SequencePosition(index)` components.

**Builder**: `.sequence(group_id, position)` — sets both components.

**Open question for node sequencing todo**: How do sequence chains span blocks? Should they? (Decision: yes, they should. Detail the cross-block scoping in the node sequencing todo.)

**Edge cases**:
- Volatile + Sequence: explosion hitting a non-active sequence cell → repairs to full HP. The explosion "wastes" its damage.
- Sequence cell killed by non-bolt damage (e.g., Explode AoE): should still advance the sequence. `advance_sequence` triggers on any cell death, not just bolt-caused.

---

### 3. Survival (Tier 4)

**Concept**: Bolt-immune turret. Fires projectiles downward when clear path. Self-destruct timer. Bump-vulnerable.

**Components**:
- `SurvivalTurret` — marker: this cell is a turret
- `SurvivalPattern(AttackPattern)` — which attack pattern to use
- `SurvivalTimer { remaining: f32, started: bool }` — self-destruct countdown. `started` flips to true on first shot. `None` for boss variant.
- `BoltImmune` — marker: bolt collisions deal zero damage (still collide for physics, just no HP reduction). Shared component — other modifiers might use this.
- `BumpVulnerable` — marker: perfect bump kills this cell instantly, regardless of HP or immunity.

**Attack patterns** (initial catalog):
```rust
enum AttackPattern {
    StraightDown,       // single projectile straight down
    Spread(u32),        // n projectiles in a downward cone
}
```

**Projectile system** (terminology TBD — see Cross-Cutting Concerns):
- Projectiles are their own entity type (NOT bolts).
- Projectiles damage cells they pass through — turrets clear their own line of fire.
- Bolts absorb projectiles on contact (projectile despawns, bolt unaffected).
- Breaker must dodge projectiles. On hit: triggers `Impact(Projectile)` (or whatever the entity type is named).
- Projectile components: `Position2D`, `Velocity2D`, `CollisionLayers`, damage value, `CleanupOnExit<NodeState>`.

**"Clear path to bottom"**: No cells in the line of fire (straight down for `StraightDown`, cone width for `Spread`). System checks before firing each tick.

**Breaker RON enforcement**: Breaker definitions MUST specify `bolt_lost` and `projectile_hit` behaviors as top-level fields, outside the effects block. These are still root effects, but enforced at the definition schema level so every breaker has them. Note: root effect structure changes with the effect refactor todo (#3) — design the enforced fields to be compatible with that refactor.

**Boss variant**: `.survival_permanent(pattern)` — no self-destruct timer. Permanent turrets.

**Builder**: `.survival(pattern, timer_secs)` / `.survival_permanent(pattern)`

**Edge cases**:
- Survival + Phantom: turret goes ghost → stops firing (can't fire when intangible). Returns to firing when solid.
- Survival + Armored: bolt-immune AND armored. Only bump kills it. Very rare, very nasty.
- "Never last standing in non-boss nodes": if only survival cells remain and all have timers, the timers are running. If a survival cell is the last cell and it's permanent (boss), that's a boss scenario and is expected.

---

### 4. Armored (Tier 5)

**Concept**: Directional weak point. Back = top (away from breaker). Bolt must come from above to damage.

**Components**:
- `ArmorValue(u8)` — armor rating (1-3). Determines how much piercing is needed to bypass.
- `ArmorFacing(ArmorDirection)` — which side the armor is on. Default: `Bottom` (facing breaker). Weak point is the opposite side (top).

**Direction enum**:
```rust
enum ArmorDirection {
    Bottom,  // armor faces down toward breaker, weak point is top (default)
    Top,     // armor faces up, weak point is bottom
    Left,
    Right,
}
```

**Systems**:
- `check_armor_direction` — on bolt-cell collision: determine which side the bolt hit from (compare bolt velocity to cell face normals via dot product). If bolt hit the armored side, block damage. If bolt hit the weak point (back), allow damage.
- **Piercing interaction**: if bolt has `PiercingRemaining >= armor_value`, bypass the direction check — damage goes through regardless. Consume `armor_value` charges of piercing.

**Hit direction detection**: Dot product of bolt velocity and face normal. Bolt moving downward hitting top face = weak point hit (if armor faces bottom).

**Builder**: `.armored(value)` — defaults `ArmorFacing(ArmorDirection::Bottom)`. `.armored_facing(value, direction)` for non-default facing.

**Edge cases**:
- Armored + Regen: must get behind it AND kill it before it regens. High-difficulty combo.
- Armored + Magnetic: magnet pulls bolt toward front (armored side), making it harder to reach the back. "The Defender."
- Armor value 0: invalid — minimum is 1.

---

### 5. Phantom (Tier 6)

**Concept**: Cycles between solid and ghost phases. Ghost = intangible. Telegraph before going ghost.

**Components**:
- `PhantomPhase(Phase)` — current phase
- `PhantomTimer(f32)` — time remaining in current phase
- `PhantomConfig { cycle_secs: f32, telegraph_secs: f32 }` — timing config

**Phase enum**:
```rust
enum PhantomPhase {
    Solid,       // normal cell, fully collidable
    Telegraph,   // ~0.5s warning, cell flickers, still collidable
    Ghost,       // intangible, bolt passes through, CollisionLayers zeroed
}
```

**Phase cycle**: Solid (~1.5s) → Telegraph (~0.5s) → Ghost (~1.0s) → Solid → ... Total ~3s. Tunable via `PhantomConfig`.

**Systems**:
- `tick_phantom_phase` — decrements `PhantomTimer`. When timer hits 0, transitions to next phase. On transition to Ghost: zero `CollisionLayers`. On transition to Solid: restore `CollisionLayers`.
- Telegraph phase: still collidable (same layers as Solid). Visual flicker only. Rewards players who hit during the telegraph window.

**Starting phase**: varies per cell. Set in node layout RON or by the builder. Phase-offset patterns create timing puzzles.

**Builder**: `.phantom(starting_phase)` — sets initial phase and timer.

**Edge cases**:
- Phantom + Sequence: must hit in order AND while solid. Miss the window → wait for next solid phase.
- Phantom + Survival: turret stops firing during ghost phase.
- Phantom during death: if killed during telegraph, dies normally. Ghost cells can't be hit.

---

### 6. Magnetic (Tier 7)

**Concept**: Pulls bolt toward cell center with inverse-square falloff.

**Components**:
- `MagneticField { radius: f32, strength: f32 }` — field radius and base strength

**Force formula**: `force = strength / max(distance², min_distance²)` where `min_distance = half cell width` (prevents infinite force at center). Direction: from bolt toward cell center. Applied as acceleration each physics tick.

**Systems**:
- `apply_magnetic_fields` — each tick: for each bolt, query all `MagneticField` cells within radius (quadtree). Compute inverse-square force per cell, sum all forces, apply as acceleration delta to bolt velocity. Runs in FixedUpdate, before bolt movement.

**Max force cap**: Cap force magnitude at 2x bolt base speed per second to prevent bolt from getting "stuck."

**Builder**: `.magnetic(radius, strength)`

**Attraction/gravitation refactor**: Existing attraction and gravity well effects should be refactored to use the same inverse-square falloff model. Establish a shared `inverse_square_attraction(source_pos, target_pos, strength, min_distance) → Vec2` utility. This belongs in the game crate (not `rantzsoft_physics2d`) since it's a gameplay force, not a physics primitive.

**Edge cases**:
- Multiple magnetic cells: forces sum. Complex gravity landscapes.
- Magnetic + Phantom: field disappears during ghost phase. Bolt suddenly freed — slingshot effect.
- Magnetic + Drift hazard: both affect bolt trajectory. Stack additively.

---

### 7. Portal (Tier 5, volatile nodes only)

**Concept**: On cell death, spawns a portal entity. Bolt entering portal teleports bolt + breaker into a sub-level.

**Components (on cell)**:
- `PortalSettings { sub_level: NodeLayout }` — what sub-level to spawn. Today: a `NodeLayout`. Post node-sequencing refactor: a reference to a node entry in the current sequence.

**Components (on portal entity)**:
- `PortalEntity` — marker
- `PortalSettings` — carried from the dead cell
- `Position2D`, `Dimensions2D`, `CollisionLayers` — collidable entity on the field
- `RequiredToClear` — portal must be cleared to complete the node
- `CleanupOnExit<NodeState>`

**Lifecycle**:
1. Cell with `PortalSettings` dies → system reads settings, calls `spawn_portal(commands, position, settings)`
2. Portal entity exists on field — collidable, required to clear
3. Bolt hits portal → save parent-level state, load sub-level, transition (<200ms)
4. Inside sub-level: timer continues from parent. Sub-level = small node (3x3 or 4x3), tier = parent_tier - 2 (min 0). No portals in sub-levels.
5. Sub-level cleared → teleport back. Portal entity destroyed. Parent-level resumes.
6. BoltLost in sub-level → teleport back. Portal remains (not cleared). Must re-enter.

**Portal spawning**: Cell builder attaches `PortalSettings` to the cell. On death, a system creates the portal entity via helper. The builder does NOT generate the portal itself.

**Sub-level generation (today)**: Reuse existing node generation pipeline with small grid, tier-2 difficulty. `NodeLayout` is pre-generated at node spawn time and stored in `PortalSettings`.

**Sub-level generation (post node-sequencing refactor)**: When generating the node sequence, determine portal cells, pre-generate sub-levels, store references in sequence. `PortalSettings` holds reference to pre-generated entry. Detail in node sequencing todo.

**State management**: Stack-based level context. Push on portal entry, pop on exit. Max depth = 1 (no portals in sub-levels).

**Node layout RON**:
```ron
portals: {
    (2, 1): (tier_offset: -2),
},
```

**Builder**: `.portal(sub_level_tier)` — attaches `PortalSettings`. Actual `NodeLayout` computed and injected at spawn time.

**Constraints**: Max 4 portals per node. No nesting.

**Edge cases**:
- Portal + Locked: must unlock before killing to spawn portal.
- Portal + Armored: must hit weak point to kill and reach portal.
- Multiple portals: each has own sub-level. Clearing one returns to parent with remaining portals.
- Timer: does NOT pause during sub-level. Sub-levels pressure the clock.

---

## Cross-Cutting Concerns

### Terminology Decision Needed
Survival turret projectiles need a game-vocabulary name. "Bullet" is generic. Candidates: "salvo", "round", "shot", "barrage", "pulse". Whatever is chosen becomes an `ImpactTarget` variant (like `Impact(Bolt)`, `Impact(Cell)`).

### Breaker RON Schema Change
Breaker definitions must force `bolt_lost` and `projectile_hit` (survival turret) behaviors as top-level required fields. These are root effects but enforced at the schema level. Note: root effect structure changes with the effect refactor todo (#3) — design the enforced fields to be compatible with that refactor.

### Inverse-Square Falloff Convention
Magnetic fields, gravity well effect, and attraction effect should all use inverse-square falloff with a min-distance cap. Refactor existing attraction and gravity well effects to use this model. Shared `inverse_square_attraction()` utility in the game crate.

### New Trigger Variant
`Impact(Projectile)` (or `Impact(<turret-shot-name>)`) — global trigger for survival turret projectile hitting something. Breakers need to specify what happens on projectile hit.

## Dependencies
- Depends on: Cell builder pattern (provides builder API and behavior folder structure)
- Depends on: Toughness + HP scaling (for appropriate HP values) — soft dependency, can use `.hp(value)` initially
- Blocks: Phase 5 modifier visuals
- Note for node sequencing todo: portal sub-level hook-up, cross-block sequence chains

## Implementation Order
Implement in infrastructure order (simpler modifiers first):
1. **Volatile** (tier 1) — simplest, just binds an existing effect
2. **Sequence** (tier 3) — new systems but straightforward state machine
3. **Armored** (tier 5) — directional hit detection, piercing interaction
4. **Phantom** (tier 6) — phase cycling, collision layer toggling
5. **Magnetic** (tier 7) — physics force application, inverse-square (includes attraction/gravity-well refactor)
6. **Survival** (tier 4) — most new entity types (projectiles), attack patterns, breaker schema change
7. **Portal** (tier 5 volatile) — most complex, sub-level state management

(Survival and Portal are last despite lower tier because they introduce the most new infrastructure.)

## Status
`ready`
