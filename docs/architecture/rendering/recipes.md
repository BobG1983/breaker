# Visual Recipes

## Concept

A visual recipe is a RON-defined composition of visual primitives with phased timing. There are no per-effect context types. A "shockwave" is simply a recipe containing `ExpandingRing + RadialDistortion + ScreenShake + SparkBurst` steps. The recipe RON deserializes directly into a generic `Recipe` struct.

The crate owns everything: the `Recipe` type, `RecipeStore`, RON loading, and dispatch. The game tells the crate where to find recipe files and which Bevy state to load in (same pattern as `rantzsoft_defaults`).

## How Recipes Work

1. **Author time**: Design a recipe RON file listing primitive steps grouped into phases. Each step has all visual params baked in. Position is NOT in the recipe — always a runtime value.
2. **Load time**: RON deserializes directly into a `Recipe` struct. Stored in `RecipeStore` keyed by name.
3. **Fire time**: Game sends `ExecuteRecipe { recipe, position, camera }`.
4. **Dispatch time**: Crate spawns a `RecipeExecution` entity, walks phases, emits typed per-primitive messages according to phase timing. Primitive handlers execute in parallel.

```rust
// Fully baked recipe — just needs position:
world.send(ExecuteRecipe {
    recipe: "shockwave_default".into(),
    position: entity_position,
    camera: Some(game_camera),
});

// Runtime-variable params — skip recipe, send primitives directly:
let range = base_range + range_per_level * stacks as f32;
world.send(SpawnExpandingRing { position: entity_position, max_radius: range, speed: 400.0, ... });
world.send(TriggerRadialDistortion { camera: game_camera, origin: entity_position, ... });
world.send(TriggerScreenShake { camera: game_camera, tier: Small, direction: None });
```

## Recipe Types

```rust
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct Recipe {
    pub name: String,
    pub phases: Vec<Phase>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Phase {
    pub trigger: PhaseTrigger,
    #[serde(default)]
    pub offset: Option<Vec2>,  // offset from anchor for all steps in this phase
    pub steps: Vec<PrimitiveStep>,
}
```

Position of each step = `message.position + phase.offset.unwrap_or(Vec2::ZERO)`.

## Phase Triggers

```rust
pub enum PhaseTrigger {
    Immediate,           // starts at time zero
    Delay(f32),          // starts N seconds after recipe begins
    AfterPhase(usize),   // starts when phase N completes (all its primitives finish)
    OnCompletion,        // starts after ALL preceding phases complete
}
```

**Multiple phases can share the same trigger** — they execute in parallel. Two phases both with `AfterPhase(1)` fire simultaneously when phase 1 completes.

## Phase Completion Tracking

When `ExecuteRecipe` dispatches, it spawns a `RecipeExecution` entity. Each phase's spawned primitive entities are children of a `PhaseGroup(phase_index)` marker entity under the execution entity. When all children of a `PhaseGroup` despawn (lifetime expired), the phase is complete and dependent phases trigger.

```
RecipeExecution entity
  └─ PhaseGroup(0) entity
  │    └─ ExpandingRing entity (lifetime: 0.3s)
  │    └─ RadialDistortion entity (lifetime: 0.3s)
  └─ PhaseGroup(1) entity (blocked until PhaseGroup(0) children gone)
       └─ (spawned when triggered)
```

Cleanup: when all `PhaseGroup` children are gone, the `RecipeExecution` entity self-despawns.

**Anchored primitives complete immediately.** Anchored steps (AnchoredBeam, AnchoredDistortion, etc.) count as "complete" the moment they spawn. They persist independently and don't gate phase completion. This means `AfterPhase(N)` triggers as soon as all non-anchored primitives in phase N expire.

## Repeating Phases

Individual phases can repeat at an interval. This is more flexible than whole-recipe looping — a recipe can have one-shot setup phases and a repeating emission phase.

```rust
#[derive(Deserialize, Clone, Debug)]
pub struct Phase {
    pub trigger: PhaseTrigger,
    #[serde(default)]
    pub offset: Option<Vec2>,
    pub steps: Vec<PrimitiveStep>,
    #[serde(default)]
    pub repeat: Option<RepeatConfig>,  // if set, this phase re-fires at interval
}

#[derive(Deserialize, Clone, Debug)]
pub struct RepeatConfig {
    pub interval: f32,          // seconds between repeats
    pub count: Option<u32>,     // None = infinite (until source entity despawns)
}
```

```ron
// resonance_cascade.recipe.ron
(
    name: "resonance_cascade",
    phases: [
        // Phase 0: one-shot setup — attach persistent aura ring
        (
            trigger: Immediate,
            steps: [
                AnchoredRing(entity: Source, radius: 20.0, thickness: 1.0, hdr: HdrBrightness(0.4), color: White, rotation_speed: 0.5),
            ],
        ),
        // Phase 1: repeating pulse — fires every 0.5s indefinitely
        (
            trigger: Delay(0.5),
            repeat: (interval: 0.5),
            steps: [
                ExpandingRing(speed: 300.0, max_radius: 40.0, thickness: 1.5, hdr: HdrBrightness(0.6), color: White, lifetime: 0.4),
            ],
        ),
    ],
)
```

Repeating phases with `count: None` stop when the `RecipeExecution` entity despawns (which happens when the source entity despawns for anchored recipes, or can be cancelled explicitly via `CancelRecipe`).

### CancelRecipe Message

```rust
/// Cancel an active recipe execution. Despawns the RecipeExecution entity and all its children.
#[derive(Message, Clone)]
pub struct CancelRecipe {
    pub entity: Entity,  // the RecipeExecution entity to cancel
}
```

Game code can also cancel by despawning the source entity directly — all anchored primitives and the RecipeExecution self-despawn.

### How Game Code Gets the RecipeExecution Entity

When `ExecuteRecipe` is handled, the crate inserts an `ActiveRecipe(Entity)` component on the source entity (if `source` is Some) pointing to the `RecipeExecution` entity. Game code reads `ActiveRecipe` to get the handle for `CancelRecipe`.

### RecipeExecution ↔ Source Entity Link

`RecipeExecution` stores `source: Option<Entity>`. Each tick, the crate checks `world.get_entity(source)` — if the source entity no longer exists, the RecipeExecution despawns itself (and all children). This is how infinite-repeating recipes stop when the entity that fired them is removed.

### Camera Fallback

If `ExecuteRecipe.camera` is `None` and the recipe contains screen effect steps (ScreenShake, ScreenFlash, etc.), those steps are **skipped** with a debug warning log. Non-screen steps (spatial primitives, particles) still fire normally.

## PrimitiveStep Enum

Each variant is a primitive with all its visual params. See [types.md](types.md) for supporting type definitions.

```rust
pub enum PrimitiveStep {
    // ── Geometric ──
    ExpandingRing { speed: f32, max_radius: f32, thickness: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },
    ExpandingDisc { speed: f32, max_radius: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },  // filled circle (Pulse)
    Beam { direction: Direction, range: f32, width: f32, hdr: HdrBrightness, color: Hue, shrink_duration: f32, afterimage_duration: f32 },
    EnergyRing { radius: f32, thickness: f32, hdr: HdrBrightness, color: Hue, rotation_speed: f32 },

    // ── Particle ──
    SparkBurst { count: u32, velocity: f32, hdr: HdrBrightness, color: Hue, gravity: f32, lifetime: f32 },
    ShardBurst { count: u32, velocity: f32, rotation_speed: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },
    GlowMotes { count: u32, drift_speed: f32, radius: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },
    ElectricArc { jitter: f32, flicker_rate: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },
    TrailBurst { count: u32, length: f32, hdr: HdrBrightness, color: Hue, fade_distance: f32 },

    // ── Screen effects ──
    ScreenShake { tier: ShakeTier },
    ScreenFlash { color: Hue, intensity: HdrBrightness, duration_frames: u32 },
    RadialDistortion { intensity: f32, duration: f32 },
    ChromaticAberration { intensity: f32, duration: f32 },
    SlowMotion { factor: f32, duration: f32, ramp_in: f32, ramp_out: f32 },
    Desaturation { target_factor: f32, duration: f32 },
    VignettePulse { color: Hue, intensity: f32, duration: f32 },  // single-shot vignette (Life Lost, Time Penalty)

    // ── Line ──
    GlowLine { start_offset: Vec2, end_offset: Vec2, width: f32, hdr: HdrBrightness, color: Hue, shimmer_speed: f32 },  // static glowing line (walls, barrier base)

    // ── Text ──
    GlitchText { text: String, size: f32, color: Hue, duration: f32 },  // glitch-shader text label (highlights)

    // ── Shape-aware destruction (reads entity's Shape from AttachVisuals) ──
    Disintegrate { entity: EntityRef, duration: f32 },  // shader-driven dissolve (noise threshold ramps up, mesh fades out). Add SparkBurst in the recipe if particles are wanted alongside.
    Split { entity: EntityRef, axis: Vec2, drift_speed: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },  // mesh splits into two halves along axis
    Fracture { entity: EntityRef, shard_count: u32, velocity: f32, hdr: HdrBrightness, color: Hue, lifetime: f32 },  // mesh → angular shards scattering outward

    // ── Anchored (entity-tracking, persistent, use Source/Target from ExecuteRecipe) ──
    AnchoredBeam { entity_a: EntityRef, entity_b: EntityRef, width: f32, hdr: HdrBrightness, color: Hue, energy_flow_speed: f32, elasticity: f32 },
    AnchoredDistortion { entity: EntityRef, radius: f32, intensity: f32, rotation_speed: f32 },
    AnchoredRing { entity: EntityRef, radius: f32, thickness: f32, hdr: HdrBrightness, color: Hue, rotation_speed: f32 },
    AnchoredArc { entity_a: EntityRef, entity_b: EntityRef, curvature: f32, hdr: HdrBrightness, color: Hue, flicker_rate: f32, jitter: f32 },
    AnchoredGlowMotes { entity: EntityRef, count: u32, drift_speed: f32, radius: f32, hdr: HdrBrightness, color: Hue, inward: bool },
}
```

## Per-Primitive Message Types (crate-owned)

Each `PrimitiveStep` variant has a corresponding Bevy message that can also be sent directly (bypassing recipes):

| Recipe Step | Direct Message | Extra Fields on Message |
|-------------|---------------|------------------------|
| ExpandingRing | `SpawnExpandingRing` | `position: Vec2` |
| ExpandingDisc | `SpawnExpandingDisc` | `position: Vec2` |
| Beam | `SpawnBeam` | `position: Vec2` |
| EnergyRing | `SpawnEnergyRing` | `position: Vec2` |
| SparkBurst | `SpawnSparkBurst` | `position: Vec2` |
| ShardBurst | `SpawnShardBurst` | `position: Vec2` |
| GlowMotes | `SpawnGlowMotes` | `position: Vec2` |
| ElectricArc | `SpawnElectricArc` | `start: Vec2, end: Vec2` |
| TrailBurst | `SpawnTrailBurst` | `position: Vec2, direction: Vec2` |
| ScreenShake | `TriggerScreenShake` | `camera: Entity` |
| ScreenFlash | `TriggerScreenFlash` | `camera: Entity` |
| RadialDistortion | `TriggerRadialDistortion` | `camera: Entity, origin: Vec2` |
| ChromaticAberration | `TriggerChromaticAberration` | `camera: Entity` |
| SlowMotion | `TriggerSlowMotion` | — |
| Desaturation | `TriggerDesaturation` | `camera: Entity` |
| VignettePulse | `TriggerVignettePulse` | `camera: Entity` |
| GlitchText | `SpawnGlitchText` | `position: Vec2` |
| GlowLine | `SpawnGlowLine` | `start: Vec2, end: Vec2` |
| Disintegrate | `TriggerDisintegrate` | `entity: Entity` |
| Split | `TriggerSplit` | `entity: Entity, axis: Vec2` |
| Fracture | `TriggerFracture` | `entity: Entity` |
| AnchoredBeam | `SpawnAnchoredBeam` | `entity_a: Entity, entity_b: Entity` |
| AnchoredDistortion | `SpawnAnchoredDistortion` | `entity: Entity, camera: Entity` |
| AnchoredRing | `SpawnAnchoredRing` | `entity: Entity` |
| AnchoredArc | `SpawnAnchoredArc` | `entity_a: Entity, entity_b: Entity` |
| AnchoredGlowMotes | `SpawnAnchoredGlowMotes` | `entity: Entity` |

## Entity Reference in Recipe Steps

Anchored primitives reference `Source` or `Target` from the `ExecuteRecipe` message:

```rust
/// Which entity a recipe step binds to.
pub enum EntityRef {
    Source,  // from ExecuteRecipe.source
    Target,  // from ExecuteRecipe.target
}
```

Recipe RON uses these as `entity: Source` or `entity_a: Source, entity_b: Target`. The crate resolves them from the message at dispatch time.

**Anchored primitive despawn**: Each tick, anchored primitives poll `world.get_entity(tracked_entity)`. If the entity no longer exists, the anchored primitive despawns. One-frame visual artifact on despawn is acceptable (the entity is already gone from gameplay).

**ElectricArc in recipes**: Uses Source/Target entity positions. The recipe dispatcher reads `GlobalTransform` from the Source and Target entities to get world-space start/end positions for the arc. Outside recipes (direct `SpawnElectricArc` message), explicit `start: Vec2` and `end: Vec2` are provided.

## RecipeStore

```rust
#[derive(Resource, Debug, Default)]
pub struct RecipeStore {
    recipes: HashMap<String, Recipe>,
}
```

Loaded via `SeedableRegistry` pattern (same as `BreakerRegistry`, `ChipTemplateRegistry`). The game tells the crate: asset directory, extensions, loading state.

**Asset path**: Recipe RON files live in the game's asset directory at `assets/recipes/`. Extension: `.recipe.ron`. The game configures this path when registering the crate's recipe loading pipeline (same pattern as `rantzsoft_defaults` registries).

## ExecuteRecipe Message

```rust
#[derive(Message, Clone)]
pub struct ExecuteRecipe {
    pub recipe: String,
    pub position: Vec2,
    pub direction: Option<Vec2>,  // for directional primitives (spark spray angle, beam direction)
    pub camera: Option<Entity>,   // for screen effect steps
    pub source: Option<Entity>,   // entity that owns/fired this effect
    pub target: Option<Entity>,   // target entity (if applicable)
}
```

No overrides. Recipes have all visual params baked in. Entity references (source, target) allow anchored primitives to be recipe-authored.

**Multi-target effects** (chain lightning, ArcWelder web) fire the same recipe multiple times with different source/target pairs. The game iterates the pairs and sends one `ExecuteRecipe` per pair. Chain lightning: fire "electric_arc" recipe for source→target1, then target1→target2, then target2→target3. ArcWelder: fire "tether_beam" for bolt1→bolt2, bolt2→bolt3, etc.

## Hot-Reload

Recipes support hot-reload through the `rantzsoft_defaults` asset watcher pipeline:
1. Bevy's `AssetServer` detects file change
2. `SeedableRegistry::update_single` replaces the recipe in `RecipeStore`
3. Next `ExecuteRecipe` picks up the new version

Active VFX from the old recipe continue playing; only new fires use the updated recipe. The debug menu should include a "re-fire last recipe" button for rapid iteration.

## RON Examples

```ron
// Simple single-position recipe — offset omitted (defaults to origin)
(
    name: "shockwave_default",
    phases: [
        (
            trigger: Immediate,
            steps: [
                ExpandingRing(speed: 400.0, max_radius: 32.0, thickness: 2.0, hdr: HdrBrightness(1.2), color: White, lifetime: 0.3),
                RadialDistortion(intensity: 0.3, duration: 0.3),
            ],
        ),
        (
            trigger: Delay(0.05),
            steps: [
                ScreenShake(tier: Small),
                SparkBurst(count: 8, velocity: 200.0, hdr: HdrBrightness(0.8), color: White, gravity: 50.0, lifetime: 0.2),
            ],
        ),
    ],
)

// Multi-position recipe with parallel phases
(
    name: "cascade_line",
    phases: [
        (trigger: Immediate, steps: [
            ExpandingRing(speed: 600.0, max_radius: 24.0, thickness: 3.0, hdr: HdrBrightness(1.4), color: White, lifetime: 0.3),
            ExpandingRing(speed: 400.0, max_radius: 48.0, thickness: 1.5, hdr: HdrBrightness(0.8), color: CornflowerBlue, lifetime: 0.3),
        ]),
        (trigger: Delay(0.15), offset: (30.0, 0.0), steps: [
            ExpandingRing(speed: 500.0, max_radius: 32.0, thickness: 2.5, hdr: HdrBrightness(1.2), color: White, lifetime: 0.25),
            ExpandingRing(speed: 350.0, max_radius: 56.0, thickness: 1.0, hdr: HdrBrightness(0.6), color: MediumSlateBlue, lifetime: 0.25),
        ]),
        // Phase 2 and 3 both trigger after phase 1 — they run in parallel
        (trigger: AfterPhase(1), offset: (60.0, 0.0), steps: [
            ExpandingRing(speed: 450.0, max_radius: 40.0, thickness: 2.0, hdr: HdrBrightness(1.0), color: Gold, lifetime: 0.2),
            ExpandingRing(speed: 300.0, max_radius: 64.0, thickness: 0.8, hdr: HdrBrightness(0.5), color: DarkGoldenrod, lifetime: 0.2),
        ]),
        (trigger: AfterPhase(1), steps: [
            ScreenShake(tier: Medium),
            SparkBurst(count: 16, velocity: 300.0, hdr: HdrBrightness(1.0), color: Gold, gravity: 80.0, lifetime: 0.3),
        ]),
    ],
)
```
