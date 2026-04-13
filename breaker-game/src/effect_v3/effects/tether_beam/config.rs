//! `TetherBeamConfig` — damage-dealing beam between two bolts.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Scale2D, Velocity2D};
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::{TetherBeamDamage, TetherBeamSource};
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
        use super::systems::*;
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
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::{Bolt, ExtraBolt},
        effect_v3::{
            components::EffectSourceChip,
            effects::tether_beam::components::{TetherBeamDamage, TetherBeamSource},
            traits::Fireable,
        },
        shared::{birthing::Birthing, rng::GameRng},
    };

    fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
        world
            .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
            .id()
    }

    #[test]
    fn fire_spawns_tether_beam_source_entity() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        let beam_count = world
            .query_filtered::<Entity, With<TetherBeamSource>>()
            .iter(&world)
            .count();
        assert!(
            beam_count >= 1,
            "should spawn at least 1 TetherBeamSource entity"
        );
    }

    #[test]
    fn tether_beam_source_references_source_as_bolt_a() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        let beams: Vec<&TetherBeamSource> =
            world.query::<&TetherBeamSource>().iter(&world).collect();
        assert_eq!(beams.len(), 1);
        assert_eq!(
            beams[0].bolt_a, source,
            "bolt_a should be the source entity"
        );
    }

    #[test]
    fn chain_false_spawns_new_bolt_and_connects_beam() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        // A new ExtraBolt should exist
        let extra_bolts: Vec<Entity> = world
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(&world)
            .collect();
        assert_eq!(
            extra_bolts.len(),
            1,
            "chain: false should spawn a new ExtraBolt"
        );

        // The beam's bolt_b should point to the new bolt
        let beams: Vec<&TetherBeamSource> =
            world.query::<&TetherBeamSource>().iter(&world).collect();
        assert_eq!(beams.len(), 1);
        assert_eq!(
            beams[0].bolt_b, extra_bolts[0],
            "bolt_b should be the newly spawned ExtraBolt",
        );
    }

    #[test]
    fn tether_beam_damage_equals_damage_mult_directly() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(2.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        let damages: Vec<f32> = world
            .query::<&TetherBeamDamage>()
            .iter(&world)
            .map(|d| d.0)
            .collect();
        assert_eq!(damages.len(), 1);
        assert!(
            (damages[0] - 2.5).abs() < 1e-3,
            "TetherBeamDamage should be 2.5 (direct config value), got {}",
            damages[0],
        );
    }

    #[test]
    fn tether_beam_source_entity_is_not_a_bolt() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        let beam_bolts = world
            .query_filtered::<Entity, (With<TetherBeamSource>, With<Bolt>)>()
            .iter(&world)
            .count();
        assert_eq!(
            beam_bolts, 0,
            "TetherBeamSource entity should NOT have Bolt marker"
        );
    }

    #[test]
    fn chain_false_spawned_bolt_has_birthing_component() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "tether_beam", &mut world);
        world.flush();

        let birthing_count = world
            .query_filtered::<&Birthing, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            birthing_count, 1,
            "spawned bolt (bolt_b) should have Birthing component"
        );
    }

    // ── Group D — fire() spawn-time chip attachment ────────────────────────

    #[test]
    fn fire_spawn_with_non_empty_source_attaches_chip_some() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "coil_chip", &mut world);
        world.flush();

        let chips: Vec<Option<String>> = world
            .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
            .iter(&world)
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 1, "exactly 1 TetherBeamSource entity expected");
        assert_eq!(chips[0], Some("coil_chip".to_string()));

        // The spawned ExtraBolt must NOT carry an EffectSourceChip.
        let extra_bolt_chip_count = world
            .query_filtered::<&EffectSourceChip, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            extra_bolt_chip_count, 0,
            "EffectSourceChip must be on the TetherBeamSource entity, not the ExtraBolt"
        );
    }

    #[test]
    fn fire_spawn_with_empty_source_attaches_chip_none() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       false,
        };
        config.fire(source, "", &mut world);
        world.flush();

        let chips: Vec<Option<String>> = world
            .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
            .iter(&world)
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 1);
        assert_eq!(
            chips[0], None,
            "empty source string must map to EffectSourceChip(None)"
        );
    }

    #[test]
    fn fire_chain_with_non_empty_source_attaches_chip_some() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
        let _other = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       true,
        };
        config.fire(source, "coil_chip", &mut world);
        world.flush();

        let chips: Vec<Option<String>> = world
            .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
            .iter(&world)
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 1, "exactly 1 TetherBeamSource entity expected");
        assert_eq!(chips[0], Some("coil_chip".to_string()));
    }

    #[test]
    fn fire_chain_with_empty_source_attaches_chip_none() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
        let _other = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       true,
        };
        config.fire(source, "", &mut world);
        world.flush();

        let chips: Vec<Option<String>> = world
            .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
            .iter(&world)
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0], None);
    }

    // ── Group E — fire_chain target selection ──────────────────────────────

    #[test]
    fn fire_chain_picks_nearest_other_bolt_by_squared_distance() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
        let nearest_bolt_entity =
            spawn_source(&mut world, Vec2::new(10.0, 0.0), Vec2::new(0.0, 400.0));
        let _mid = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));
        let _far = spawn_source(&mut world, Vec2::new(200.0, 0.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       true,
        };
        config.fire(source, "coil_chip", &mut world);
        world.flush();

        let beams: Vec<TetherBeamSource> = world
            .query::<&TetherBeamSource>()
            .iter(&world)
            .cloned()
            .collect();
        assert_eq!(beams.len(), 1);
        assert_eq!(beams[0].bolt_a, source);
        assert_eq!(
            beams[0].bolt_b, nearest_bolt_entity,
            "chain target must be the nearest other bolt"
        );
    }

    #[test]
    fn fire_chain_with_only_source_bolt_is_noop() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       true,
        };
        config.fire(source, "coil_chip", &mut world);
        world.flush();

        let beam_count = world.query::<&TetherBeamSource>().iter(&world).count();
        assert_eq!(
            beam_count, 0,
            "no beam should be spawned with only the source bolt"
        );

        let chip_count = world
            .query_filtered::<&EffectSourceChip, With<TetherBeamSource>>()
            .iter(&world)
            .count();
        assert_eq!(
            chip_count, 0,
            "no EffectSourceChip should be spawned either"
        );
    }

    #[test]
    fn fire_chain_with_two_equidistant_bolts_spawns_exactly_one_beam() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
        let right_bolt = spawn_source(&mut world, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));
        let left_bolt = spawn_source(&mut world, Vec2::new(-50.0, 0.0), Vec2::new(0.0, 400.0));

        let config = TetherBeamConfig {
            damage_mult: OrderedFloat(1.5),
            chain:       true,
        };
        config.fire(source, "coil_chip", &mut world);
        world.flush();

        let beams: Vec<TetherBeamSource> = world
            .query::<&TetherBeamSource>()
            .iter(&world)
            .cloned()
            .collect();
        assert_eq!(beams.len(), 1, "exactly 1 beam should be spawned");
        assert_eq!(beams[0].bolt_a, source);
        assert!(
            beams[0].bolt_b == left_bolt || beams[0].bolt_b == right_bolt,
            "beam target must be one of the two equidistant bolts"
        );
    }
}
