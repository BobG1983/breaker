//! Typestate builder for bolt entity construction.
//!
//! Entry point: [`Bolt::builder()`]. The builder prevents invalid combinations
//! at compile time via five typestate dimensions: Position, Speed, Angle,
//! Motion, and Role. `build()` and `spawn()` are only available when all
//! dimensions are satisfied.

use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{PreviousScale, Scale2D, Spatial, Velocity2D};

use crate::{
    bolt::{
        components::{
            Bolt, BoltInitialAngle, BoltLifespan, BoltRadius, BoltRespawnAngleSpread,
            BoltRespawnOffsetY, BoltServing, BoltSpawnOffsetY, ExtraBolt, PrimaryBolt,
            SpawnedByEvolution,
        },
        resources::BoltConfig,
    },
    effect::{BoundEffects, EffectNode},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer,
        WALL_LAYER,
    },
};

/// Default bolt radius when neither `config()` nor `with_radius()` is called.
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

// ── Optional data ───────────────────────────────────────────────────────────

#[derive(Default)]
struct OptionalBoltData {
    spawned_by: Option<String>,
    lifespan: Option<f32>,
    radius: Option<f32>,
    inherited_effects: Option<BoundEffects>,
    with_effects: Option<Vec<(String, EffectNode)>>,
    bolt_params: Option<BoltConfigParams>,
}

struct BoltConfigParams {
    spawn_offset_y: f32,
    respawn_offset_y: f32,
    respawn_angle_spread: f32,
    initial_angle: f32,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Typestate builder for bolt entity construction.
pub struct BoltBuilder<P, S, A, M, R> {
    position: P,
    speed: S,
    angle: A,
    motion: M,
    role: R,
    optional: OptionalBoltData,
}

// ── Entry point ─────────────────────────────────────────────────────────────

impl Bolt {
    /// Creates a bolt builder in the unconfigured state.
    #[must_use]
    pub fn builder() -> BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, NoRole> {
        BoltBuilder {
            position: NoPosition,
            speed: NoSpeed,
            angle: NoAngle,
            motion: NoMotion,
            role: NoRole,
            optional: OptionalBoltData::default(),
        }
    }
}

// ── Position transition ─────────────────────────────────────────────────────

impl<S, A, M, R> BoltBuilder<NoPosition, S, A, M, R> {
    pub fn at_position(self, pos: Vec2) -> BoltBuilder<HasPosition, S, A, M, R> {
        BoltBuilder {
            position: HasPosition { pos },
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Speed transition ────────────────────────────────────────────────────────

impl<P, A, M, R> BoltBuilder<P, NoSpeed, A, M, R> {
    pub fn with_speed(self, base: f32, min: f32, max: f32) -> BoltBuilder<P, HasSpeed, A, M, R> {
        BoltBuilder {
            position: self.position,
            speed: HasSpeed { base, min, max },
            angle: self.angle,
            motion: self.motion,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Angle transition ────────────────────────────────────────────────────────

impl<P, S, M, R> BoltBuilder<P, S, NoAngle, M, R> {
    pub fn with_angle(self, h: f32, v: f32) -> BoltBuilder<P, S, HasAngle, M, R> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: HasAngle { h, v },
            motion: self.motion,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Motion transitions ──────────────────────────────────────────────────────

impl<P, S, A, R> BoltBuilder<P, S, A, NoMotion, R> {
    pub fn serving(self) -> BoltBuilder<P, S, A, Serving, R> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: Serving,
            role: self.role,
            optional: self.optional,
        }
    }

    pub fn with_velocity(self, vel: Velocity2D) -> BoltBuilder<P, S, A, HasVelocity, R> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: HasVelocity { vel },
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Role transitions ────────────────────────────────────────────────────────

impl<P, S, A, M> BoltBuilder<P, S, A, M, NoRole> {
    pub fn primary(self) -> BoltBuilder<P, S, A, M, Primary> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: Primary,
            optional: self.optional,
        }
    }

    pub fn extra(self) -> BoltBuilder<P, S, A, M, Extra> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: self.angle,
            motion: self.motion,
            role: Extra,
            optional: self.optional,
        }
    }
}

// ── from_config convenience ─────────────────────────────────────────────────

impl<P, M, R> BoltBuilder<P, NoSpeed, NoAngle, M, R> {
    pub fn config(self, config: &BoltConfig) -> BoltBuilder<P, HasSpeed, HasAngle, M, R> {
        let mut optional = self.optional;
        optional.bolt_params = Some(BoltConfigParams {
            spawn_offset_y: config.spawn_offset_y,
            respawn_offset_y: config.respawn_offset_y,
            respawn_angle_spread: config.respawn_angle_spread,
            initial_angle: config.initial_angle,
        });
        optional.radius = optional.radius.or(Some(config.radius));
        BoltBuilder {
            position: self.position,
            speed: HasSpeed {
                base: config.base_speed,
                min: config.min_speed,
                max: config.max_speed,
            },
            angle: HasAngle {
                h: config.min_angle_horizontal.to_radians(),
                v: config.min_angle_vertical.to_radians(),
            },
            motion: self.motion,
            role: self.role,
            optional,
        }
    }
}

// ── Optional chainable methods (any typestate) ──────────────────────────────

impl<P, S, A, M, R> BoltBuilder<P, S, A, M, R> {
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
/// Returns `(base, spatial, radius_components)` as nested tuple bundles.
fn build_core(params: &CoreParams, optional: &OptionalBoltData) -> impl Bundle + use<> {
    let radius = optional.radius.unwrap_or(DEFAULT_RADIUS);

    let base_components = (
        Bolt,
        params.vel,
        GameDrawLayer::Bolt,
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
        BoltRadius(radius),
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

    // Optional bolt params from config()
    if let Some(params) = optional.bolt_params {
        entity.insert((
            BoltSpawnOffsetY(params.spawn_offset_y),
            BoltRespawnOffsetY(params.respawn_offset_y),
            BoltRespawnAngleSpread(params.respawn_angle_spread),
            BoltInitialAngle(params.initial_angle),
        ));
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

// ── build() terminal impls ──────────────────────────────────────────────────

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Primary> {
    /// Builds the core component bundle for a primary serving bolt.
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

    /// Spawns a primary serving bolt entity with all components.
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

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra> {
    /// Builds the core component bundle for an extra serving bolt.
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

    /// Spawns an extra serving bolt entity with all components.
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

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Primary> {
    /// Builds the core component bundle for a primary bolt with velocity.
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

    /// Spawns a primary bolt entity with velocity and all components.
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

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Extra> {
    /// Builds the core component bundle for an extra bolt with velocity.
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

    /// Spawns an extra bolt entity with velocity and all components.
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
    use rantzsoft_spatial2d::components::{
        BaseSpeed, InterpolateTransform2D, MaxSpeed, MinAngleHorizontal, MinAngleVertical,
        MinSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D, Spatial, Spatial2D,
    };

    use super::*;
    use crate::{
        bolt::components::{
            BoltInitialAngle, BoltLifespan, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY,
            BoltServing, BoltSpawnOffsetY, ExtraBolt, PrimaryBolt, SpawnedByEvolution,
        },
        effect::EffectKind,
        shared::{
            BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd,
            GameDrawLayer, WALL_LAYER,
        },
    };

    // ── Section A: Entry Point and Typestate Dimensions ──────────────────

    // Behavior 1: Bolt::builder() returns a builder in the fully-unconfigured state
    #[test]
    fn bolt_new_returns_unconfigured_builder() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, NoRole> = Bolt::builder();
        // Type annotation compiles successfully — that is the assertion.
    }

    #[test]
    fn bolt_new_twice_produces_independent_builders() {
        let builder_a = Bolt::builder();
        let builder_b = Bolt::builder();
        // Both builders are independent — modifying one does not affect the other.
        let _a = builder_a.at_position(Vec2::new(1.0, 2.0));
        let _b = builder_b.at_position(Vec2::new(3.0, 4.0));
    }

    // Behavior 2: .at_position() transitions Position dimension
    #[test]
    fn at_position_transitions_to_has_position() {
        let _builder: BoltBuilder<HasPosition, NoSpeed, NoAngle, NoMotion, NoRole> =
            Bolt::builder().at_position(Vec2::new(100.0, 250.0));
    }

    #[test]
    fn at_position_stores_position_in_spawn() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(100.0, 250.0))
            .serving()
            .primary()
            .spawn(&mut world);
        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert!(
            (pos.0.x - 100.0).abs() < f32::EPSILON && (pos.0.y - 250.0).abs() < f32::EPSILON,
            "Position2D should be (100.0, 250.0), got {:?}",
            pos.0
        );
    }

    #[test]
    fn at_position_accepts_zero() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert_eq!(pos.0, Vec2::ZERO, "Position2D should be Vec2::ZERO");
    }

    #[test]
    fn at_position_accepts_negative_coordinates() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(-200.0, -100.0))
            .serving()
            .primary()
            .spawn(&mut world);
        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert!(
            (pos.0.x - (-200.0)).abs() < f32::EPSILON && (pos.0.y - (-100.0)).abs() < f32::EPSILON,
            "Position2D should be (-200.0, -100.0), got {:?}",
            pos.0
        );
    }

    // Behavior 3: .with_speed() transitions Speed dimension
    #[test]
    fn with_speed_transitions_to_has_speed() {
        let _builder: BoltBuilder<NoPosition, HasSpeed, NoAngle, NoMotion, NoRole> =
            Bolt::builder().with_speed(400.0, 200.0, 800.0);
    }

    #[test]
    fn with_speed_stores_values_in_spawn() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let base = world
            .get::<BaseSpeed>(entity)
            .expect("entity should have BaseSpeed");
        assert!(
            (base.0 - 400.0).abs() < f32::EPSILON,
            "BaseSpeed should be 400.0, got {}",
            base.0
        );
        let min = world
            .get::<MinSpeed>(entity)
            .expect("entity should have MinSpeed");
        assert!(
            (min.0 - 200.0).abs() < f32::EPSILON,
            "MinSpeed should be 200.0, got {}",
            min.0
        );
        let max = world
            .get::<MaxSpeed>(entity)
            .expect("entity should have MaxSpeed");
        assert!(
            (max.0 - 800.0).abs() < f32::EPSILON,
            "MaxSpeed should be 800.0, got {}",
            max.0
        );
    }

    #[test]
    fn with_speed_equal_min_max_base_fixed_speed() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .with_speed(400.0, 400.0, 400.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let base = world.get::<BaseSpeed>(entity).unwrap();
        let min = world.get::<MinSpeed>(entity).unwrap();
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (base.0 - 400.0).abs() < f32::EPSILON
                && (min.0 - 400.0).abs() < f32::EPSILON
                && (max.0 - 400.0).abs() < f32::EPSILON,
            "All speed values should be 400.0 for fixed-speed bolt"
        );
    }

    // Behavior 4: .with_angle() transitions Angle dimension
    #[test]
    fn with_angle_transitions_to_has_angle() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, HasAngle, NoMotion, NoRole> =
            Bolt::builder().with_angle(0.087, 0.087);
    }

    #[test]
    fn with_angle_stores_values_in_spawn() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let h = world
            .get::<MinAngleHorizontal>(entity)
            .expect("entity should have MinAngleHorizontal");
        assert!(
            (h.0 - 0.087).abs() < f32::EPSILON,
            "MinAngleHorizontal should be 0.087, got {}",
            h.0
        );
        let v = world
            .get::<MinAngleVertical>(entity)
            .expect("entity should have MinAngleVertical");
        assert!(
            (v.0 - 0.087).abs() < f32::EPSILON,
            "MinAngleVertical should be 0.087, got {}",
            v.0
        );
    }

    #[test]
    fn with_angle_zero_valid() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.0, 0.0)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        assert!(
            h.0.abs() < f32::EPSILON && v.0.abs() < f32::EPSILON,
            "Zero angles should produce MinAngleHorizontal(0.0) and MinAngleVertical(0.0)"
        );
    }

    // Behavior 5: .serving() transitions Motion dimension
    #[test]
    fn serving_transitions_to_serving() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, Serving, NoRole> =
            Bolt::builder().serving();
    }

    // Behavior 6: .with_velocity() transitions Motion dimension
    #[test]
    fn with_velocity_transitions_to_has_velocity() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, HasVelocity, NoRole> =
            Bolt::builder().with_velocity(Velocity2D(Vec2::new(102.9, 385.5)));
    }

    #[test]
    fn with_velocity_stores_velocity_in_spawn() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(200.0, 300.0))
            .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
            .extra()
            .spawn(&mut world);
        let vel = world
            .get::<Velocity2D>(entity)
            .expect("entity should have Velocity2D");
        assert!(
            (vel.0.x - 102.9).abs() < f32::EPSILON && (vel.0.y - 385.5).abs() < f32::EPSILON,
            "Velocity2D should be (102.9, 385.5), got {:?}",
            vel.0
        );
    }

    #[test]
    fn with_velocity_zero_valid() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::ZERO))
            .extra()
            .spawn(&mut world);
        // Must also check a non-#[require] component to avoid false pass from stub
        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "entity should have ExtraBolt marker"
        );
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert_eq!(
            vel.0,
            Vec2::ZERO,
            "Velocity2D(Vec2::ZERO) should be valid for non-serving bolt"
        );
    }

    // Behavior 7: .primary() transitions Role dimension
    #[test]
    fn as_primary_transitions_to_primary() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, Primary> =
            Bolt::builder().primary();
    }

    // Behavior 8: .extra() transitions Role dimension
    #[test]
    fn as_extra_transitions_to_extra() {
        let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, Extra> =
            Bolt::builder().extra();
    }

    // ── Section C: config() Convenience ────────────────────────────

    // Behavior 9: config() satisfies Speed + Angle
    #[test]
    fn from_config_transitions_speed_and_angle() {
        let config = BoltConfig::default();
        let _builder: BoltBuilder<NoPosition, HasSpeed, HasAngle, NoMotion, NoRole> =
            Bolt::builder().config(&config);
    }

    #[test]
    fn from_config_stores_default_speed_values() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 400.0).abs() < f32::EPSILON,
            "BaseSpeed from default config should be 400.0, got {}",
            base.0
        );
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!(
            (min.0 - 200.0).abs() < f32::EPSILON,
            "MinSpeed from default config should be 200.0, got {}",
            min.0
        );
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (max.0 - 800.0).abs() < f32::EPSILON,
            "MaxSpeed from default config should be 800.0, got {}",
            max.0
        );
    }

    #[test]
    fn from_config_custom_speed_values_propagate() {
        let config = BoltConfig {
            base_speed: 100.0,
            min_speed: 50.0,
            max_speed: 150.0,
            min_angle_horizontal: 10.0,
            min_angle_vertical: 10.0,
            ..BoltConfig::default()
        };
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 100.0).abs() < f32::EPSILON,
            "BaseSpeed from custom config should be 100.0, got {}",
            base.0
        );
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!(
            (min.0 - 50.0).abs() < f32::EPSILON,
            "MinSpeed from custom config should be 50.0, got {}",
            min.0
        );
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (max.0 - 150.0).abs() < f32::EPSILON,
            "MaxSpeed from custom config should be 150.0, got {}",
            max.0
        );
    }

    // Behavior 10: config() stores bolt-specific params
    #[test]
    fn from_config_stores_bolt_params_default() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius from default config should be 8.0, got {}",
            radius.0
        );
        let spawn_offset = world.get::<BoltSpawnOffsetY>(entity).unwrap();
        assert!(
            (spawn_offset.0 - 30.0).abs() < f32::EPSILON,
            "BoltSpawnOffsetY from default config should be 30.0, got {}",
            spawn_offset.0
        );
        let respawn_offset = world.get::<BoltRespawnOffsetY>(entity).unwrap();
        assert!(
            (respawn_offset.0 - 30.0).abs() < f32::EPSILON,
            "BoltRespawnOffsetY from default config should be 30.0, got {}",
            respawn_offset.0
        );
        let respawn_angle = world.get::<BoltRespawnAngleSpread>(entity).unwrap();
        assert!(
            (respawn_angle.0 - 0.524).abs() < f32::EPSILON,
            "BoltRespawnAngleSpread from default config should be 0.524, got {}",
            respawn_angle.0
        );
        let initial_angle = world.get::<BoltInitialAngle>(entity).unwrap();
        assert!(
            (initial_angle.0 - 0.26).abs() < f32::EPSILON,
            "BoltInitialAngle from default config should be 0.26, got {}",
            initial_angle.0
        );
    }

    #[test]
    fn from_config_stores_bolt_params_custom() {
        let config = BoltConfig {
            radius: 12.0,
            spawn_offset_y: 40.0,
            respawn_offset_y: 35.0,
            respawn_angle_spread: 0.6,
            initial_angle: 0.3,
            ..BoltConfig::default()
        };
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 12.0).abs() < f32::EPSILON,
            "BoltRadius should be 12.0, got {}",
            radius.0
        );
        let spawn_offset = world.get::<BoltSpawnOffsetY>(entity).unwrap();
        assert!(
            (spawn_offset.0 - 40.0).abs() < f32::EPSILON,
            "BoltSpawnOffsetY should be 40.0, got {}",
            spawn_offset.0
        );
        let respawn_offset = world.get::<BoltRespawnOffsetY>(entity).unwrap();
        assert!(
            (respawn_offset.0 - 35.0).abs() < f32::EPSILON,
            "BoltRespawnOffsetY should be 35.0, got {}",
            respawn_offset.0
        );
        let respawn_angle = world.get::<BoltRespawnAngleSpread>(entity).unwrap();
        assert!(
            (respawn_angle.0 - 0.6).abs() < f32::EPSILON,
            "BoltRespawnAngleSpread should be 0.6, got {}",
            respawn_angle.0
        );
        let initial_angle = world.get::<BoltInitialAngle>(entity).unwrap();
        assert!(
            (initial_angle.0 - 0.3).abs() < f32::EPSILON,
            "BoltInitialAngle should be 0.3, got {}",
            initial_angle.0
        );
    }

    // Behavior 11: config() converts angle degrees to radians
    #[test]
    fn from_config_converts_angles_to_radians() {
        let config = BoltConfig {
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            ..BoltConfig::default()
        };
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        let expected_h = 5.0_f32.to_radians();
        assert!(
            (h.0 - expected_h).abs() < 1e-5,
            "MinAngleHorizontal should be {} (5 degrees in radians), got {}",
            expected_h,
            h.0
        );
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        let expected_v = 5.0_f32.to_radians();
        assert!(
            (v.0 - expected_v).abs() < 1e-5,
            "MinAngleVertical should be {} (5 degrees in radians), got {}",
            expected_v,
            v.0
        );
    }

    #[test]
    fn from_config_zero_angles_produce_zero_radians() {
        let config = BoltConfig {
            min_angle_horizontal: 0.0,
            min_angle_vertical: 0.0,
            ..BoltConfig::default()
        };
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);
        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        assert!(
            h.0.abs() < f32::EPSILON,
            "MinAngleHorizontal(0.0) should be 0.0, got {}",
            h.0
        );
        assert!(
            v.0.abs() < f32::EPSILON,
            "MinAngleVertical(0.0) should be 0.0, got {}",
            v.0
        );
    }

    // ── Section D: Optional Chainable Methods ───────────────────────────

    // Behavior 12: .spawned_by() stores evolution attribution
    #[test]
    fn spawned_by_stores_evolution_name() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("mirror_protocol")
            .spawn(&mut world);
        let spawned_by = world
            .get::<SpawnedByEvolution>(entity)
            .expect("entity should have SpawnedByEvolution");
        assert_eq!(
            spawned_by.0, "mirror_protocol",
            "SpawnedByEvolution should be 'mirror_protocol'"
        );
    }

    #[test]
    fn spawned_by_empty_string_accepted() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("")
            .spawn(&mut world);
        let spawned_by = world
            .get::<SpawnedByEvolution>(entity)
            .expect("entity should have SpawnedByEvolution");
        assert_eq!(
            spawned_by.0, "",
            "SpawnedByEvolution should be empty string"
        );
    }

    // Behavior 13: .with_lifespan() stores bolt lifespan timer
    #[test]
    fn with_lifespan_stores_timer() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(3.5)
            .spawn(&mut world);
        let lifespan = world
            .get::<BoltLifespan>(entity)
            .expect("entity should have BoltLifespan");
        assert!(
            (lifespan.0.duration().as_secs_f32() - 3.5).abs() < 1e-3,
            "BoltLifespan timer duration should be ~3.5, got {}",
            lifespan.0.duration().as_secs_f32()
        );
    }

    #[test]
    fn with_lifespan_zero_produces_zero_duration() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(0.0)
            .spawn(&mut world);
        let lifespan = world
            .get::<BoltLifespan>(entity)
            .expect("entity should have BoltLifespan");
        assert!(
            lifespan.0.duration().as_secs_f32().abs() < 1e-3,
            "BoltLifespan with 0.0 should have zero duration, got {}",
            lifespan.0.duration().as_secs_f32()
        );
    }

    // Behavior 14: .with_radius() overrides config-provided radius
    #[test]
    fn with_radius_overrides_config_radius() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(16.0)
            .spawn(&mut world);
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 16.0).abs() < f32::EPSILON,
            "BoltRadius should be 16.0 (overridden), not 8.0, got {}",
            radius.0
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 16.0).abs() < f32::EPSILON && (scale.y - 16.0).abs() < f32::EPSILON,
            "Scale2D should be (16.0, 16.0), got ({}, {})",
            scale.x,
            scale.y
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 16.0).abs() < f32::EPSILON
                && (prev_scale.y - 16.0).abs() < f32::EPSILON,
            "PreviousScale should be (16.0, 16.0), got ({}, {})",
            prev_scale.x,
            prev_scale.y
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 16.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 16.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (16.0, 16.0), got {:?}",
            aabb.half_extents
        );
    }

    #[test]
    fn with_radius_small_value() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(0.5)
            .spawn(&mut world);
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 0.5).abs() < f32::EPSILON,
            "BoltRadius should be 0.5, got {}",
            radius.0
        );
    }

    // Behavior 15: .with_effects() stores effect nodes as BoundEffects
    #[test]
    fn with_effects_stores_bound_effects() {
        let mut world = World::new();
        let effects = vec![(
            "chip_a".to_string(),
            EffectNode::Do(EffectKind::DamageBoost(5.0)),
        )];
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(effects)
            .spawn(&mut world);
        let bound = world
            .get::<BoundEffects>(entity)
            .expect("entity should have BoundEffects");
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should have 1 entry, got {}",
            bound.0.len()
        );
        assert_eq!(
            bound.0[0].0, "chip_a",
            "first entry chip name should be 'chip_a'"
        );
    }

    #[test]
    fn with_effects_empty_vec_inserts_empty_bound_effects() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(vec![])
            .spawn(&mut world);
        let bound = world
            .get::<BoundEffects>(entity)
            .expect("entity should have BoundEffects with empty vec");
        assert!(
            bound.0.is_empty(),
            "BoundEffects should be empty, got {} entries",
            bound.0.len()
        );
    }

    // Behavior 16: .with_effects() and .with_inherited_effects() combine
    #[test]
    fn with_effects_and_inherited_effects_combine() {
        let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
        let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
        let inherited = BoundEffects(vec![("chip_b".to_string(), node_b)]);

        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_effects(vec![("chip_a".to_string(), node_a)])
            .with_inherited_effects(&inherited)
            .spawn(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("entity should have BoundEffects");
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should have 2 entries (explicit + inherited), got {}",
            bound.0.len()
        );
        // Explicit effects first, inherited appended
        assert_eq!(
            bound.0[0].0, "chip_a",
            "first entry should be explicit 'chip_a'"
        );
        assert_eq!(
            bound.0[1].0, "chip_b",
            "second entry should be inherited 'chip_b'"
        );
    }

    #[test]
    fn inherited_effects_before_with_effects_same_result() {
        let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
        let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
        let inherited = BoundEffects(vec![("chip_b".to_string(), node_b)]);

        let mut world = World::new();
        // Order reversed: with_inherited_effects BEFORE with_effects
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .with_effects(vec![("chip_a".to_string(), node_a)])
            .spawn(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("entity should have BoundEffects");
        assert_eq!(bound.0.len(), 2);
        // Same result regardless of call order: explicit first, inherited appended
        assert_eq!(bound.0[0].0, "chip_a", "explicit always first");
        assert_eq!(bound.0[1].0, "chip_b", "inherited always second");
    }

    // Behavior 17: Optional methods can be called in any order
    #[test]
    fn optional_methods_any_order() {
        let config = BoltConfig::default();
        let mut world = World::new();
        let entity = Bolt::builder()
            .spawned_by("test")
            .with_lifespan(2.0)
            .with_radius(10.0)
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .extra()
            .spawn(&mut world);

        assert!(
            world.get::<SpawnedByEvolution>(entity).is_some(),
            "SpawnedByEvolution should be present"
        );
        assert!(
            world.get::<BoltLifespan>(entity).is_some(),
            "BoltLifespan should be present"
        );
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 10.0).abs() < f32::EPSILON,
            "BoltRadius should be 10.0, got {}",
            radius.0
        );
    }

    #[test]
    fn no_optional_methods_defaults() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .spawn(&mut world);

        assert!(
            world.get::<SpawnedByEvolution>(entity).is_none(),
            "SpawnedByEvolution should NOT be present when not called"
        );
        assert!(
            world.get::<BoltLifespan>(entity).is_none(),
            "BoltLifespan should NOT be present when not called"
        );
        // BoltRadius should still be present from from_config with default 8.0
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should default to 8.0, got {}",
            radius.0
        );
    }

    // Behavior 18: Optional methods available regardless of typestate
    #[test]
    fn optional_methods_available_in_initial_state() {
        // This test verifies that optional methods compile from any state.
        let _builder = Bolt::builder()
            .spawned_by("test")
            .with_lifespan(1.0)
            .with_radius(5.0)
            .with_effects(vec![]);
    }

    // ── Section E: build() — Component Tuple Output ─────────────────────

    // Behavior 19: build() on a primary serving bolt produces correct components
    #[test]
    fn build_primary_serving_has_bolt_marker() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<Bolt>(entity).is_some(),
            "entity should have Bolt marker"
        );
        assert!(
            world.get::<PrimaryBolt>(entity).is_some(),
            "entity should have PrimaryBolt marker"
        );
        assert!(
            world.get::<BoltServing>(entity).is_some(),
            "entity should have BoltServing marker"
        );
    }

    #[test]
    fn build_primary_serving_has_spatial_markers() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<Spatial>(entity).is_some(),
            "entity should have Spatial marker"
        );
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "entity should have Spatial2D (via Spatial #[require])"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "entity should have InterpolateTransform2D (via Spatial #[require])"
        );
    }

    #[test]
    fn build_primary_serving_has_position() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert!(
            (pos.0.x - 0.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
            "Position2D should be (0.0, 50.0), got {:?}",
            pos.0
        );
        let prev_pos = world
            .get::<PreviousPosition>(entity)
            .expect("entity should have PreviousPosition");
        assert!(
            (prev_pos.0.x - 0.0).abs() < f32::EPSILON && (prev_pos.0.y - 50.0).abs() < f32::EPSILON,
            "PreviousPosition should be (0.0, 50.0), got {:?}",
            prev_pos.0
        );
    }

    #[test]
    fn build_primary_serving_has_zero_velocity() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        // Guard against false pass from stub — check a non-#[require] component
        assert!(
            world.get::<PrimaryBolt>(entity).is_some(),
            "entity should have PrimaryBolt marker from builder"
        );
        let vel = world
            .get::<Velocity2D>(entity)
            .expect("entity should have Velocity2D");
        assert_eq!(
            vel.0,
            Vec2::ZERO,
            "Serving bolt should have Velocity2D(Vec2::ZERO)"
        );
    }

    #[test]
    fn build_primary_serving_has_speed_components() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 400.0).abs() < f32::EPSILON,
            "BaseSpeed should be 400.0"
        );
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!(
            (min.0 - 200.0).abs() < f32::EPSILON,
            "MinSpeed should be 200.0"
        );
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (max.0 - 800.0).abs() < f32::EPSILON,
            "MaxSpeed should be 800.0"
        );
    }

    #[test]
    fn build_primary_serving_has_angle_components() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        let expected_h = 5.0_f32.to_radians();
        assert!(
            (h.0 - expected_h).abs() < 1e-5,
            "MinAngleHorizontal should be {expected_h}"
        );
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        let expected_v = 5.0_f32.to_radians();
        assert!(
            (v.0 - expected_v).abs() < 1e-5,
            "MinAngleVertical should be {expected_v}"
        );
    }

    #[test]
    fn build_primary_serving_has_radius_components() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should be 8.0"
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
            "Scale2D should be (8.0, 8.0), got ({}, {})",
            scale.x,
            scale.y
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 8.0).abs() < f32::EPSILON && (prev_scale.y - 8.0).abs() < f32::EPSILON,
            "PreviousScale should be (8.0, 8.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 8.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 8.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (8.0, 8.0)"
        );
    }

    #[test]
    fn build_primary_serving_has_cleanup_on_run_end() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<CleanupOnRunEnd>(entity).is_some(),
            "Primary bolt should have CleanupOnRunEnd"
        );
    }

    #[test]
    fn build_primary_serving_has_collision_layers() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let layers = world
            .get::<CollisionLayers>(entity)
            .expect("entity should have CollisionLayers");
        assert_eq!(
            layers.membership, BOLT_LAYER,
            "membership should be BOLT_LAYER"
        );
        assert_eq!(
            layers.mask,
            CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
            "mask should be CELL|WALL|BREAKER"
        );
    }

    #[test]
    fn build_primary_serving_has_game_draw_layer() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let layer = world
            .get::<GameDrawLayer>(entity)
            .expect("entity should have GameDrawLayer");
        assert!(
            matches!(layer, GameDrawLayer::Bolt),
            "GameDrawLayer should be Bolt"
        );
    }

    // Behavior 20: build() on an extra bolt with velocity produces correct components
    #[test]
    fn build_extra_velocity_has_extra_bolt_marker() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(200.0, 300.0))
            .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<Bolt>(entity).is_some(),
            "entity should have Bolt"
        );
        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "entity should have ExtraBolt"
        );
        assert!(
            world.get::<PrimaryBolt>(entity).is_none(),
            "extra bolt should NOT have PrimaryBolt"
        );
        assert!(
            world.get::<BoltServing>(entity).is_none(),
            "velocity bolt should NOT have BoltServing"
        );
    }

    #[test]
    fn build_extra_velocity_has_explicit_velocity() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(200.0, 300.0))
            .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.0.x - 102.9).abs() < f32::EPSILON && (vel.0.y - 385.5).abs() < f32::EPSILON,
            "Velocity2D should be (102.9, 385.5), got {:?}",
            vel.0
        );
    }

    #[test]
    fn build_extra_velocity_has_cleanup_on_node_exit() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(200.0, 300.0))
            .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<CleanupOnNodeExit>(entity).is_some(),
            "Extra bolt should have CleanupOnNodeExit"
        );
        assert!(
            world.get::<CleanupOnRunEnd>(entity).is_none(),
            "Extra bolt should NOT have CleanupOnRunEnd"
        );
    }

    #[test]
    fn build_extra_velocity_has_spatial_markers() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(200.0, 300.0))
            .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<Spatial>(entity).is_some(),
            "entity should have Spatial"
        );
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "entity should have Spatial2D"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "entity should have InterpolateTransform2D"
        );
    }

    #[test]
    fn build_extra_bolt_at_zero_pos_straight_up() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        let pos = world.get::<Position2D>(entity).unwrap();
        assert_eq!(pos.0, Vec2::ZERO);
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
            "Velocity should be (0.0, 400.0)"
        );
    }

    // Behavior 21: Serving bolt always has Velocity2D(Vec2::ZERO)
    #[test]
    fn serving_bolt_always_zero_velocity() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(999.0, 100.0, 2000.0)
            .with_angle(0.0, 0.0)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        // Guard against false pass from stub — check non-#[require] component
        assert!(
            world.get::<BoltServing>(entity).is_some(),
            "entity should have BoltServing marker from builder"
        );
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert_eq!(
            vel.0,
            Vec2::ZERO,
            "Serving bolt Velocity2D should be Vec2::ZERO regardless of speed config"
        );
    }

    // Behavior 22: Primary bolt gets CleanupOnRunEnd, not CleanupOnNodeExit
    #[test]
    fn primary_bolt_has_cleanup_on_run_end_not_node_exit() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<CleanupOnRunEnd>(entity).is_some(),
            "Primary bolt should have CleanupOnRunEnd"
        );
        assert!(
            world.get::<CleanupOnNodeExit>(entity).is_none(),
            "Primary bolt should NOT have CleanupOnNodeExit"
        );
    }

    // Behavior 23: Extra bolt gets CleanupOnNodeExit, not CleanupOnRunEnd
    #[test]
    fn extra_bolt_has_cleanup_on_node_exit_not_run_end() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<CleanupOnNodeExit>(entity).is_some(),
            "Extra bolt should have CleanupOnNodeExit"
        );
        assert!(
            world.get::<CleanupOnRunEnd>(entity).is_none(),
            "Extra bolt should NOT have CleanupOnRunEnd"
        );
    }

    // Behavior 24: build() uses spatial builder internally
    #[test]
    fn build_uses_spatial_builder_for_velocity_constraints() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(500.0, 100.0, 900.0)
            .with_angle(0.1, 0.2)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<Spatial>(entity).is_some(),
            "entity should have Spatial marker"
        );
        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 500.0).abs() < f32::EPSILON,
            "BaseSpeed should be 500.0"
        );
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!(
            (min.0 - 100.0).abs() < f32::EPSILON,
            "MinSpeed should be 100.0"
        );
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (max.0 - 900.0).abs() < f32::EPSILON,
            "MaxSpeed should be 900.0"
        );
        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        assert!(
            (h.0 - 0.1).abs() < f32::EPSILON,
            "MinAngleHorizontal should be 0.1"
        );
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        assert!(
            (v.0 - 0.2).abs() < f32::EPSILON,
            "MinAngleVertical should be 0.2"
        );
    }

    // Behavior 25: build() without config() still includes BoltRadius default
    #[test]
    fn build_without_from_config_has_default_radius() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should default to 8.0 without config(), got {}",
            radius.0
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
            "Scale2D should be (8.0, 8.0)"
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 8.0).abs() < f32::EPSILON && (prev_scale.y - 8.0).abs() < f32::EPSILON,
            "PreviousScale should be (8.0, 8.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 8.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 8.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (8.0, 8.0)"
        );
    }

    #[test]
    fn build_without_from_config_has_no_bolt_params() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(
            world.get::<BoltSpawnOffsetY>(entity).is_none(),
            "Should NOT have BoltSpawnOffsetY without config()"
        );
        assert!(
            world.get::<BoltRespawnOffsetY>(entity).is_none(),
            "Should NOT have BoltRespawnOffsetY without config()"
        );
        assert!(
            world.get::<BoltRespawnAngleSpread>(entity).is_none(),
            "Should NOT have BoltRespawnAngleSpread without config()"
        );
        assert!(
            world.get::<BoltInitialAngle>(entity).is_none(),
            "Should NOT have BoltInitialAngle without config()"
        );
    }

    // Behavior 26: build() with .with_radius() override
    #[test]
    fn build_with_radius_override_sets_physical_dimensions() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(20.0)
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 20.0).abs() < f32::EPSILON,
            "BoltRadius should be 20.0"
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 20.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
            "Scale2D should be (20.0, 20.0)"
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 20.0).abs() < f32::EPSILON
                && (prev_scale.y - 20.0).abs() < f32::EPSILON,
            "PreviousScale should be (20.0, 20.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 20.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 20.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (20.0, 20.0)"
        );
    }

    #[test]
    fn build_with_radius_zero_no_panic() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(0.0)
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            radius.0.abs() < f32::EPSILON,
            "BoltRadius(0.0) should be accepted"
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
            "Scale2D should be (0.0, 0.0)"
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            prev_scale.x.abs() < f32::EPSILON && prev_scale.y.abs() < f32::EPSILON,
            "PreviousScale should be (0.0, 0.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            aabb.half_extents.x.abs() < f32::EPSILON && aabb.half_extents.y.abs() < f32::EPSILON,
            "Aabb2D half_extents should be (0.0, 0.0)"
        );
    }

    // ── Section F: spawn(&mut World) -> Entity ──────────────────────────

    // Behavior 27: spawn() creates entity with all build components
    #[test]
    fn spawn_primary_serving_creates_complete_entity() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(0.0, 50.0))
            .serving()
            .primary()
            .spawn(&mut world);

        assert!(
            world.get_entity(entity).is_ok(),
            "spawned entity should exist"
        );
        assert!(world.get::<Bolt>(entity).is_some(), "should have Bolt");
        assert!(
            world.get::<PrimaryBolt>(entity).is_some(),
            "should have PrimaryBolt"
        );
        assert!(
            world.get::<BoltServing>(entity).is_some(),
            "should have BoltServing"
        );
        assert!(
            world.get::<Spatial>(entity).is_some(),
            "should have Spatial"
        );
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "should have Spatial2D"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "should have InterpolateTransform2D"
        );

        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            (pos.0.x - 0.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
            "Position2D should be (0.0, 50.0)"
        );
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert_eq!(vel.0, Vec2::ZERO, "serving bolt velocity should be zero");

        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!((base.0 - 400.0).abs() < f32::EPSILON);
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!((min.0 - 200.0).abs() < f32::EPSILON);
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!((max.0 - 800.0).abs() < f32::EPSILON);
        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!((radius.0 - 8.0).abs() < f32::EPSILON);

        assert!(world.get::<CleanupOnRunEnd>(entity).is_some());
        assert!(world.get::<CollisionLayers>(entity).is_some());
        assert!(world.get::<GameDrawLayer>(entity).is_some());
    }

    // Behavior 28: spawn() for extra bolt
    #[test]
    fn spawn_extra_bolt_creates_entity_with_extra_markers() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::new(50.0, 100.0))
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawn(&mut world);

        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "should have ExtraBolt"
        );
        assert!(
            world.get::<CleanupOnNodeExit>(entity).is_some(),
            "should have CleanupOnNodeExit"
        );
        assert!(
            world.get::<PrimaryBolt>(entity).is_none(),
            "should NOT have PrimaryBolt"
        );
        assert!(
            world.get::<Spatial>(entity).is_some(),
            "should have Spatial"
        );
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "should have Spatial2D"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "should have InterpolateTransform2D"
        );

        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
            "Velocity2D should be (0.0, 400.0)"
        );
    }

    // Behavior 29: spawn() with .spawned_by() inserts SpawnedByEvolution
    #[test]
    fn spawn_with_spawned_by_inserts_evolution_marker() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawned_by("chain_bolt")
            .spawn(&mut world);
        let spawned_by = world
            .get::<SpawnedByEvolution>(entity)
            .expect("should have SpawnedByEvolution");
        assert_eq!(spawned_by.0, "chain_bolt");
    }

    // Behavior 30: spawn() with .with_lifespan() inserts BoltLifespan
    #[test]
    fn spawn_with_lifespan_inserts_timer() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_lifespan(5.0)
            .spawn(&mut world);
        let lifespan = world
            .get::<BoltLifespan>(entity)
            .expect("should have BoltLifespan");
        assert!(
            (lifespan.0.duration().as_secs_f32() - 5.0).abs() < 1e-3,
            "BoltLifespan duration should be ~5.0"
        );
    }

    // ── Section G: with_inherited_effects() and Effect Transfer ──────────

    // Behavior 31: with_inherited_effects() stores effects for spawn-time insertion
    #[test]
    fn spawn_with_inherited_effects_inserts_bound_effects() {
        let node_a = EffectNode::Do(EffectKind::DamageBoost(5.0));
        let node_b = EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 });
        let inherited = BoundEffects(vec![
            ("chip_a".to_string(), node_a),
            ("chip_b".to_string(), node_b),
        ]);

        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .spawn(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("should have BoundEffects");
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should have 2 inherited entries"
        );
        assert_eq!(bound.0[0].0, "chip_a");
        assert_eq!(bound.0[1].0, "chip_b");
    }

    #[test]
    fn spawn_with_empty_inherited_effects_inserts_empty_bound_effects() {
        let inherited = BoundEffects(vec![]);

        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .spawn(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("should have BoundEffects even when empty");
        assert!(bound.0.is_empty());
    }

    // Behavior 32: spawn() without inherited/with effects does NOT insert BoundEffects
    #[test]
    fn spawn_without_effects_has_no_bound_effects() {
        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .spawn(&mut world);

        // Guard against false pass — verify a non-#[require] component is present
        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "entity should have ExtraBolt marker from builder"
        );
        assert!(
            world.get::<BoundEffects>(entity).is_none(),
            "entity should NOT have BoundEffects when no effects methods called"
        );
    }

    // Behavior 33: with_inherited_effects() clones effects
    #[test]
    fn inherited_effects_are_cloned_not_moved() {
        let node = EffectNode::Do(EffectKind::DamageBoost(5.0));
        let inherited = BoundEffects(vec![("chip".to_string(), node)]);

        let mut world = World::new();
        let entity = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .with_inherited_effects(&inherited)
            .spawn(&mut world);

        // Original reference is still valid (it was borrowed, not consumed)
        assert_eq!(
            inherited.0.len(),
            1,
            "original BoundEffects should still have its entries"
        );
        assert_eq!(inherited.0[0].0, "chip");

        // Spawned entity also has the effects
        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "chip");
    }

    // ── Section H: Method Ordering Independence ─────────────────────────

    // Behavior 34: Dimensions can be satisfied in any order
    #[test]
    fn dimensions_any_order_extra_velocity() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .extra()
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .with_angle(0.087, 0.087)
            .at_position(Vec2::new(50.0, 50.0))
            .with_speed(400.0, 200.0, 800.0)
            .build();
        let entity = world.spawn(bundle).id();

        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 50.0).abs() < f32::EPSILON,
            "Position2D should be (50.0, 50.0)"
        );
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
            "Velocity2D should be (0.0, 400.0)"
        );
        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "should have ExtraBolt"
        );
        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 400.0).abs() < f32::EPSILON,
            "BaseSpeed should be 400.0"
        );
    }

    #[test]
    fn from_config_in_middle_of_chain() {
        let config = BoltConfig::default();
        let mut world = World::new();
        let bundle = Bolt::builder()
            .primary()
            .config(&config)
            .at_position(Vec2::ZERO)
            .serving()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(world.get::<PrimaryBolt>(entity).is_some());
        assert!(world.get::<BoltServing>(entity).is_some());
        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!((base.0 - 400.0).abs() < f32::EPSILON);
    }

    // Behavior 35: Optional methods interleaved with dimension methods
    #[test]
    fn optional_interleaved_with_dimension_methods() {
        let config = BoltConfig::default();
        let mut world = World::new();
        let bundle = Bolt::builder()
            .spawned_by("test")
            .at_position(Vec2::ZERO)
            .with_lifespan(2.0)
            .config(&config)
            .with_radius(10.0)
            .serving()
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        // Bolt params from config should be present
        assert!(world.get::<BoltRadius>(entity).is_some());
        assert!(world.get::<BaseSpeed>(entity).is_some());
        assert!(world.get::<BoltServing>(entity).is_some());
        assert!(world.get::<ExtraBolt>(entity).is_some());
    }

    // ── Section J: Default Collision Layers and Draw Layer ───────────────

    // Behavior 38: All built bolts have correct CollisionLayers
    #[test]
    fn collision_layers_primary_bolt() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let layers = world.get::<CollisionLayers>(entity).unwrap();
        assert_eq!(layers.membership, BOLT_LAYER);
        assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);
    }

    #[test]
    fn collision_layers_extra_bolt_same_primary() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        let layers = world.get::<CollisionLayers>(entity).unwrap();
        assert_eq!(
            layers.membership, BOLT_LAYER,
            "extra bolt membership should be BOLT_LAYER"
        );
        assert_eq!(
            layers.mask,
            CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
            "extra bolt mask should be CELL|WALL|BREAKER"
        );
    }

    // Behavior 39: All built bolts have GameDrawLayer::Bolt
    #[test]
    fn game_draw_layer_primary() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let layer = world.get::<GameDrawLayer>(entity).unwrap();
        assert!(matches!(layer, GameDrawLayer::Bolt));
    }

    #[test]
    fn game_draw_layer_extra() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .config(&BoltConfig::default())
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        let layer = world.get::<GameDrawLayer>(entity).unwrap();
        assert!(matches!(layer, GameDrawLayer::Bolt));
    }

    // ── Section K: Manual Path Round-Trip ────────────────────────────────

    // Behavior 40: Manual speed + angle path produces correct spatial components
    #[test]
    fn manual_path_produces_correct_spatial_components() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(500.0, 250.0, 750.0)
            .with_angle(0.1, 0.15)
            .at_position(Vec2::new(10.0, 20.0))
            .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        let base = world.get::<BaseSpeed>(entity).unwrap();
        assert!(
            (base.0 - 500.0).abs() < f32::EPSILON,
            "BaseSpeed should be 500.0"
        );
        let min = world.get::<MinSpeed>(entity).unwrap();
        assert!(
            (min.0 - 250.0).abs() < f32::EPSILON,
            "MinSpeed should be 250.0"
        );
        let max = world.get::<MaxSpeed>(entity).unwrap();
        assert!(
            (max.0 - 750.0).abs() < f32::EPSILON,
            "MaxSpeed should be 750.0"
        );
        let h = world.get::<MinAngleHorizontal>(entity).unwrap();
        assert!(
            (h.0 - 0.1).abs() < f32::EPSILON,
            "MinAngleHorizontal should be 0.1"
        );
        let v = world.get::<MinAngleVertical>(entity).unwrap();
        assert!(
            (v.0 - 0.15).abs() < f32::EPSILON,
            "MinAngleVertical should be 0.15"
        );
        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            (pos.0.x - 10.0).abs() < f32::EPSILON && (pos.0.y - 20.0).abs() < f32::EPSILON,
            "Position2D should be (10.0, 20.0)"
        );
        let vel = world.get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.0.x - 200.0).abs() < f32::EPSILON && (vel.0.y - 300.0).abs() < f32::EPSILON,
            "Velocity2D should be (200.0, 300.0)"
        );
        assert!(world.get::<Spatial>(entity).is_some());
        assert!(world.get::<Spatial2D>(entity).is_some());
        assert!(world.get::<InterpolateTransform2D>(entity).is_some());
    }

    #[test]
    fn manual_path_has_no_config_bolt_params() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(500.0, 250.0, 750.0)
            .with_angle(0.1, 0.15)
            .at_position(Vec2::new(10.0, 20.0))
            .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
            .extra()
            .build();
        let entity = world.spawn(bundle).id();

        assert!(world.get::<BoltSpawnOffsetY>(entity).is_none());
        assert!(world.get::<BoltRespawnOffsetY>(entity).is_none());
        assert!(world.get::<BoltRespawnAngleSpread>(entity).is_none());
        assert!(world.get::<BoltInitialAngle>(entity).is_none());

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should default to 8.0"
        );
    }

    // Behavior 41: Manual path with .with_radius() sets physical dimensions
    #[test]
    fn manual_path_with_radius_sets_physical_dimensions() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .with_radius(15.0)
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 15.0).abs() < f32::EPSILON,
            "BoltRadius should be 15.0"
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 15.0).abs() < f32::EPSILON && (scale.y - 15.0).abs() < f32::EPSILON,
            "Scale2D should be (15.0, 15.0)"
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 15.0).abs() < f32::EPSILON
                && (prev_scale.y - 15.0).abs() < f32::EPSILON,
            "PreviousScale should be (15.0, 15.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 15.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 15.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (15.0, 15.0)"
        );
    }

    #[test]
    fn manual_path_without_radius_uses_default() {
        let mut world = World::new();
        let bundle = Bolt::builder()
            .with_speed(400.0, 200.0, 800.0)
            .with_angle(0.087, 0.087)
            .at_position(Vec2::ZERO)
            .serving()
            .primary()
            .build();
        let entity = world.spawn(bundle).id();

        let radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (radius.0 - 8.0).abs() < f32::EPSILON,
            "default BoltRadius should be 8.0"
        );
        let scale = world.get::<Scale2D>(entity).unwrap();
        assert!(
            (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
            "Scale2D should be (8.0, 8.0)"
        );
        let prev_scale = world.get::<PreviousScale>(entity).unwrap();
        assert!(
            (prev_scale.x - 8.0).abs() < f32::EPSILON && (prev_scale.y - 8.0).abs() < f32::EPSILON,
            "PreviousScale should be (8.0, 8.0)"
        );
        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 8.0).abs() < f32::EPSILON
                && (aabb.half_extents.y - 8.0).abs() < f32::EPSILON,
            "Aabb2D half_extents should be (8.0, 8.0)"
        );
    }
}
