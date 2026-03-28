//! Commands extension for queuing effect fire/reverse/transfer operations.

use bevy::prelude::*;

use super::core::{BoundEffects, EffectKind, EffectNode, StagedEffects};

/// Extension trait on [`Commands`] for queuing effect operations.
pub trait EffectCommandsExt {
    /// Queue firing an effect on an entity.
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind);
    /// Queue reversing an effect on an entity.
    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind);
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
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind) {
        self.queue(FireEffectCommand { entity, effect });
    }

    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind) {
        self.queue(ReverseEffectCommand { entity, effect });
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
}

impl Command for FireEffectCommand {
    fn apply(self, world: &mut World) {
        self.effect.fire(self.entity, world);
    }
}

struct ReverseEffectCommand {
    entity: Entity,
    effect: EffectKind,
}

impl Command for ReverseEffectCommand {
    fn apply(self, world: &mut World) {
        self.effect.reverse(self.entity, world);
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
            effect.fire(self.entity, world);
        }
    }
}
