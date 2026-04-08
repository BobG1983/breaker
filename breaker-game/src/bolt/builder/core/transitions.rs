use bevy::prelude::*;

use super::types::*;
use crate::{
    bolt::{
        components::Bolt,
        definition::BoltDefinition,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    prelude::*,
};

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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn with_angle(
        self,
        min_angle_h: f32,
        min_angle_v: f32,
    ) -> BoltBuilder<P, S, HasAngle, M, R, V> {
        BoltBuilder {
            position: self.position,
            speed: self.speed,
            angle: HasAngle {
                min_angle_h,
                min_angle_v,
            },
            motion: self.motion,
            role: self.role,
            visual: self.visual,
            optional: self.optional,
        }
    }
}

// ── Motion transitions ──────────────────────────────────────────────────────

impl<P, S, A, R, V> BoltBuilder<P, S, A, NoMotion, R, V> {
    #[must_use]
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

    #[must_use]
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
    #[must_use]
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

    #[must_use]
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
    #[must_use]
    pub fn rendered(
        self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> BoltBuilder<P, S, A, M, R, Rendered> {
        let color_rgb = self.optional.color_rgb.unwrap_or(DEFAULT_BOLT_COLOR_RGB);
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
    #[must_use]
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
    #[must_use]
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
                min_angle_h: def.min_angle_horizontal.to_radians(),
                min_angle_v: def.min_angle_vertical.to_radians(),
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
    #[must_use]
    pub fn spawned_by(mut self, name: &str) -> Self {
        self.optional.spawned_by = Some(name.to_string());
        self
    }

    #[must_use]
    pub const fn with_lifespan(mut self, duration: f32) -> Self {
        self.optional.lifespan = Some(duration);
        self
    }

    #[must_use]
    pub const fn with_radius(mut self, r: f32) -> Self {
        self.optional.radius = Some(r);
        self
    }

    #[must_use]
    pub fn with_inherited_effects(mut self, effects: &BoundEffects) -> Self {
        self.optional.inherited_effects = Some(effects.clone());
        self
    }

    #[must_use]
    pub fn with_effects(mut self, nodes: Vec<(String, EffectNode)>) -> Self {
        self.optional.with_effects = Some(nodes);
        self
    }

    #[must_use]
    pub const fn with_base_damage(mut self, damage: f32) -> Self {
        self.optional.override_base_damage = Some(damage);
        self
    }

    #[must_use]
    pub fn with_definition_name(mut self, name: String) -> Self {
        self.optional.override_definition_name = Some(name);
        self
    }

    #[must_use]
    pub const fn with_angle_spread(mut self, spread: f32) -> Self {
        self.optional.override_angle_spread = Some(spread);
        self
    }

    #[must_use]
    pub const fn with_spawn_offset_y(mut self, offset: f32) -> Self {
        self.optional.override_spawn_offset_y = Some(offset);
        self
    }

    /// Marks this bolt for birthing animation on spawn.
    ///
    /// When spawned, the entity starts with zeroed `Scale2D`, `PreviousScale`,
    /// and `CollisionLayers`. A [`Birthing`] component is inserted that drives
    /// the scale lerp and layer restoration.
    #[must_use]
    pub const fn birthed(mut self) -> Self {
        self.optional.birthed = true;
        self
    }
}
