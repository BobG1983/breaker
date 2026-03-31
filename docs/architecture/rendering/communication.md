# Communication Patterns & System Ordering

## Message Types

### Gameplay → VFX (entity attachment)
`AttachVisuals { entity, config: EntityVisualConfig }` — gameplay sends once at spawn. Crate attaches mesh/material/shaders.

### Gameplay → VFX (dynamic state)
`SetModifier { entity, modifier, source }` — gameplay sends each FixedUpdate for per-frame visual state (speed → trail length). Overwrites by source key.

### Gameplay → VFX (chip effects)
`AddModifier { entity, modifier, source }` / `RemoveModifier { entity, source }` — gameplay sends on effect fire/reverse. Stacks with DR.

### Gameplay → VFX (recipe VFX)
`ExecuteRecipe { recipe, position, direction, camera, source, target }` — gameplay sends to fire a fully-authored visual recipe. Direction for directional primitives, source/target for anchored primitives. For effects with runtime-variable params, gameplay sends typed primitive messages directly.

### Gameplay → VFX (event VFX)
Same `ExecuteRecipe` message. Cell death sends `ExecuteRecipe { recipe: "shatter_sparks", position }`. No separate event VFX message types.

### VFX → Gameplay (completion)
`TransitionComplete` for state transitions gated on animation.

### Typed Per-Primitive Messages (crate-owned)

Screen effect messages carry camera `Entity` explicitly:

```rust
TriggerScreenShake { camera: Entity, tier: ShakeTier, direction: Option<Vec2> }
TriggerScreenFlash { camera: Entity, color: Hue, intensity: HdrBrightness, duration_frames: u32 }
TriggerRadialDistortion { camera: Entity, origin: Vec2, intensity: f32, duration: f32 }
```

Spatial primitives get position from ExecuteRecipe or direct send:

```rust
SpawnExpandingRing { position: Vec2, speed: f32, max_radius: f32, ... }
SpawnBeam { position: Vec2, direction: Direction, range: f32, width: f32, ... }
SpawnSparkBurst { position: Vec2, count: u32, ... }
```

## System Ordering

### Gameplay → VFX Pipeline (FixedUpdate)

```
Gameplay systems (physics, collisions, effect fire/reverse)
    → Effect fire() sends ExecuteRecipe / direct primitive messages
    → Modifier messages sent (SetModifier, AddModifier, RemoveModifier)
    ↓
VFX crate (FixedUpdate, after gameplay)
    → ExecuteRecipe handler: spawns RecipeExecution entity, emits per-primitive messages for Immediate phases
    → Per-primitive handlers: spawn VFX entities (parallel)
    → Modifier handler: updates computed visual state per entity
    → VFX tick: advances active primitives, checks PhaseGroup completion for recipe phasing
    → VFX cleanup: despawns expired VFX entities
```

### Visual Update (Update)

```
Shader updates: crate updates material uniforms from computed modifiers
    → Interpolation for smooth display between FixedUpdate ticks
```

### Post-Processing Pipeline (Render Graph)

See [screen_effects.md](screen_effects.md) for the full pipeline order and FullscreenMaterial implementation details.

---

## Domain Restructuring

Phase 5 eliminates the `ui/` and `fx/` domains. Their responsibilities are absorbed into `screen/`, `run/`, `shared/`, and `rantzsoft_vfx`. There is no `rendering/` or `graphics/` game domain.

### Before (current)

```
screen/     — state registration, transitions, cleanup
ui/         — chip select, menus, pause, HUD, side panels
fx/         — transition overlays, fade-out, punch scale
```

### After

```
screen/                      — screen lifecycle + per-screen UI + transitions
  main_menu/                 — main menu systems + interactive idle UI
  run_setup/                 — breaker/bolt select systems + UI
  chip_select/               — chip card layout, rarity treatments, timer pressure, selection
  run_end/                   — victory/defeat presentation, stats, "almost unlocked" teases
  pause/                     — pause overlay + desaturation, menu options
  playing/
    hud/                     — diegetic HUD: timer wall glow, life orbs, node progress
  transition/                — transition styles (Flash, Sweep, Glitch, Collapse/Rebuild)
                               PlayingState substate management, TransitionComplete

run/                         — (expanded) gains visual feedback systems
  temperature.rs             — RunTemperature palette (runtime shift for grid/bloom/walls)
  danger_vignette.rs         — reads timer + lives, sends VFX messages

shared/                      — (expanded) gains graphics config
  graphics_config.rs         — GraphicsConfig / GraphicsDefaults resource

rantzsoft_vfx                — (expanded) gains animation primitives
                               fade-out, punch scale (absorbed from fx/)
```

No `graphics/` or `rendering/` game domain. Game-specific visual concerns are dispersed to the domains that own the relevant game state.

### Eliminated Domains

| Domain | Absorbed Into |
|--------|--------------|
| `ui/` | Per-screen UI → `screen/<screen_name>/`. Diegetic HUD → `screen/playing/hud/`. Side panels → removed. |
| `fx/` | Transitions → `screen/transition/`. Fade-out, punch scale → `rantzsoft_vfx` (generic). |
| `graphics/` | Never created. Config → `shared/`. Temperature + vignette → `run/`. HUD → `screen/`. Animation → crate. |

### game.rs Plugin Changes

- Remove: `UiPlugin`, `FxPlugin`
- Modify: `ScreenPlugin` absorbs per-screen UI from `UiPlugin` and transitions from `FxPlugin`
- Add: `RantzVfxPlugin` (new crate)
- `GraphicsConfig` registered as a shared resource (not a plugin)

### PlayingState Expansion (from transitions.md)

`TransitionOut`, `ChipSelect`, `TransitionIn` move from `GameState` to `PlayingState` substates. See [transitions.md](transitions.md) for the full rationale.

---

## Game-Side VFX Orchestration

Which gameplay domain sends VFX messages for each game event. The crate provides the primitives; these domains compose them.

### Cell Destruction VFX (cells/ domain)

cells/ determines destruction context based on recent kill rate:
- Single kill → sends `ExecuteRecipe` with `death_recipe` (from cell RON)
- Combo (2-4 rapid kills) → sends with `death_recipe_combo` (falls back to `death_recipe` if None)
- Chain (5+ kills) → sends with `death_recipe_chain` (falls back to `death_recipe` if None)

Context detection: cells/ tracks recent `CellDestroyedAt` messages within a time window.

### Cell Hit VFX (cells/ domain)

On `DamageCell` message: sends `ExecuteRecipe` with cell's `hit_recipe`, position at impact point, direction from bolt velocity.

### Bolt Lost VFX (bolt/ domain)

`bolt_lost` system sends all VFX directly after gameplay logic:
- `ExecuteRecipe` with bolt's `death_recipe` (exit streak)
- `TriggerSlowMotion { factor: 0.3, duration: 0.3 }`
- `TriggerDesaturation { target_factor: 0.7, duration: 0.3 }`

### Life Lost VFX (effect/ domain)

`life_lost::fire()` sends:
- `TriggerSlowMotion { factor: 0.2, duration: 0.5 }` (longer than bolt lost)
- `TriggerVignettePulse` (danger flash)

### Bump Grade VFX (breaker/ domain)

On `BumpPerformed { grade }`, the bump system reads the breaker's rendering config:
- Perfect → `ExecuteRecipe` with `perfect_bump_recipe`
- Early → `ExecuteRecipe` with `early_bump_recipe`
- Late → `ExecuteRecipe` with `late_bump_recipe`
- Whiff → nothing (silence IS the feedback)

### Shield Barrier VFX (effect/ domain)

The shield effect manages the barrier visual:
- On `ShieldActive` added to breaker entity: spawns barrier entity with `AttachVisuals` (GlowLine along bottom edge)
- On charge consumed: sends damage recipe on the barrier entity (crack progression)
- On `ShieldActive` removed (last charge): sends `TriggerFracture` on barrier entity + despawns it

### Effect VFX (effect/ domain)

Each chip effect's `fire()` sends the appropriate VFX:
- Effects with standalone VFX (Shockwave, Explode, etc.): `ExecuteRecipe` with the effect's recipe
- Effects with entity modifiers (SpeedBoost, DamageBoost, etc.): `AddModifier` on the target entity
- Effects with continuous VFX (Gravity Well, Tether Beam): `ExecuteRecipe` with anchored steps, source/target entities

Each effect's `reverse()` sends `RemoveModifier` for any modifiers it added.

### Run Won / Run Over VFX (run/ domain)

- Run won: `TriggerSlowMotion { factor: 0.0 }` (freeze-frame) + `TriggerScreenFlash` (white) + transition
- Run over: `TriggerSlowMotion { factor: 0.15 }` (extended) + `TriggerDesaturation { target_factor: 1.0 }` (full monochrome)

### Highlight VFX (run/ domain)

On `HighlightTriggered { kind }`: `ExecuteRecipe` with the highlight's recipe (glitch text + per-highlight game element VFX).
