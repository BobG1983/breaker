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

struct TransferCommand {
    entity: Entity,
    chip_name: String,
    children: Vec<EffectNode>,
    permanent: bool,
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
