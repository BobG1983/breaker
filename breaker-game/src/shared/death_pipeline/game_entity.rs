//! `GameEntity` marker trait for entity types that participate in the death pipeline.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    walls::components::Wall,
};

/// Marker trait for entity types that participate in the death pipeline.
///
/// Each `impl` creates a separate Bevy message queue — `DamageDealt<Cell>` and
/// `DamageDealt<Bolt>` are independent message types.
pub(crate) trait GameEntity: Component {}

impl GameEntity for Cell {}
impl GameEntity for Bolt {}
impl GameEntity for Wall {}
impl GameEntity for Breaker {}
impl GameEntity for Salvo {}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::shared::death_pipeline::{
        damage_dealt::DamageDealt, destroyed::Destroyed, kill_yourself::KillYourself,
    };

    // ── Behavior 4: Salvo implements GameEntity — generic types constructible ──

    #[test]
    fn salvo_game_entity_damage_dealt_constructible() {
        let entity = bevy::prelude::Entity::PLACEHOLDER;
        let msg = DamageDealt::<Salvo> {
            dealer:      Some(entity),
            target:      entity,
            amount:      10.0,
            source_chip: None,
            _marker:     PhantomData,
        };
        assert!(
            (msg.amount - 10.0).abs() < f32::EPSILON,
            "DamageDealt<Salvo> should be constructible"
        );
    }

    #[test]
    fn salvo_game_entity_kill_yourself_constructible() {
        let entity = bevy::prelude::Entity::PLACEHOLDER;
        let msg = KillYourself::<Salvo> {
            victim:  entity,
            killer:  None,
            _marker: PhantomData,
        };
        assert_eq!(msg.victim, entity);
    }

    #[test]
    fn salvo_game_entity_destroyed_constructible() {
        let entity = bevy::prelude::Entity::PLACEHOLDER;
        let msg = Destroyed::<Salvo> {
            victim:     entity,
            killer:     None,
            victim_pos: bevy::prelude::Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        };
        assert_eq!(msg.victim, entity);
    }
}
