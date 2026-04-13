//! `ChainLightningConfig` — chain lightning arcs between cells.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    effect_v3::{components::EffectSourceChip, traits::Fireable},
    state::types::NodeState,
};

/// Configuration for chain lightning that arcs between nearby cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainLightningConfig {
    /// Number of times the lightning jumps between cells.
    pub arcs:        u32,
    /// Maximum distance each arc can jump to find a new target.
    pub range:       OrderedFloat<f32>,
    /// Multiplier applied to base damage for each arc hit.
    pub damage_mult: OrderedFloat<f32>,
    /// How fast each lightning arc travels between cells in world units per second.
    pub arc_speed:   OrderedFloat<f32>,
}

impl Fireable for ChainLightningConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let base_damage = world
            .get::<BoltBaseDamage>(entity)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });

        world.spawn((
            ChainLightningChain {
                remaining_jumps: self.arcs,
                damage:          base_damage * self.damage_mult.0,
                hit_set:         HashSet::new(),
                state:           ChainState::Idle,
                range:           self.range.0,
                arc_speed:       self.arc_speed.0,
                source_pos:      pos,
            },
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::tick_chain_lightning;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_chain_lightning.in_set(EffectV3Systems::Tick),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
        effect_v3::traits::Fireable,
    };

    fn make_config() -> ChainLightningConfig {
        ChainLightningConfig {
            arcs:        3,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(1.5),
            arc_speed:   OrderedFloat(500.0),
        }
    }

    // ── C2: ChainLightning base damage reads BoltBaseDamage from source entity ──

    #[test]
    fn chain_lightning_uses_bolt_base_damage_from_source_entity() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(20.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1, "expected 1 chain lightning entity");
        let expected_damage = 20.0 * 1.5;
        assert!(
            (chains[0].damage - expected_damage).abs() < f32::EPSILON,
            "chain damage should be 20.0 * 1.5 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_zero_bolt_base_damage() {
        let mut world = World::new();
        let source = world
            .spawn((BoltBaseDamage(0.0), Position2D(Vec2::new(50.0, 50.0))))
            .id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            chains[0].damage.abs() < f32::EPSILON,
            "chain damage should be 0.0 * 1.5 = 0.0, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_falls_back_to_default_when_bolt_base_damage_absent() {
        let mut world = World::new();
        let source = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

        make_config().fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.5;
        assert!(
            (chains[0].damage - expected_damage).abs() < f32::EPSILON,
            "chain damage should fall back to DEFAULT_BOLT_BASE_DAMAGE * 1.5 = {expected_damage}, got {}",
            chains[0].damage,
        );
    }

    #[test]
    fn chain_lightning_zero_damage_mult_produces_zero_damage() {
        let mut world = World::new();
        let source = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

        let config = ChainLightningConfig {
            arcs:        3,
            range:       OrderedFloat(100.0),
            damage_mult: OrderedFloat(0.0),
            arc_speed:   OrderedFloat(500.0),
        };
        config.fire(source, "zap", &mut world);
        world.flush();

        let chains: Vec<&ChainLightningChain> =
            world.query::<&ChainLightningChain>().iter(&world).collect();
        assert_eq!(chains.len(), 1);
        assert!(
            chains[0].damage.abs() < f32::EPSILON,
            "chain damage should be 10.0 * 0.0 = 0.0, got {}",
            chains[0].damage,
        );
    }
}
