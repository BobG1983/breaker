//! Commands extension for queuing effect fire/reverse/transfer operations.

use bevy::prelude::*;

use super::super::core::{BoundEffects, EffectKind, EffectNode, StagedEffects};

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

pub(super) struct FireEffectCommand {
    pub(super) entity: Entity,
    pub(super) effect: EffectKind,
    pub(super) source_chip: String,
}

impl Command for FireEffectCommand {
    fn apply(self, world: &mut World) {
        self.effect.fire(self.entity, &self.source_chip, world);
    }
}

pub(super) struct ReverseEffectCommand {
    pub(super) entity: Entity,
    pub(super) effect: EffectKind,
    pub(super) source_chip: String,
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
    pub(super) entity: Entity,
    pub(super) effects: Vec<(String, EffectNode)>,
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

use super::super::core::Target;
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
