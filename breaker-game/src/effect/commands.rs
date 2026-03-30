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

/// Inserts [`BoundEffects`] and [`StagedEffects`] on the entity if absent.
///
/// Must be called on a live `EntityWorldMut` (after a successful `get_entity_mut`).
/// Both components are always inserted as a pair.
fn ensure_effect_components(entity_ref: &mut EntityWorldMut<'_>) {
    if entity_ref.get::<BoundEffects>().is_none() {
        entity_ref.insert(BoundEffects::default());
    }
    if entity_ref.get::<StagedEffects>().is_none() {
        entity_ref.insert(StagedEffects::default());
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
            ensure_effect_components(&mut entity_ref);
            if let Some(mut bound) = entity_ref.get_mut::<BoundEffects>() {
                for entry in self.effects {
                    bound.0.push(entry);
                }
            }
        }
    }
}

/// Command that transfers effect children to an entity's [`BoundEffects`] or [`StagedEffects`].
///
/// Splits children into `Do` nodes (fired immediately) and non-`Do` nodes (stored for trigger evaluation).
/// Always inserts both `BoundEffects` and `StagedEffects` on the target entity if absent,
/// regardless of which children are present — matching [`PushBoundEffects`]'s contract.
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
            ensure_effect_components(&mut entity_ref);
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
    use crate::effect::{core::Trigger, effects::damage_boost::ActiveDamageBoosts};

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

    // -- Section II: TransferCommand insert-if-absent bug fix tests ──────────

    #[test]
    fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_when_child() {
        let mut world = World::new();
        let entity = world.spawn(Name::new("test")).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "aegis".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should contain exactly 1 entry"
        );
        assert_eq!(bound.0[0].0, "aegis");
        assert_eq!(
            bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            }
        );

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted as default alongside BoundEffects");
        assert!(
            staged.0.is_empty(),
            "StagedEffects should be empty (default)"
        );
    }

    #[test]
    fn transfer_non_permanent_inserts_staged_effects_when_absent_and_stores_when_child() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "flux".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when absent");
        assert_eq!(
            staged.0.len(),
            1,
            "StagedEffects should contain exactly 1 entry"
        );
        assert_eq!(staged.0[0].0, "flux");
        assert_eq!(
            staged.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }
        );

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted as default alongside StagedEffects");
        assert!(bound.0.is_empty(), "BoundEffects should be empty (default)");
    }

    #[test]
    fn transfer_non_permanent_stores_multiple_non_do_children() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "flux".to_string(),
            children: vec![
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
                EffectNode::Until {
                    trigger: Trigger::TimeExpires(3.0),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
            ],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when absent");
        assert_eq!(
            staged.0.len(),
            2,
            "StagedEffects should contain both When and Until entries"
        );
        assert_eq!(staged.0[0].0, "flux");
        assert_eq!(staged.0[1].0, "flux");
    }

    #[test]
    fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_on_child() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "redirect".to_string(),
            children: vec![EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should contain exactly 1 On entry"
        );
        assert_eq!(bound.0[0].0, "redirect");
        assert_eq!(
            bound.0[0].1,
            EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }
        );

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted as default");
        assert!(staged.0.is_empty());
    }

    #[test]
    fn transfer_permanent_stores_on_child_with_empty_then() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "redirect".to_string(),
            children: vec![EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "On node with empty then should still be stored"
        );
        assert_eq!(
            bound.0[0].1,
            EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![],
            }
        );
    }

    #[test]
    fn transfer_permanent_appends_to_existing_bound_effects() {
        let mut world = World::new();
        let existing_entry = (
            "old_chip".to_string(),
            EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            },
        );
        let entity = world
            .spawn((BoundEffects(vec![existing_entry]), StagedEffects::default()))
            .id();

        let cmd = TransferCommand {
            entity,
            chip_name: "new_chip".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should contain original + new entry"
        );
        assert_eq!(bound.0[0].0, "old_chip", "original entry at index 0");
        assert_eq!(bound.0[1].0, "new_chip", "new entry appended at index 1");
        assert_eq!(
            bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
            }
        );

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert!(
            staged.0.is_empty(),
            "StagedEffects should remain empty after permanent transfer"
        );
    }

    #[test]
    fn transfer_do_children_fire_even_without_bound_effects() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "amp".to_string(),
            children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            permanent: true,
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "Do child should fire even when BoundEffects was absent"
        );

        // Insert-if-absent should happen unconditionally
        assert!(
            world.get::<BoundEffects>(entity).is_some(),
            "BoundEffects should be inserted even when only Do children exist"
        );
        assert!(
            world.get::<StagedEffects>(entity).is_some(),
            "StagedEffects should be inserted even when only Do children exist"
        );
    }

    #[test]
    fn transfer_mixed_do_and_when_children_without_bound_effects() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "overclock".to_string(),
            children: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ],
            permanent: true,
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "Do child should fire regardless of BoundEffects absence"
        );

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "When child should be stored in BoundEffects"
        );
        assert_eq!(bound.0[0].0, "overclock");
        assert_eq!(
            bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }
        );
    }

    #[test]
    fn transfer_mixed_when_before_do_children_both_processed() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        // When comes before Do in the children vec -- order should not matter
        let cmd = TransferCommand {
            entity,
            chip_name: "overclock".to_string(),
            children: vec![
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
            ],
            permanent: true,
        };
        cmd.apply(&mut world);

        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "Do child should fire regardless of order in children vec"
        );

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "When child should be stored in BoundEffects"
        );
    }

    #[test]
    fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_until_child() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "aegis".to_string(),
            children: vec![EffectNode::Until {
                trigger: Trigger::TimeExpires(5.0),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "Until child should be stored in BoundEffects"
        );
        assert_eq!(bound.0[0].0, "aegis");
        assert_eq!(
            bound.0[0].1,
            EffectNode::Until {
                trigger: Trigger::TimeExpires(5.0),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            }
        );
    }

    #[test]
    fn transfer_permanent_stores_until_child_with_zero_duration() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "aegis".to_string(),
            children: vec![EffectNode::Until {
                trigger: Trigger::TimeExpires(0.0),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            1,
            "Until with zero duration should still be stored"
        );
    }

    #[test]
    fn transfer_non_permanent_inserts_staged_effects_when_absent_and_stores_once_child() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "second_wind".to_string(),
            children: vec![EffectNode::Once(vec![EffectNode::Do(
                EffectKind::SecondWind,
            )])],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when absent");
        assert_eq!(
            staged.0.len(),
            1,
            "Once child should be stored in StagedEffects"
        );
        assert_eq!(staged.0[0].0, "second_wind");
        assert_eq!(
            staged.0[0].1,
            EffectNode::Once(vec![EffectNode::Do(EffectKind::SecondWind)])
        );

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted as default alongside StagedEffects");
        assert!(bound.0.is_empty());
    }

    #[test]
    fn transfer_non_permanent_stores_once_child_with_empty_children() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let cmd = TransferCommand {
            entity,
            chip_name: "second_wind".to_string(),
            children: vec![EffectNode::Once(vec![])],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when absent");
        assert_eq!(
            staged.0.len(),
            1,
            "Once with empty children should still be stored"
        );
    }

    #[test]
    fn transfer_entity_with_staged_but_not_bound_inserts_bound_and_stores_permanent_child() {
        let mut world = World::new();
        let entity = world.spawn(StagedEffects::default()).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "asymmetric_a".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent (StagedEffects was present)");
        assert_eq!(
            bound.0.len(),
            1,
            "Permanent child should be stored in BoundEffects"
        );
        assert_eq!(bound.0[0].0, "asymmetric_a");
        assert_eq!(
            bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }
        );
    }

    #[test]
    fn transfer_entity_with_staged_preserves_existing_staged_entries_when_inserting_bound() {
        let mut world = World::new();
        let existing_staged = vec![(
            "pre_existing".to_string(),
            EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
            },
        )];
        let entity = world.spawn(StagedEffects(existing_staged)).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "asymmetric_a".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "Pre-existing StagedEffects entries must not be disturbed"
        );
        assert_eq!(staged.0[0].0, "pre_existing");
    }

    #[test]
    fn transfer_entity_with_bound_but_not_staged_inserts_staged_and_stores_non_permanent_child() {
        let mut world = World::new();
        let entity = world.spawn(BoundEffects::default()).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "asymmetric_b".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted when absent (BoundEffects was present)");
        assert_eq!(
            staged.0.len(),
            1,
            "Non-permanent child should be stored in StagedEffects"
        );
        assert_eq!(staged.0[0].0, "asymmetric_b");
        assert_eq!(
            staged.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }
        );
    }

    #[test]
    fn transfer_entity_with_bound_preserves_existing_bound_entries_when_inserting_staged() {
        let mut world = World::new();
        let existing_bound = vec![(
            "pre_existing".to_string(),
            EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
            },
        )];
        let entity = world.spawn(BoundEffects(existing_bound)).id();

        let cmd = TransferCommand {
            entity,
            chip_name: "asymmetric_b".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
            permanent: false,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "Pre-existing BoundEffects entries must not be disturbed"
        );
        assert_eq!(bound.0[0].0, "pre_existing");
    }

    #[test]
    fn transfer_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        let cmd = TransferCommand {
            entity,
            chip_name: "ghost".to_string(),
            children: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            permanent: true,
        };
        // Should not panic -- the entity-not-found guard handles this
        cmd.apply(&mut world);
    }

    // -- Section III: A1-A3 coverage tests ────────────────────────────────

    #[test]
    fn transfer_permanent_stores_multiple_mixed_non_do_children_in_bound_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let when_node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };
        let until_node = EffectNode::Until {
            trigger: Trigger::TimeExpires(5.0),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        };
        let once_node = EffectNode::Once(vec![EffectNode::Do(EffectKind::DamageBoost(1.0))]);

        let cmd = TransferCommand {
            entity,
            chip_name: "aegis".to_string(),
            children: vec![when_node.clone(), until_node.clone(), once_node.clone()],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted");
        assert_eq!(
            bound.0.len(),
            3,
            "BoundEffects should contain When, Until, and Once entries"
        );
        assert_eq!(bound.0[0].0, "aegis");
        assert_eq!(bound.0[0].1, when_node);
        assert_eq!(bound.0[1].0, "aegis");
        assert_eq!(bound.0[1].1, until_node);
        assert_eq!(bound.0[2].0, "aegis");
        assert_eq!(bound.0[2].1, once_node);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted as default");
        assert!(
            staged.0.is_empty(),
            "StagedEffects should have 0 entries for permanent transfer"
        );
    }

    #[test]
    fn transfer_non_permanent_stores_mixed_non_do_children_in_staged_and_fires_do() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

        let when_node = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        };
        let until_node = EffectNode::Until {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        };
        let do_node = EffectNode::Do(EffectKind::DamageBoost(3.0));

        let cmd = TransferCommand {
            entity,
            chip_name: "flux".to_string(),
            children: vec![when_node.clone(), until_node.clone(), do_node],
            permanent: false,
        };
        cmd.apply(&mut world);

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted");
        assert_eq!(
            staged.0.len(),
            2,
            "StagedEffects should contain When and Until entries"
        );
        assert_eq!(staged.0[0].0, "flux");
        assert_eq!(staged.0[0].1, when_node);
        assert_eq!(staged.0[1].0, "flux");
        assert_eq!(staged.0[1].1, until_node);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted as default");
        assert!(
            bound.0.is_empty(),
            "BoundEffects should have 0 entries for non-permanent transfer"
        );

        // Do child should have been fired immediately
        let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            boosts.0,
            vec![3.0],
            "Do(DamageBoost(3.0)) should have fired, populating ActiveDamageBoosts"
        );
    }

    #[test]
    fn push_bound_effects_inserts_components_when_absent_then_appends() {
        let mut world = World::new();
        let entity = world.spawn(Name::new("bare")).id();

        let effects = vec![
            (
                "chip_a".to_string(),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                },
            ),
            (
                "chip_b".to_string(),
                EffectNode::Until {
                    trigger: Trigger::TimeExpires(3.0),
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ),
        ];

        PushBoundEffects { entity, effects }.apply(&mut world);

        let bound = world
            .get::<BoundEffects>(entity)
            .expect("BoundEffects should be inserted when absent");
        assert_eq!(
            bound.0.len(),
            2,
            "BoundEffects should contain exactly 2 entries"
        );
        assert_eq!(bound.0[0].0, "chip_a");
        assert_eq!(
            bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            }
        );
        assert_eq!(bound.0[1].0, "chip_b");
        assert_eq!(
            bound.0[1].1,
            EffectNode::Until {
                trigger: Trigger::TimeExpires(3.0),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }
        );

        let staged = world
            .get::<StagedEffects>(entity)
            .expect("StagedEffects should be inserted as default by ensure_effect_components");
        assert!(
            staged.0.is_empty(),
            "StagedEffects should be empty (default)"
        );
    }

    #[test]
    fn push_bound_effects_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        PushBoundEffects {
            entity,
            effects: vec![(
                "chip_a".to_string(),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                },
            )],
        }
        .apply(&mut world);
        // no panic = pass
    }

    #[test]
    fn push_bound_effects_appends_to_existing_bound_effects() {
        let mut world = World::new();
        let existing = vec![(
            "existing".to_string(),
            EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
            },
        )];
        let entity = world
            .spawn((BoundEffects(existing), StagedEffects::default()))
            .id();

        PushBoundEffects {
            entity,
            effects: vec![
                (
                    "chip_a".to_string(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBump,
                        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                    },
                ),
                (
                    "chip_b".to_string(),
                    EffectNode::Until {
                        trigger: Trigger::TimeExpires(3.0),
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                ),
            ],
        }
        .apply(&mut world);

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert_eq!(
            bound.0.len(),
            3,
            "BoundEffects should contain 1 existing + 2 appended = 3 entries"
        );
        assert_eq!(bound.0[0].0, "existing", "original entry at index 0");
        assert_eq!(bound.0[1].0, "chip_a", "first appended entry at index 1");
        assert_eq!(bound.0[2].0, "chip_b", "second appended entry at index 2");

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert!(
            staged.0.is_empty(),
            "StagedEffects should remain empty after push_bound_effects"
        );
    }
}
