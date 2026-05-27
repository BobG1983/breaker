//! `SpawnPhantomConfig` — spawn phantom bolt with limited lifetime.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{
    bolt::{
        builder::DEFAULT_BOLT_COLOR_RGB,
        components::{LifetimeEndBehavior, PhantomBolt, PhantomDedupKey, PhantomParams},
    },
    effect_v3::traits::Fireable,
    prelude::*,
};

/// Default base speed for phantom bolts spawned from chip effects. Matches the
/// standard `BoltDefinition::base_speed`; no registry is available inside
/// `Fireable::fire` so the values are hardcoded here.
const DEFAULT_BOLT_BASE_SPEED: f32 = 400.0;
const DEFAULT_BOLT_MIN_SPEED: f32 = 200.0;
const DEFAULT_BOLT_MAX_SPEED: f32 = 800.0;
/// 5 degrees expressed in radians — matches every test fixture's
/// `min_angle_horizontal` / `min_angle_vertical` of 5.0.
const DEFAULT_BOLT_MIN_ANGLE_H_RAD: f32 = 0.087_266_46;
const DEFAULT_BOLT_MIN_ANGLE_V_RAD: f32 = 0.087_266_46;

/// Configuration for spawning a temporary phantom bolt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnPhantomConfig {
    /// How long the phantom bolt exists before despawning.
    pub duration:   OrderedFloat<f32>,
    /// Maximum phantom bolts from this source that can exist at once.
    pub max_active: u32,
}

impl Fireable for SpawnPhantomConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Phase 1 — max_active dedup count by (chip, fired_from).
        let existing_count = world
            .query_filtered::<&PhantomDedupKey, With<PhantomBolt>>()
            .iter(world)
            .filter(|key| {
                matches!(
                    key,
                    PhantomDedupKey::Chip { chip, fired_from }
                        if chip == source && *fired_from == entity
                )
            })
            .count();
        if existing_count >= self.max_active as usize {
            return;
        }

        // Phase 2 — read source state via immutable borrows.
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let vel = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);
        let duration = self.duration.0;
        let dedup_key = PhantomDedupKey::Chip {
            chip:       source.to_string(),
            fired_from: entity,
        };

        // Phase 3 — allocate mesh + material handles (must precede `commands`).
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::new(1.0));
        let material =
            world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(ColorMaterial::from_color(Color::linear_rgb(
                    DEFAULT_BOLT_COLOR_RGB[0],
                    DEFAULT_BOLT_COLOR_RGB[1],
                    DEFAULT_BOLT_COLOR_RGB[2],
                )));

        // Phase 4 — builder chain. Hardcoded speed/angle defaults (no
        // BoltRegistry available inside Fireable::fire).
        let mut commands = world.commands();
        Bolt::builder()
            .at_position(pos)
            .with_speed(
                DEFAULT_BOLT_BASE_SPEED,
                DEFAULT_BOLT_MIN_SPEED,
                DEFAULT_BOLT_MAX_SPEED,
            )
            .with_angle(DEFAULT_BOLT_MIN_ANGLE_H_RAD, DEFAULT_BOLT_MIN_ANGLE_V_RAD)
            .with_velocity(Velocity2D(vel))
            .extra()
            .phantom(PhantomParams { dedup_key })
            .with_lifespan(duration)
            .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
            .rendered_handles(mesh, material)
            .spawn(&mut commands);
    }
}
