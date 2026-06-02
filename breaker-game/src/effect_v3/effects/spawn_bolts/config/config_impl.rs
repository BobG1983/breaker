//! `SpawnBoltsConfig` — fire-and-forget bolt spawning.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::BaseSpeed;
use serde::{Deserialize, Serialize};

use crate::{
    bolt::components::{BoltLifespan, ExtraBolt, PrimaryBolt},
    effect_v3::{storage::BoundEffects, traits::Fireable},
    prelude::*,
};

/// Spawns extra bolts at the entity's position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnBoltsConfig {
    /// Number of bolts to spawn.
    pub count:    u32,
    /// Optional duration in seconds before each spawned bolt despawns (None = permanent).
    pub lifespan: Option<OrderedFloat<f32>>,
    /// Whether spawned bolts copy the first primary bolt's effect trees.
    pub inherit:  bool,
}

impl Fireable for SpawnBoltsConfig {
    fn fire(
        &self,
        entity: Entity,
        _source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        if self.count == 0 {
            return;
        }

        // Phase 1: Compute deterministic spread angles.
        //
        // Previous behavior consumed `GameRng` here, but the order in which
        // `walk_bound_effects` visits BoundEffects is determined by Bevy's
        // archetype iteration — which depends on entity insertion order and
        // can vary between processes even with a fixed seed. That made
        // SpawnBolts non-deterministic across separate runs and caused
        // intermittent `evolution_lifecycle` `NoEntityLeaks` violations
        // (W8 §B). A deterministic fan distributes bolts evenly across
        // [-π/2, π/2] (full upper hemisphere) and gives every player the
        // same predictable spread regardless of process scheduling.
        // Phase 2: Read entity state (immutable borrows)
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let base_speed = world.get::<BaseSpeed>(entity).map_or(400.0, |s| s.0);

        let inherited_effects = if self.inherit {
            let mut query = world.query_filtered::<&BoundEffects, With<PrimaryBolt>>();
            query.iter(world).next().cloned()
        } else {
            None
        };

        let lifespan = self.lifespan;

        // Phase 3: Spawn bolts
        for angle in spread_angles(self.count) {
            let vel = Vec2::new(base_speed * angle.sin(), base_speed * angle.cos());
            let birthing = Birthing::new(Scale2D { x: 8.0, y: 8.0 }, CollisionLayers::default());

            let mut bolt_entity =
                world.spawn((Bolt, ExtraBolt, Position2D(pos), Velocity2D(vel), birthing));

            if let Some(duration) = lifespan {
                bolt_entity.insert(BoltLifespan(Timer::from_seconds(
                    duration.0,
                    TimerMode::Once,
                )));
            }

            if let Some(ref effects) = inherited_effects {
                bolt_entity.insert(effects.clone());
            }
        }
    }
}

/// Deterministic angle spread for `SpawnBolts`.
///
/// Distributes `count` bolts across `(-π/2, π/2)` (open interval) so no
/// bolt is fired purely horizontally — the extremes of the original random
/// range produced unplayable horizontal trajectories. For `count == 1` the
/// single bolt fires straight up (angle = 0). For `count >= 2`, angles use
/// the formula `-π/2 + (i + 1)·π/(count + 1)`, giving:
/// - count=2 → [-30°, +30°]
/// - count=3 → [-45°, 0°, +45°]
/// - count=4 → [-54°, -18°, +18°, +54°]
///
/// Iterator order is left-to-right.
fn spread_angles(count: u32) -> impl Iterator<Item = f32> {
    let n = count as f32;
    (0..count).map(move |i| {
        if count == 1 {
            return 0.0;
        }
        let t = (i as f32 + 1.0) / (n + 1.0);
        -FRAC_PI_2 + t * (2.0 * FRAC_PI_2)
    })
}
