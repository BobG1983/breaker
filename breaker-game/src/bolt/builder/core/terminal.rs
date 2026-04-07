use bevy::{
    prelude::*,
    time::{Timer, TimerMode},
};
use rantzsoft_spatial2d::components::Spatial;

use super::types::*;
use crate::{
    bolt::components::{
        BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltLifespan, BoltSpawnOffsetY,
        ExtraBolt, PrimaryBolt, SpawnedByEvolution,
    },
    prelude::*,
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER,
        size::{BaseRadius, MaxRadius, MinRadius},
    },
};

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
        .with_clamped_speed(params.base_speed, params.min_speed, params.max_speed)
        .with_clamped_angle(params.min_angle_h, params.min_angle_v)
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
    commands: &mut Commands,
    core: impl Bundle,
    is_serving: bool,
    is_primary: bool,
    optional: OptionalBoltData,
) -> Entity {
    let mut entity = commands.spawn(core);

    // Role marker + cleanup
    if is_primary {
        entity.insert((PrimaryBolt, CleanupOnExit::<RunState>::default()));
    } else {
        entity.insert((ExtraBolt, CleanupOnExit::<NodeState>::default()));
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

    if optional.birthed {
        let target_scale = Scale2D {
            x: optional.radius.unwrap_or(DEFAULT_RADIUS),
            y: optional.radius.unwrap_or(DEFAULT_RADIUS),
        };
        let stashed_layers =
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

        entity.insert((
            Scale2D { x: 0.0, y: 0.0 },
            PreviousScale { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing::new(target_scale, stashed_layers),
        ));
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
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBolt,
            CleanupOnExit::<RunState>::default(),
            BoltServing,
        )
    }

    /// Spawns a headless primary serving bolt entity with all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(commands, core, true, true, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra, Headless> {
    /// Builds the core component bundle for a headless extra serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBolt,
            CleanupOnExit::<NodeState>::default(),
            BoltServing,
        )
    }

    /// Spawns a headless extra serving bolt entity with all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(commands, core, true, false, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Primary, Headless> {
    /// Builds the core component bundle for a headless primary bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (core, PrimaryBolt, CleanupOnExit::<RunState>::default())
    }

    /// Spawns a headless primary bolt entity with velocity and all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(commands, core, false, true, self.optional)
    }
}

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, HasVelocity, Extra, Headless> {
    /// Builds the core component bundle for a headless extra bolt with velocity.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (core, ExtraBolt, CleanupOnExit::<NodeState>::default())
    }

    /// Spawns a headless extra bolt entity with velocity and all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(commands, core, false, false, self.optional)
    }
}

// ── Rendered build() terminal impls ────────────────────────────────────────

impl BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Primary, Rendered> {
    /// Builds the core component bundle for a rendered primary serving bolt.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBolt,
            CleanupOnExit::<RunState>::default(),
            BoltServing,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered primary serving bolt entity with all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, true, true, self.optional);
        commands.entity(entity).insert((
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
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBolt,
            CleanupOnExit::<NodeState>::default(),
            BoltServing,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered extra serving bolt entity with all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: Velocity2D(Vec2::ZERO),
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, true, false, self.optional);
        commands.entity(entity).insert((
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
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBolt,
            CleanupOnExit::<RunState>::default(),
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered primary bolt entity with velocity and all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, false, true, self.optional);
        commands.entity(entity).insert((
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
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBolt,
            CleanupOnExit::<NodeState>::default(),
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
            GameDrawLayer::Bolt,
        )
    }

    /// Spawns a rendered extra bolt entity with velocity and all components.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            base_speed: self.speed.base,
            min_speed: self.speed.min,
            max_speed: self.speed.max,
            min_angle_h: self.angle.min_angle_h,
            min_angle_v: self.angle.min_angle_v,
            vel: self.motion.vel,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, false, false, self.optional);
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Bolt,
        ));
        entity
    }
}
