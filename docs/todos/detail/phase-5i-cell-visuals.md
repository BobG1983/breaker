# 5f: Cell Visuals

## Summary

Replace flat `ColorMaterial` rectangles with SDF-rendered cells using the `visuals/` domain types from 5e. The cell builder (todo #4) attaches `Shape`, `Hue`, `GlowParams`, and an `EntityGlowMaterial` based on each `CellTypeDefinition`'s rendering block. Damage visual systems communicate health state through direct Rust functions (fracture, fade, flicker, shrink, color shift). Death visual systems produce context-adaptive destruction effects (dissolve, shatter, energy release) using particles from `rantzsoft_particles2d` and screen effects from `rantzsoft_postprocess`. No recipe system. No god-messages. Direct function calls for all visual effects.

## Context

Currently, cells are spawned as flat `Mesh2d(Rectangle)` + `MeshMaterial2d<ColorMaterial>` in `spawn_cells_from_layout` (see `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs`). Damage feedback is a manual color computation in `handle_cell_hit` that tweaks RGB channels on the `ColorMaterial` based on health fraction and `CellDamageVisuals` parameters. There are no destruction effects — `cleanup_cell` simply despawns the entity.

The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) eliminates the old recipe system and `AttachVisuals` message. Instead:
- Cell builder attaches visual components (Shape, Hue, GlowParams, EntityGlowMaterial) at spawn time
- Damage display is a per-frame system that reads `CellHealth::fraction()` and drives `EntityGlowMaterial` uniform updates via direct Rust functions
- Death effects are Rust functions called by the cleanup pipeline that spawn particle emitters and trigger screen effects

## What to Build

### 1. CellTypeDefinition Rendering Block

Add a `rendering` field to `CellTypeDefinition` using `EntityVisualConfig` from the `visuals/` domain. This replaces the current `color_rgb`, `damage_hdr_base`, `damage_green_min`, `damage_blue_range`, `damage_blue_base` fields.

Current `CellTypeDefinition` fields to replace:
```
color_rgb: [f32; 3]
damage_hdr_base: f32
damage_green_min: f32
damage_blue_range: f32
damage_blue_base: f32
```

New field:
```
rendering: EntityVisualConfig   // shape, color (Hue), glow (GlowParams)
damage_display: DamageDisplay   // how damage shows (Fracture, Fade, Flicker, Shrink, ColorShift)
death_effect: DeathEffect       // how destruction looks (Dissolve, Shatter, EnergyRelease)
```

`rendering` uses `EntityVisualConfig` from `visuals/types/entity_visual_config.rs`. For cells, only `shape`, `color`, and `glow` are used (no aura, no trail).

### 2. DamageDisplay Enum

Defines how accumulated damage is visually communicated. Each variant is a direct Rust function that modifies `EntityGlowMaterial` uniforms — NOT a recipe, NOT a message.

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub(crate) enum DamageDisplay {
    Fracture,    // cracks appear via noise-based dissolve threshold creeping in
    Fade,        // core_brightness dims proportionally to health fraction
    Flicker,     // alpha_override oscillates, frequency increases as health drops
    Shrink,      // squash_x and squash_y scale down with health fraction
    ColorShift,  // color shifts from base hue toward dim red as health drops
}
```

Each variant maps to a pure function: `fn apply_damage_display(variant, health_fraction, material_uniforms, time) -> updated_uniforms`.

### 3. DeathEffect Enum

Defines the destruction visual when a cell reaches 0 HP. Each variant is a direct Rust function that spawns particles and optionally triggers screen effects.

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub(crate) enum DeathEffect {
    Dissolve,       // clean fade with spark burst (default for single kills)
    Shatter,        // fracture into shards that scatter outward
    EnergyRelease,  // expanding light ring + dense particle spray
}
```

Each variant maps to a function that takes the cell's position, dimensions, color, and destruction context, then spawns the appropriate `ParticleEmitter` entities and triggers any screen effects.

### 4. DamageDisplay and DeathEffect Components

Store the cell's damage display variant and death effect variant as components, set by the cell builder from `CellTypeDefinition`:

```rust
#[derive(Component, Clone, Debug)]
pub(crate) struct CellDamageDisplay(pub DamageDisplay);

#[derive(Component, Clone, Debug)]
pub(crate) struct CellDeathEffect(pub DeathEffect);
```

### 5. Cell Builder Visual Attachment

The cell builder (from todo #4, cell builder pattern) reads the `rendering` field from `CellTypeDefinition` and attaches:
- `EntityGlowMaterial` via `MeshMaterial2d<EntityGlowMaterial>` (replaces `MeshMaterial2d<ColorMaterial>`)
- `Shape` component (from `rendering.shape`)
- `Hue` derived color in the material uniforms (from `rendering.color`)
- `GlowParams` values mapped to material uniforms (from `rendering.glow`)
- `CellDamageDisplay` component (from `damage_display`)
- `CellDeathEffect` component (from `death_effect`)

This replaces the current `CellDamageVisuals` component and the manual `ColorMaterial::from_color(def.color())` setup in `spawn_cells_from_grid`.

Per-type visual identity from the design doc (`docs/design/graphics/gameplay-elements.md`):

| Cell Type | Shape | Color (Hue) | GlowParams | DamageDisplay | DeathEffect |
|-----------|-------|-------------|------------|---------------|-------------|
| Standard | `RoundedRectangle { corner_radius: 2.0 }` | `MediumSlateBlue` | moderate core (2.0), medium halo | `Fade` | `Dissolve` |
| Tough | `Hexagon` | `MediumSeaGreen` | bright core (3.0), dense halo, higher bloom | `Fracture` | `Shatter` |
| Lock | `Octagon` | `Gold` | amber core (2.5), warm halo | `ColorShift` | `EnergyRelease` |
| Regen | `Circle` | `LimeGreen` | soft core (1.5), gentle halo | `Flicker` | `Dissolve` |

### 6. RON File Updates

Update all cell RON files to use the new rendering block format. Remove `color_rgb`, `damage_hdr_base`, `damage_green_min`, `damage_blue_range`, `damage_blue_base`. Add `rendering`, `damage_display`, `death_effect`.

Example `standard.cell.ron`:
```ron
(
    id: "standard",
    alias: 'S',
    hp: 10.0,
    required_to_clear: true,
    rendering: (
        shape: Some(RoundedRectangle(corner_radius: 2.0)),
        color: Some(MediumSlateBlue),
        glow: Some((
            core_brightness: HdrBrightness(2.0),
            halo_radius: 8.0,
            halo_falloff: 3.0,
            bloom: BloomIntensity(0.5),
        )),
    ),
    damage_display: Fade,
    death_effect: Dissolve,
)
```

### 7. Damage Display System — `update_cell_damage_display`

Replaces the inline damage visual logic currently in `handle_cell_hit`. Instead of computing color on damage message receipt, a per-frame system reads `CellHealth::fraction()` and `CellDamageDisplay` to update `EntityGlowMaterial` uniforms.

This system runs in `FixedUpdate`, after `handle_cell_hit` (so health is current).

**Why a separate system instead of inline in `handle_cell_hit`:**
- Separation of concerns: `handle_cell_hit` handles gameplay (damage, destruction requests), this system handles visuals
- Enables continuous visual effects (flicker needs time, not just damage events)
- Works for regen cells (health changes from regen tick, not from `DamageCell` messages)

**Per-variant behavior** (all are pure functions operating on material uniforms):

| Variant | What changes | How |
|---------|-------------|-----|
| `Fracture` | `dissolve_threshold` | Creeps from 0.0 (full health) toward a max of ~0.4 (near death). Noise-based cracks appear in the SDF shape. Never reaches 1.0 (full dissolve) — that is reserved for death. |
| `Fade` | `core_brightness`, `alpha_override` | `core_brightness` scales linearly with health fraction. `alpha_override` scales from 1.0 to 0.4 (never fully invisible while alive). |
| `Flicker` | `alpha_override` | Oscillates between 0.3 and 1.0 using `sin(time * frequency)`. Frequency increases as health drops: `base_freq + (1.0 - health_fraction) * max_freq_boost`. At full health, no flicker. |
| `Shrink` | `squash_x`, `squash_y` | Both scale from 1.0 (full health) to 0.5 (near death). Cell visually contracts. The `Scale2D` and `Aabb2D` are NOT changed — physics stays the same, only the visual shrinks. |
| `ColorShift` | `color` uniform | Lerps from the base Hue toward a dim red (e.g., `DarkRed`) based on `1.0 - health_fraction`. At full health, shows the original color. |

**System signature:**
```rust
fn update_cell_damage_display(
    cells: Query<(&CellHealth, &CellDamageDisplay, &MeshMaterial2d<EntityGlowMaterial>), With<Cell>>,
    mut materials: ResMut<Assets<EntityGlowMaterial>>,
    time: Res<Time>,
)
```

### 8. Death Effect Functions

Direct Rust functions called from the death pipeline (between `RequestCellDestroyed` and entity despawn). Each function spawns particle emitters and optionally triggers screen effects. These are NOT systems — they are helper functions called by the death system.

The current `cleanup_cell` system despawns immediately. This phase inserts a visual death step: on `RequestCellDestroyed`, the cell's death effect fires, then after a brief delay (or immediately for Dissolve), the entity despawns.

**Per-variant behavior:**

#### `Dissolve`
- Ramp `dissolve_threshold` in `EntityGlowMaterial` from 0.0 to 1.0 over ~0.3s (the shader's noise-based dissolve with burning edge)
- Spawn a `RadialBurst` particle emitter at the cell's position: ~12 sparks, cell's color, short lifetime (0.2-0.4s), moderate speed
- Entity despawns when dissolve completes

#### `Shatter`
- Immediately set `dissolve_threshold` to 1.0 (cell shape vanishes)
- Spawn 6-10 shard particle entities: angular fragments bursting outward from cell center, cell's color, rotation speed, gravity, longer lifetime (0.4-0.8s)
- Spawn a `RadialBurst` of smaller sparks underneath for density
- Optionally: micro screen shake if enabled

#### `EnergyRelease`
- Spawn an expanding ring entity (a circle mesh with `EntityGlowMaterial`, scaling up + fading out over ~0.3s) at cell position
- Spawn a dense `RadialBurst`: ~24 sparks, bright HDR color, high speed, short lifetime
- Trigger `TriggerScreenFlash` (from `rantzsoft_postprocess`) with the cell's color, low intensity, short duration
- Cell dissolves simultaneously

### 9. Context-Adaptive Death Selection

The design doc (`docs/design/graphics/effects-particles.md`) specifies that death effects scale with context:

| Context | Visual Tier |
|---------|------------|
| Single cell break | Base `death_effect` from cell definition |
| Combo (2-4 rapid kills) | Override to `Shatter` regardless of cell definition |
| Chain reaction (5+ kills) | Override to `EnergyRelease` regardless of cell definition |

A `DestructionContext` resource or message field tracks the recent kill rate. The death system reads this context and may override the cell's configured `death_effect` with a higher-tier effect.

**Implementation:** Add a `RecentKillTracker` resource that tracks cell destruction timestamps. The death system queries this tracker:
- 1 kill in the last 0.5s: use cell's `death_effect`
- 2-4 kills in the last 0.5s: force `Shatter`
- 5+ kills in the last 0.5s: force `EnergyRelease`

The tracker is a simple ring buffer of timestamps, cleaned each frame. Keep it under 64 entries.

### 10. Death Pipeline Refactor

Currently `cleanup_cell` immediately despawns. With visual death effects, the pipeline becomes:

1. `RequestCellDestroyed` (from `handle_cell_hit`) — cell entity is still alive
2. Effect bridges evaluate (`bridge_cell_death`) — entity still alive
3. **NEW**: `fire_cell_death_effect` system — reads `CellDeathEffect`, `RecentKillTracker`, spawns particles/screen effects, begins dissolve animation on the cell's material
4. `cleanup_cell` — despawns the cell entity (may need a brief delay for dissolve, or despawn can be immediate if particles carry the visual)

Decision: **immediate despawn with particles carrying the visual.** The cell entity despawns on the same frame as `RequestCellDestroyed` (as it does now). The death effect function spawns independent particle entities and an optional expanding ring entity that persist after the cell is gone. This avoids needing a timer-based deferred despawn system and keeps the destruction pipeline simple.

For `Dissolve` specifically, the dissolve animation plays on independent VFX entities (a snapshot quad at the cell's position with the dissolve shader ramping), not on the cell entity itself.

### 11. Remove CellDamageVisuals Component

The `CellDamageVisuals` component (`hdr_base`, `green_min`, `blue_range`, `blue_base`) is replaced by `CellDamageDisplay` + `EntityGlowMaterial` uniforms. Remove:
- `CellDamageVisuals` struct from `cells/components/types.rs`
- `CellDamageVisuals` insertion in `spawn_cells_from_grid`
- `CellDamageVisuals` in `DamageVisualQuery`
- Inline damage color computation in `handle_cell_hit` (replaced by `update_cell_damage_display` system)

### 12. Update handle_cell_hit

Remove the visual feedback block from `handle_cell_hit`. The system should only handle gameplay: damage application, destruction detection, message sending. Visual feedback moves to `update_cell_damage_display`.

Current code to remove from `handle_cell_hit`:
```rust
// Visual feedback — dim HDR intensity based on remaining health
let frac = health.fraction();
let intensity = frac * visuals.hdr_base;
if let Some(material) = materials.get_mut(material_handle.id()) {
    material.color = Color::srgb(
        intensity,
        visuals.green_min * frac,
        visuals.blue_range.mul_add(1.0 - frac, visuals.blue_base),
    );
}
```

Remove `materials: ResMut<Assets<ColorMaterial>>` from the system's parameters. Remove `CellDamageVisuals` and `MeshMaterial2d<ColorMaterial>` from `DamageVisualQuery`.

### 13. Lock Cell Visual State

Locked cells have a distinct visual indicator: a gold/amber overlay or glow intensity boost that communicates "this cell is immune." When unlocked (by `check_lock_release`), the overlay fades.

Implementation: `update_cell_damage_display` checks `Has<Locked>` — if locked, apply a gold tint overlay to the material color uniform (additive blend with the cell's base color). When `Locked` is removed, the tint disappears on the next frame.

### 14. Regen Cell Pulse

Regen cells have a visible pulsing glow that communicates "this cell is healing." This is a continuous visual independent of damage.

Implementation: `update_cell_damage_display` checks `Has<CellRegen>` — if present, apply a slow sine-wave oscillation to `core_brightness` (e.g., `base * (1.0 + 0.2 * sin(time * 2.0))`). The pulse is gentle enough to read as "alive" without being distracting.

### 15. Orbit Cell Visuals

Orbit cells (children of shield cells) get their own `EntityGlowMaterial` with:
- Smaller shape (Circle or Diamond — visually distinct from parent)
- Brighter glow than parent (clearly separate entities)
- Color from `ShieldBehavior::color_rgb` mapped to `Hue`

The cell builder handles this when spawning orbit children. The orbit visual ring (faint trail showing the orbit path) is a stretch goal — may be deferred to a later phase.

## Module Structure

New and modified files:

```
breaker-game/src/cells/
    components/
        types.rs                            // MODIFIED: remove CellDamageVisuals, add CellDamageDisplay, CellDeathEffect
    definition.rs                           // MODIFIED: add rendering, damage_display, death_effect fields; remove color_rgb, damage_* fields
    queries.rs                              // MODIFIED: update DamageVisualQuery (remove CellDamageVisuals, ColorMaterial refs)
    visuals/
        mod.rs                              // pub(crate) mod damage_display; pub(crate) mod death_effect; pub(crate) mod enums;
        enums.rs                            // DamageDisplay enum, DeathEffect enum
        damage_display/
            mod.rs                          // pub(crate) mod system; #[cfg(test)] mod tests;
            system.rs                       // update_cell_damage_display system
            tests/
                mod.rs
                fracture_tests.rs           // dissolve_threshold scales with health
                fade_tests.rs               // brightness/alpha scale with health
                flicker_tests.rs            // alpha oscillation frequency scales with health
                shrink_tests.rs             // squash scales with health
                color_shift_tests.rs        // color lerps toward red with damage
                locked_overlay_tests.rs     // gold tint on locked cells
                regen_pulse_tests.rs        // brightness oscillation on regen cells
        death_effect/
            mod.rs                          // pub(crate) mod system; pub(crate) mod functions; #[cfg(test)] mod tests;
            system.rs                       // fire_cell_death_effect system
            functions.rs                    // dissolve(), shatter(), energy_release() helper functions
            tests/
                mod.rs
                dissolve_tests.rs           // particles spawned, correct count/color/position
                shatter_tests.rs            // shard particles spawned with rotation/gravity
                energy_release_tests.rs     // ring entity + dense particles + screen flash
                context_adaptive_tests.rs   // kill rate overrides death effect tier
    resources/
        mod.rs                              // MODIFIED: add RecentKillTracker
        recent_kill_tracker.rs              // RecentKillTracker resource (ring buffer of timestamps)

breaker-game/src/cells/
    systems/
        handle_cell_hit/
            system.rs                       // MODIFIED: remove visual feedback block, remove material access
        cleanup_cell.rs                     // MODIFIED: call fire_cell_death_effect before despawn

breaker-game/src/state/run/node/systems/
    spawn_cells_from_layout/
        system.rs                           // MODIFIED: use EntityGlowMaterial instead of ColorMaterial, attach visual components

breaker-game/assets/cells/
    standard.cell.ron                       // MODIFIED: new rendering block format
    tough.cell.ron                          // MODIFIED: new rendering block format
    lock.cell.ron                           // MODIFIED: new rendering block format
    regen.cell.ron                          // MODIFIED: new rendering block format
```

## Type Definitions

### DamageDisplay

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub(crate) enum DamageDisplay {
    Fracture,
    Fade,
    Flicker,
    Shrink,
    ColorShift,
}
```

### DeathEffect

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub(crate) enum DeathEffect {
    Dissolve,
    Shatter,
    EnergyRelease,
}
```

### CellDamageDisplay Component

```rust
#[derive(Component, Clone, Debug)]
pub(crate) struct CellDamageDisplay(pub DamageDisplay);
```

### CellDeathEffect Component

```rust
#[derive(Component, Clone, Debug)]
pub(crate) struct CellDeathEffect(pub DeathEffect);
```

### RecentKillTracker Resource

```rust
#[derive(Resource, Debug)]
pub(crate) struct RecentKillTracker {
    timestamps: VecDeque<f64>,  // game time of each recent cell kill
    window: f64,                // time window for combo detection (default 0.5s)
}

impl RecentKillTracker {
    pub(crate) fn record_kill(&mut self, time: f64);
    pub(crate) fn recent_kills(&self, current_time: f64) -> usize;
    pub(crate) fn cleanup(&mut self, current_time: f64);  // remove entries older than window
}
```

## What NOT to Do

- Do NOT implement a recipe system or RON-driven VFX sequences — all visual effects are direct Rust functions
- Do NOT use `AttachVisuals` or `ExecuteRecipe` messages — builders attach visuals directly, death functions spawn effects directly
- Do NOT modify `EntityGlowMaterial` uniforms from `handle_cell_hit` — the `update_cell_damage_display` system owns visual updates
- Do NOT change cell physics (Aabb2D, CollisionLayers, Scale2D) based on visual damage state — physics stays constant, only visuals change
- Do NOT implement the modifier computation system (5j) — cell damage display is independent of the `ModifierStack` system
- Do NOT implement chip effect visual modifiers on cells (Powder Keg flickering, etc.) — those are 5j (dynamic visuals) using the `VisualModifier` / `ModifierStack` system
- Do NOT implement trail or aura for cells — cells only use shape, color, and glow from `EntityVisualConfig`
- Do NOT implement the temperature palette shift for standard cells — that is 5j
- Do NOT implement bolt/breaker visual attachment — those are separate phases (5f, 5g)
- Do NOT implement shard mesh shapes for particle effects — use default quad particles from `rantzsoft_particles2d`; custom meshes are a future optimization
- Do NOT implement deferred despawn with timers — use immediate despawn with independent particle/VFX entities carrying the visual

## Dependencies

### Requires

| Dependency | What it provides | Status |
|-----------|-----------------|--------|
| 5e (visuals/ domain) | `Shape`, `Hue`, `GlowParams`, `EntityVisualConfig`, `EntityGlowMaterial`, `entity_glow.wgsl` | Not started |
| 5c (rantzsoft_particles2d) | `ParticleEmitter`, `RadialBurst`, `DirectionalBurst` presets | Not started |
| 5d (rantzsoft_postprocess) | `TriggerScreenFlash`, `TriggerRadialDistortion` | Not started |
| Todo #4 (cell builder) | `Cell::builder().definition(&def).spawn()` pattern | Not started |

### Independent of

- 5f (bolt visuals) — different domain, different builder
- 5g (breaker visuals) — different domain, different builder
- 5j (dynamic visuals) — modifier system, temperature palette — not needed for base cell visuals

### Required by

- 5k (bump VFX) — cell hit particles build on cell death particles
- 5j (dynamic visuals) — Powder Keg and other cell modifiers layer on top of base cell visuals

## Verification

- Each cell type renders with its configured SDF shape (RoundedRectangle, Hexagon, Octagon, Circle)
- Each cell type has its configured color via `Hue` (MediumSlateBlue, MediumSeaGreen, Gold, LimeGreen)
- Each cell type has its configured glow parameters (core_brightness, halo_radius, halo_falloff, bloom)
- Damage progression is visually distinct per `DamageDisplay` variant:
  - Fracture: cracks appear and grow with damage (dissolve_threshold increases)
  - Fade: cell dims as health drops (core_brightness and alpha decrease)
  - Flicker: cell flickers faster as health drops (alpha oscillation frequency increases)
  - Shrink: cell visually contracts with damage (squash_x/squash_y decrease)
  - ColorShift: cell shifts toward red as health drops
- Locked cells display a gold tint overlay that disappears on unlock
- Regen cells display a slow pulsing glow animation
- Cell destruction spawns particles at the correct position and in the correct color
- Dissolve death effect: spark burst + dissolve visual
- Shatter death effect: shard particles with rotation and gravity
- EnergyRelease death effect: expanding ring + dense particles + screen flash
- Context-adaptive death selection: single kill uses cell's default, combo (2-4) forces Shatter, chain (5+) forces EnergyRelease
- RecentKillTracker correctly counts kills within the time window and cleans up old entries
- `handle_cell_hit` no longer contains visual feedback code
- `CellDamageVisuals` component is fully removed
- All existing cell-related tests pass (handle_cell_hit, cleanup_cell, check_lock_release, spawn_cells_from_layout)
- RON files parse correctly with the new rendering block format
- `cargo all-dclippy` clean
- `cargo all-dtest` clean
- Orbit children render with distinct shape and color

## Status
`[NEEDS DETAIL]` — blocked on 5e API design (how builders consume EntityVisualConfig, how damage/death functions interface with particles + postprocess crates)
