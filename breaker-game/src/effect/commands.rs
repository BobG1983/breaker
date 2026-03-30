//! Commands extension for queuing effect fire/reverse/transfer operations.

use bevy::prelude::*;

use super::core::{BoundEffects, EffectKind, EffectNode, StagedEffects};

/// Extension trait on [`Commands`] for queuing effect operations.
pub trait EffectCommandsExt {
    /// Queue firing an effect on an entity.
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String);
    /// Queue reversing an effect on an entity.
    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String);
    /// Queue transferring effect children to an entity's `BoundEffects` or `StagedEffects`.
    fn transfer_effect(
        &mut self,
        entity: Entity,
        chip_name: String,
        children: Vec<EffectNode>,
        permanent: bool,
    );
    /// Queue pushing pre-built effect entries to an entity's [`BoundEffects`],
    /// inserting [`BoundEffects`] and [`StagedEffects`] if absent.
    fn push_bound_effects(&mut self, entity: Entity, effects: Vec<(String, EffectNode)>);
}

impl EffectCommandsExt for Commands<'_, '_> {
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String) {
        self.queue(FireEffectCommand {
            entity,
            effect,
            source_chip,
        });
    }

    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String) {
        self.queue(ReverseEffectCommand {
            entity,
            effect,
            source_chip,
        });
    }

    fn transfer_effect(
        &mut self,
        entity: Entity,
        chip_name: String,
        children: Vec<EffectNode>,
        permanent: bool,
    ) {
        self.queue(TransferCommand {
            entity,
            chip_name,
            children,
            permanent,
        });
    }

    fn push_bound_effects(&mut self, entity: Entity, effects: Vec<(String, EffectNode)>) {
        self.queue(PushBoundEffects { entity, effects });
    }
}

struct FireEffectCommand {
    entity: Entity,
    effect: EffectKind,
    source_chip: String,
}

impl Command for FireEffectCommand {
    fn apply(self, world: &mut World) {
        self.effect.fire(self.entity, &self.source_chip, world);
    }
}

struct ReverseEffectCommand {
    entity: Entity,
    effect: EffectKind,
    source_chip: String,
}

impl Command for ReverseEffectCommand {
    fn apply(self, world: &mut World) {
        self.effect.reverse(self.entity, &self.source_chip, world);
    }
}

/// Custom command that inserts `BoundEffects` + `StagedEffects` if absent,
/// then appends effect entries to the entity's `BoundEffects`.
pub(crate) struct PushBoundEffects {
    entity: Entity,
    effects: Vec<(String, EffectNode)>,
}

impl Command for PushBoundEffects {
    fn apply(self, world: &mut World) {
        if let Ok(mut entity_ref) = world.get_entity_mut(self.entity) {
            if entity_ref.get::<BoundEffects>().is_none() {
                entity_ref.insert(BoundEffects::default());
            }
            if entity_ref.get::<StagedEffects>().is_none() {
                entity_ref.insert(StagedEffects::default());
            }
            if let Some(mut bound) = entity_ref.get_mut::<BoundEffects>() {
                for entry in self.effects {
                    bound.0.push(entry);
                }
            }
        }
    }
}

pub(crate) struct TransferCommand {
    pub(crate) entity: Entity,
    pub(crate) chip_name: String,
    pub(crate) children: Vec<EffectNode>,
    pub(crate) permanent: bool,
}

impl Command for TransferCommand {
    fn apply(self, world: &mut World) {
        let mut do_effects = Vec::new();
        let mut other_children = Vec::new();

        for child in self.children {
            match child {
                EffectNode::Do(effect) => do_effects.push(effect),
                other => other_children.push(other),
            }
        }

        if let Ok(mut entity_ref) = world.get_entity_mut(self.entity) {
            for child in other_children {
                if self.permanent {
                    if let Some(mut bound) = entity_ref.get_mut::<BoundEffects>() {
                        bound.0.push((self.chip_name.clone(), child));
                    }
                } else if let Some(mut staged) = entity_ref.get_mut::<StagedEffects>() {
                    staged.0.push((self.chip_name.clone(), child));
                }
            }
        }

        for effect in do_effects {
            effect.fire(self.entity, &self.chip_name, world);
        }
    }
}

use super::core::Target;
use crate::{
    bolt::components::Bolt, breaker::components::Breaker, cells::components::Cell,
    wall::components::Wall,
};

/// Command that resolves an `On` node: queries entities matching the target,
/// then transfers children to each resolved entity.
pub(crate) struct ResolveOnCommand {
    pub(crate) target: Target,
    pub(crate) chip_name: String,
    pub(crate) children: Vec<EffectNode>,
    pub(crate) permanent: bool,
}

impl Command for ResolveOnCommand {
    fn apply(self, world: &mut World) {
        let entities = resolve_target_from_world(self.target, world);
        for entity in entities {
            TransferCommand {
                entity,
                chip_name: self.chip_name.clone(),
                children: self.children.clone(),
                permanent: self.permanent,
            }
            .apply(world);
        }
    }
}

/// Resolve a [`Target`] to entities using direct world queries.
/// Used by [`ResolveOnCommand`] at command-apply time when system queries
/// are not available.
fn resolve_target_from_world(target: Target, world: &mut World) -> Vec<Entity> {
    match target {
        Target::Breaker => {
            let mut query = world.query_filtered::<Entity, With<Breaker>>();
            query.iter(world).collect()
        }
        Target::Bolt | Target::AllBolts => {
            let mut query = world.query_filtered::<Entity, With<Bolt>>();
            query.iter(world).collect()
        }
        Target::Cell | Target::AllCells => {
            let mut query = world.query_filtered::<Entity, With<Cell>>();
            query.iter(world).collect()
        }
        Target::Wall | Target::AllWalls => {
            let mut query = world.query_filtered::<Entity, With<Wall>>();
            query.iter(world).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect::effects::damage_boost::ActiveDamageBoosts;

    // -- Section I: commands.rs source_chip threading tests ───────────────────

    #[test]
    fn fire_effect_command_passes_source_chip_to_fire() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        let cmd = FireEffectCommand {
            entity,
            effect: EffectKind::DamageBoost(2.0),
            source_chip: "test_chip".to_string(),
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "fire() should have been called — ActiveDamageBoosts should have [2.0]"
        );
    }

    #[test]
    fn fire_effect_command_with_empty_source_chip() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        let cmd = FireEffectCommand {
            entity,
            effect: EffectKind::DamageBoost(2.0),
            source_chip: String::new(),
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "fire() should work with empty source_chip"
        );
    }

    #[test]
    fn reverse_effect_command_passes_source_chip_to_reverse() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();

        let cmd = ReverseEffectCommand {
            entity,
            effect: EffectKind::DamageBoost(2.0),
            source_chip: String::new(),
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert!(
            boosts.0.is_empty(),
            "reverse() should have removed the 2.0 entry — ActiveDamageBoosts should be empty"
        );
    }

    #[test]
    fn fire_effect_extension_queues_command_that_fires_effect() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn(ActiveDamageBoosts(vec![])).id();

        // Queue the fire_effect command via a system
        app.add_systems(Update, move |mut commands: Commands| {
            commands.fire_effect(
                entity,
                EffectKind::DamageBoost(2.0),
                "chip_name".to_string(),
            );
        });

        app.update();

        let boosts = app.world().get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "fire_effect command should have been applied — ActiveDamageBoosts should have [2.0]"
        );
    }

    #[test]
    fn transfer_command_passes_chip_name_to_fire_for_do_children() {
        let mut world = World::new();
        let entity = world
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();

        let cmd = TransferCommand {
            entity,
            chip_name: "transfer_chip".to_string(),
            children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            permanent: true,
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "TransferCommand should fire DamageBoost via chip_name as source_chip"
        );
    }
}
