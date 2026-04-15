//! `TetherBeamConfig` — damage-dealing beam between two bolts.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Scale2D, Velocity2D};
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::super::components::{TetherBeamDamage, TetherBeamSource, TetherBeamWidth};
use crate::{
    bolt::components::{Bolt, ExtraBolt},
    effect_v3::{components::EffectSourceChip, traits::Fireable},
    shared::{birthing::Birthing, rng::GameRng},
    state::types::NodeState,
};

/// Configuration for a tether beam that links two bolts and damages cells crossing it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TetherBeamConfig {
    /// Multiplier applied to base damage for cells the beam crosses each tick.
    pub damage_mult: OrderedFloat<f32>,
    /// false = spawn a new bolt and beam to it; true = connect existing bolts.
    pub chain:       bool,
    /// Beam half-width in world units (perpendicular distance from the
    /// beam line). Stamped as `TetherBeamWidth` onto the spawned beam
    /// entity. Required — no serde default.
    pub width:       OrderedFloat<f32>,
}

impl Fireable for TetherBeamConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if self.chain {
            self.fire_chain(entity, source, world);
        } else {
            self.fire_spawn(entity, source, world);
        }
    }

    fn register(app: &mut App) {
        use super::super::systems::*;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (tick_tether_beam, cleanup_tether_beams)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl TetherBeamConfig {
    /// Spawn a new bolt and connect it to the source with a tether beam.
    fn fire_spawn(&self, entity: Entity, source: &str, world: &mut World) {
        // Phase 1: Generate random angle (mutable borrow of GameRng)
        let angle: f32 = {
            let mut rng = world.resource_mut::<GameRng>();
            rng.0.random_range(-FRAC_PI_2..=FRAC_PI_2)
        };

        // Phase 2: Read source state
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let base_speed = world.get::<BaseSpeed>(entity).map_or(400.0, |s| s.0);

        let vel = Vec2::new(base_speed * angle.sin(), base_speed * angle.cos());
        let birthing = Birthing::new(Scale2D { x: 8.0, y: 8.0 }, CollisionLayers::default());

        // Phase 3: Spawn new bolt
        let new_bolt = world
            .spawn((Bolt, ExtraBolt, Position2D(pos), Velocity2D(vel), birthing))
            .id();

        let chip = EffectSourceChip::from_source(source);

        // Spawn tether beam source entity (NOT a bolt)
        world.spawn((
            TetherBeamSource {
                bolt_a: entity,
                bolt_b: new_bolt,
            },
            TetherBeamDamage(self.damage_mult.0),
            TetherBeamWidth(self.width.0),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    /// Connect the source bolt to the nearest existing bolt with a tether beam.
    fn fire_chain(&self, entity: Entity, source: &str, world: &mut World) {
        // Read source position
        let source_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        // Find nearest other bolt
        let nearest = world
            .query_filtered::<(Entity, &Position2D), With<Bolt>>()
            .iter(world)
            .filter(|(e, _)| *e != entity)
            .min_by(|(_, pos_a), (_, pos_b)| {
                let dist_a = (pos_a.0 - source_pos).length_squared();
                let dist_b = (pos_b.0 - source_pos).length_squared();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(e, _)| e);

        let Some(nearest_bolt) = nearest else {
            #[cfg(debug_assertions)]
            warn!("TetherBeam chain: no other bolt found to connect to");
            return;
        };

        let chip = EffectSourceChip::from_source(source);

        // Spawn tether beam source entity
        world.spawn((
            TetherBeamSource {
                bolt_a: entity,
                bolt_b: nearest_bolt,
            },
            TetherBeamDamage(self.damage_mult.0),
            TetherBeamWidth(self.width.0),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}
