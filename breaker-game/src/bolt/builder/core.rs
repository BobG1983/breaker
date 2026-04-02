//! Typestate builder for bolt entity construction.
//!
//! Entry point: [`Bolt::builder()`]. The builder prevents invalid combinations
//! at compile time via six typestate dimensions: Position, Speed, Angle,
//! Motion, Role, and Visual. `build()` and `spawn()` are only available when
//! all dimensions are satisfied.

use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{PreviousScale, Scale2D, Spatial, Velocity2D};

use crate::{
    bolt::{
        components::{
            Bolt, BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltLifespan, BoltServing,
            BoltSpawnOffsetY, ExtraBolt, PrimaryBolt, SpawnedByEvolution,
        },
        definition::BoltDefinition,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    effect::{BoundEffects, EffectNode},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer,
        WALL_LAYER,
        size::{BaseRadius, MaxRadius, MinRadius},
    },
};

/// Default bolt radius when neither `definition()` nor `with_radius()` is called.
const DEFAULT_RADIUS: f32 = 8.0;

// ── Typestate markers ───────────────────────────────────────────────────────

pub struct NoPosition;
pub struct HasPosition {
    pos: Vec2,
}
pub struct NoSpeed;
pub struct HasSpeed {
    base: f32,
    min: f32,
    max: f32,
}
pub struct NoAngle;
pub struct HasAngle {
    h: f32,
    v: f32,
}
pub struct NoMotion;
pub struct Serving;
pub struct HasVelocity {
    vel: Velocity2D,
}
pub struct NoRole;
pub struct Primary;
pub struct Extra;

// ── Visual dimension markers ───────────────────────────────────────────────

/// Visual dimension not yet chosen.
pub struct Unvisual;
/// Rendered bolt with mesh and material.
pub struct Rendered {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}
/// Headless bolt without visual components.
pub struct Headless;

// ── Optional data ───────────────────────────────────────────────────────────

#[derive(Default)]
struct OptionalBoltData {
    spawned_by: Option<String>,
    lifespan: Option<f32>,
    radius: Option<f32>,
    inherited_effects: Option<BoundEffects>,
    with_effects: Option<Vec<(String, EffectNode)>>,
    definition_params: Option<BoltDefinitionParams>,
    override_base_damage: Option<f32>,
    override_definition_name: Option<String>,
    override_angle_spread: Option<f32>,
    override_spawn_offset_y: Option<f32>,
    color_rgb: Option<[f32; 3]>,
}

struct BoltDefinitionParams {
    name: String,
    base_damage: f32,
    angle_spread: f32,
    spawn_offset_y: f32,
    min_radius: Option<f32>,
    max_radius: Option<f32>,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Typestate builder for bolt entity construction.
pub struct BoltBuilder<P, S, A, M, R, V> {
    position: P,
    speed: S,
    angle: A,
    motion: M,
    role: R,
    visual: V,
    optional: OptionalBoltData,
}

// ── Entry point ─────────────────────────────────────────────────────────────

impl Bolt {
    /// Creates a bolt builder in the unconfigured state.
    #[must_use]
    pub fn builder() -> BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, NoRole, Unvisual> {
        BoltBuilder {
            position: NoPosition,
            speed: NoSpeed,
            angle: NoAngle,
            motion: NoMotion,
            role: NoRole,
            visual: Unvisual,
            optional: OptionalBoltData::default(),
        }
    }
}

// ── Position transition ─────────────────────────────────────────────────────

impl<S, A, M, R, V> BoltBuilder<NoPosition, S, A, M, R, V> {
    pub fn at_position(self, pos: Vec2) -> BoltBuilder<HasPosition, S, A, M, R, V> {
        BoltBuilder {
            position: HasPosition { pos },
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Speed transition ────────────────────────────────────────────────────────

impl<P, A, M, R, V> BoltBuilder<P, NoSpeed, A, M, R, V> {
    pub fn with_speed(self, base: f32, min: f32, max: f32) -> BoltBuilder<P, HasSpeed, A, M, R, V> {
        BoltBuilder {
            position: self.position,
            speed: HasSpeed { base, min, max },
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Angle transition ────────────────────────────────────────────────────────

impl<P, S, M, R, V> BoltBuilder<P, S, NoAngle, M, R, V> {
    pub fn with_angle(self, h: f32, v: f32) -> BoltBuilder<P, S, HasAngle, M, R, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: HasAngle { h, v },
            motion: self.motion,
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Motion transitions ──────────────────────────────────────────────────────

impl<P, S, A, R, V> BoltBuilder<P, S, A, NoMotion, R, V> {
    pub fn serving(self) -> BoltBuilder<P, S, A, Serving, R, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: Serving,
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }

    pub fn with_velocity(self, vel: Velocity2D) -> BoltBuilder<P, S, A, HasVelocity, R, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: HasVelocity { vel },
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Role transitions ────────────────────────────────────────────────────────

impl<P, S, A, M, V> BoltBuilder<P, S, A, M, NoRole, V> {
    pub fn primary(self) -> BoltBuilder<P, S, A, M, Primary, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: Primary,
            visual: self.visual,
            optional: self.optional,
        }
    }

    pub fn extra(self) -> BoltBuilder<P, S, A, M, Extra, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: Extra,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Visual transitions ──────────────────────────────────────────────────────

impl<P, S, A, M, R> BoltBuilder<P, S, A, M, R, Unvisual> {
    /// Configures the bolt for rendered mode with mesh and material.
    pub fn rendered(
        self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> BoltBuilder<P, S, A, M, R, Rendered> {
        let color_rgb = self.optional.color_rgb.unwrap_or([6.0, 5.0, 0.5]);
        let color = Color::linear_rgb(color_rgb[0], color_rgb[1], color_rgb[2]);
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            visual: Rendered {
                mesh: meshes.add(Circle::new(1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
            optional: self.optional,
        }
    }

    /// Configures the bolt for headless mode (no rendering components).
    pub fn headless(self) -> BoltBuilder<P, S, A, M, R, Headless> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            visual: Headless,
            optional: self.optional,
        }
    }
}

// ── from_definition convenience ──────────────────────────────────────────────

impl<P, M, R, V> BoltBuilder<P, NoSpeed, NoAngle, M, R, V> {
    /// Configure the bolt from a `BoltDefinition`.
    ///
    /// Sets speed (base/min/max), angle constraints (`min_h`/`min_v` converted
    /// to radians), and radius from the definition. Also stores definition
    /// params (name, `base_damage`, `angle_spread`, `spawn_offset_y`) for
    /// insertion in `spawn_inner`.
    pub fn definition(self, def: &BoltDefinition) -> BoltBuilder<P, HasSpeed, HasAngle, M, R, V> {
        let mut optional = self.optional;
        optional.definition_params = Some(BoltDefinitionParams {
            name: def.name.clone(),
            base_damage: def.base_damage,
            angle_spread: DEFAULT_BOLT_ANGLE_SPREAD,
            spawn_offset_y: DEFAULT_BOLT_SPAWN_OFFSET_Y,
            min_radius: def.min_radius,
            max_radius: def.max_radius,
        });
        optional.radius = optional.radius.or(Some(def.radius));
        optional.color_rgb = optional.color_rgb.or(Some(def.color_rgb));
        BoltBuilder {
            position: self.position,
            speed: HasSpeed {
                base: def.base_speed,
                min: def.min_speed,
                max: def.max_speed,
            },
            angle: HasAngle {
                h: def.min_angle_horizontal.to_radians(),
                v: def.min_angle_vertical.to_radians(),
            },
            motion: self.motion,
            role: self.role,
            visual: self.visual,
            optional,
        }
    }
}

// ── Optional chainable methods (any typestate) ──────────────────────────────

impl<P, S, A, M, R, V> BoltBuilder<P, S, A, M, R, V> {
    pub fn spawned_by(mut self, name: &str) -> Self {
        self.optional.spawned_by = Some(name.to_string());
        self
    }

    pub const fn with_lifespan(mut self, duration: f32) -> Self {
        self.optional.lifespan = Some(duration);
        self
    }

    pub const fn with_radius(mut self, r: f32) -> Self {
        self.optional.radius = Some(r);
        self
    }

    pub fn with_inherited_effects(mut self, effects: &BoundEffects) -> Self {
        self.optional.inherited_effects = Some(effects.clone());
        self
    }

    pub fn with_effects(mut self, nodes: Vec<(String, EffectNode)>) -> Self {
        self.optional.with_effects = Some(nodes);
        self
    }

    pub const fn with_base_damage(mut self, damage: f32) -> Self {
        self.optional.override_base_damage = Some(damage);
        self
    }

    pub fn with_definition_name(mut self, name: String) -> Self {
        self.optional.override_definition_name = Some(name);
        self
    }

    pub const fn with_angle_spread(mut self, spread: f32) -> Self {
        self.optional.override_angle_spread = Some(spread);
        self
    }

    pub const fn with_spawn_offset_y(mut self, offset: f32) -> Self {
        self.optional.override_spawn_offset_y = Some(offset);
        self
    }
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Extracted values from typestate markers, ready for `build_core`.
struct CoreParams {
    pos: Vec2,
    base: f32,
    min: f32,
    max: f32,
    h: f32,
    v: f32,
    vel: Velocity2D,
}

/// Builds the core component tuple shared by all terminal states.
///
/// Does NOT include `GameDrawLayer::Bolt` — that is only added by Rendered builds.
fn build_core(params: &CoreParams, optional: &OptionalBoltData) -> impl Bundle + use<> {
    let radius = optional.radius.unwrap_or(DEFAULT_RADIUS);

    let base_components = (
        Bolt,
        params.vel,
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
    );

    let spatial_components = Spatial::builder()
        .at_position(params.pos)
        .with_clamped_speed(params.base, params.min, params.max)
        .with_clamped_angle(params.h, params.v)
        .build();

    let radius_components = (
        Scale2D {
            x: radius,
            y: radius,
        },
        PreviousScale {
            x: radius,
            y: radius,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
        BaseRadius(radius),
    );

    (base_components, spatial_components, radius_components)
}

/// Spawns a bolt entity with all components, including optionals and effects.
fn spawn_inner(
    world: &mut World,
    core: impl Bundle,
    is_serving: bool,
    is_primary: bool,
    optional: OptionalBoltData,
) -> Entity {
    let mut entity = world.spawn(core);

    // Role marker + cleanup
    if is_primary {
        entity.insert((PrimaryBolt, CleanupOnRunEnd));
    } else {
        entity.insert((ExtraBolt, CleanupOnNodeExit));
    }

    // Serving marker
    if is_serving {
        entity.insert(BoltServing);
    }

    // Optional bolt definition params from definition()
    if let Some(def_params) = optional.definition_params {
        entity.insert((
            BoltBaseDamage(def_params.base_damage),
            BoltDefinitionRef(def_params.name),
            BoltAngleSpread(def_params.angle_spread),
            BoltSpawnOffsetY(def_params.spawn_offset_y),
        ));
        if let Some(min_r) = def_params.min_radius {
            entity.insert(MinRadius(min_r));
        }
        if let Some(max_r) = def_params.max_radius {
            entity.insert(MaxRadius(max_r));
        }
    }

    // Override individual definition-derived components if explicit .with_*() was called
    if let Some(base_damage) = optional.override_base_damage {
        entity.insert(BoltBaseDamage(base_damage));
    }
    if let Some(name) = optional.override_definition_name {
        entity.insert(BoltDefinitionRef(name));
    }
    if let Some(angle_spread) = optional.override_angle_spread {
        entity.insert(BoltAngleSpread(angle_spread));
    }
    if let Some(spawn_offset_y) = optional.override_spawn_offset_y {
        entity.insert(BoltSpawnOffsetY(spawn_offset_y));
    }

    // Optional: spawned_by
    if let Some(name) = optional.spawned_by {
        entity.insert(SpawnedByEvolution(name));
    }

    // Optional: lifespan
    if let Some(duration) = optional.lifespan {
        entity.insert(BoltLifespan(Timer::from_seconds(duration, TimerMode::Once)));
    }

    // Effect components — spawn-time only
    // Explicit effects first, inherited effects appended
    let has_explicit = optional.with_effects.is_some();
    let has_inherited = optional.inherited_effects.is_some();
    if has_explicit || has_inherited {
        let mut effect_entries: Vec<(String, EffectNode)> = Vec::new();
        if let Some(with_effects) = optional.with_effects {
            effect_entries.extend(with_effects);
        }
        if let Some(inherited) = optional.inherited_effects {
            effect_entries.extend(inherited.0);
        }
        entity.insert(BoundEffects(effect_entries));
    }

    entity.id()
}

// ── Headless build() terminal impls ────────────────────────────────────────

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Primary, Headless> {
    /// Builds the core component bundle for a headless primary serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (core, PrimaryBolt, CleanupOnRunEnd, BoltServing)
    }

    /// Spawns a headless primary serving bolt entity with all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(world, core, true, true, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra, Headless> {
    /// Builds the core component bundle for a headless extra serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (core, ExtraBolt, CleanupOnNodeExit, BoltServing)
    }

    /// Spawns a headless extra serving bolt entity with all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(world, core, true, false, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Primary, Headless> {
    /// Builds the core component bundle for a headless primary bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (core, PrimaryBolt, CleanupOnRunEnd)
    }

    /// Spawns a headless primary bolt entity with velocity and all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(world, core, false, true, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Extra, Headless> {
    /// Builds the core component bundle for a headless extra bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (core, ExtraBolt, CleanupOnNodeExit)
    }

    /// Spawns a headless extra bolt entity with velocity and all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(world, core, false, false, self.optional)
    }
}

// ── Rendered build() terminal impls ────────────────────────────────────────

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Primary, Rendered> {
    /// Builds the core component bundle for a rendered primary serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBolt,
            CleanupOnRunEnd,
            BoltServing,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered primary serving bolt entity with all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(world, core, true, true, self.optional);
        world.entity_mut(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Bolt,
        ));
        entity
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra, Rendered> {
    /// Builds the core component bundle for a rendered extra serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBolt,
            CleanupOnNodeExit,
            BoltServing,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered extra serving bolt entity with all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(world, core, true, false, self.optional);
        world.entity_mut(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Bolt,
        ));
        entity
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Primary, Rendered> {
    /// Builds the core component bundle for a rendered primary bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBolt,
            CleanupOnRunEnd,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered primary bolt entity with velocity and all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(world, core, false, true, self.optional);
        world.entity_mut(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Bolt,
        ));
        entity
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Extra, Rendered> {
    /// Builds the core component bundle for a rendered extra bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBolt,
            CleanupOnNodeExit,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered extra bolt entity with velocity and all components.
    pub fn spawn(self, world: &mut World) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base: self.speed.base,
            min: self.speed.min,
            max: self.speed.max,
            h: self.angle.h,
            v: self.angle.v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(world, core, false, false, self.optional);
        world.entity_mut(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Bolt,
        ));
        entity
    }
}
