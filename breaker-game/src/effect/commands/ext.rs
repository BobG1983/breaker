//! Commands extension for queuing effect fire/reverse/transfer operations.

use bevy::prelude::*;

use crate::effect::core::{
    BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger,
    TriggerContext,
};

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
        context: TriggerContext,
    );
    /// Queue pushing pre-built effect entries to an entity's [`BoundEffects`],
    /// inserting [`BoundEffects`] and [`StagedEffects`] if absent.
    fn push_bound_effects(&mut self, entity: Entity, effects: Vec<(String, EffectNode)>);
    /// Queue dispatching initial effects by convention: resolve targets from the
    /// world, fire `Do` effects immediately, store `When`/`Until`/`Once` in
    /// `BoundEffects`, and defer `AllBolts`/`AllCells`/`AllWalls` via wrapping.
    fn dispatch_initial_effects(&mut self, effects: Vec<RootEffect>, source_chip: Option<String>);
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
        context: TriggerContext,
    ) {
        self.queue(TransferCommand {
            entity,
            chip_name,
            children,
            permanent,
            context,
        });
    }

    fn push_bound_effects(&mut self, entity: Entity, effects: Vec<(String, EffectNode)>) {
        self.queue(PushBoundEffects { entity, effects });
    }

    fn dispatch_initial_effects(&mut self, effects: Vec<RootEffect>, source_chip: Option<String>) {
        self.queue(DispatchInitialEffects {
            effects,
            source_chip,
        });
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

/// Command that dispatches initial effects by convention.
///
/// Resolves targets from the world, fires `Do` effects immediately, stores
/// `When`/`Until`/`Once` in `BoundEffects`, and defers `AllBolts`/`AllCells`/`AllWalls`
/// via wrapping with `When(NodeStart, On(target, permanent: true, ...))` on the first breaker.
pub(crate) struct DispatchInitialEffects {
    pub(super) effects: Vec<RootEffect>,
    pub(super) source_chip: Option<String>,
}

impl Command for DispatchInitialEffects {
    fn apply(self, world: &mut World) {
        let chip_name = self.source_chip.unwrap_or_default();

        // Hoist entity resolution — avoid repeated QueryState creation per root effect
        let primary_breakers: Vec<Entity> = {
            let mut q = world.query_filtered::<Entity, (With<Breaker>, With<PrimaryBreaker>)>();
            q.iter(world).collect()
        };
        let primary_bolts: Vec<Entity> = {
            let mut q = world.query_filtered::<Entity, With<PrimaryBolt>>();
            q.iter(world).collect()
        };

        for root in self.effects {
            let RootEffect::On { target, then } = root;

            match target {
                Target::Breaker => {
                    for &entity in &primary_breakers {
                        TransferCommand {
                            entity,
                            chip_name: chip_name.clone(),
                            children: then.clone(),
                            permanent: true,
                            context: TriggerContext::default(),
                        }
                        .apply(world);
                    }
                }
                Target::Bolt => {
                    for &entity in &primary_bolts {
                        TransferCommand {
                            entity,
                            chip_name: chip_name.clone(),
                            children: then.clone(),
                            permanent: true,
                            context: TriggerContext::default(),
                        }
                        .apply(world);
                    }
                }
                Target::Cell | Target::Wall => {
                    // No entities to dispatch to at init time — skip silently
                }
                Target::AllBolts | Target::AllCells | Target::AllWalls => {
                    // Deferred dispatch: wrap and push to first breaker
                    if let Some(&breaker_entity) = primary_breakers.first() {
                        let wrapped = EffectNode::When {
                            trigger: Trigger::NodeStart,
                            then: vec![EffectNode::On {
                                target,
                                permanent: true,
                                then,
                            }],
                        };
                        push_bound_to(breaker_entity, &chip_name, wrapped, world);
                    } else {
                        warn!(
                            "DispatchInitialEffects: no primary breaker found for deferred {:?} dispatch — skipping",
                            target
                        );
                    }
                }
            }
        }
    }
}

/// Pushes a single effect node to an entity's `BoundEffects`, ensuring
/// `BoundEffects` and `StagedEffects` exist.
fn push_bound_to(entity: Entity, chip_name: &str, node: EffectNode, world: &mut World) {
    if let Ok(mut entity_ref) = world.get_entity_mut(entity) {
        ensure_effect_components(&mut entity_ref);
        if let Some(mut bound) = entity_ref.get_mut::<BoundEffects>() {
            bound.0.push((chip_name.to_owned(), node));
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
    pub(crate) context: TriggerContext,
}

impl Command for TransferCommand {
    fn apply(self, world: &mut World) {
        let mut do_effects = Vec::new();
        let mut on_children = Vec::new();
        let mut other_children = Vec::new();

        for child in self.children {
            match child {
                EffectNode::Do(effect) => do_effects.push(effect),
                EffectNode::On {
                    target,
                    permanent,
                    then,
                } => on_children.push((target, permanent, then)),
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

        // Recursively resolve nested On nodes, propagating the original
        // trigger context so same-target chains fully unwrap.
        for (target, permanent, then) in on_children {
            ResolveOnCommand {
                target,
                chip_name: self.chip_name.clone(),
                children: then,
                permanent,
                context: self.context,
            }
            .apply(world);
        }
    }
}

use crate::{bolt::components::PrimaryBolt, breaker::components::PrimaryBreaker, prelude::*};

/// Command that resolves an `On` node: queries entities matching the target,
/// then transfers children to each resolved entity.
///
/// Reads the matching field from [`TriggerContext`] to resolve singular targets
/// to the specific entity involved in the trigger event.
pub(crate) struct ResolveOnCommand {
    pub(crate) target: Target,
    pub(crate) chip_name: String,
    pub(crate) children: Vec<EffectNode>,
    pub(crate) permanent: bool,
    pub(crate) context: TriggerContext,
}

impl Command for ResolveOnCommand {
    fn apply(self, world: &mut World) {
        let entities = match self.target {
            // All* targets always resolve to every entity of that type.
            Target::AllBolts | Target::AllCells | Target::AllWalls => {
                resolve_all(self.target, world)
            }
            // Singular targets: read the matching context field.
            _ => {
                let ctx_entity = match self.target {
                    Target::Bolt => self.context.bolt,
                    Target::Breaker => self.context.breaker,
                    Target::Cell => self.context.cell,
                    Target::Wall => self.context.wall,
                    _ => None,
                };
                match ctx_entity {
                    Some(e) => vec![e],
                    None => resolve_default(self.target, world),
                }
            }
        };
        for entity in entities {
            TransferCommand {
                entity,
                chip_name: self.chip_name.clone(),
                children: self.children.clone(),
                permanent: self.permanent,
                context: self.context,
            }
            .apply(world);
        }
    }
}

/// Resolve an `All*` target to every entity of that type.
fn resolve_all(target: Target, world: &mut World) -> Vec<Entity> {
    match target {
        Target::AllBolts => {
            let mut query = world.query_filtered::<Entity, With<Bolt>>();
            query.iter(world).collect()
        }
        Target::AllCells => {
            let mut query = world.query_filtered::<Entity, With<Cell>>();
            query.iter(world).collect()
        }
        Target::AllWalls => {
            let mut query = world.query_filtered::<Entity, With<Wall>>();
            query.iter(world).collect()
        }
        // Singular targets shouldn't reach here.
        _ => vec![],
    }
}

/// Default resolution for singular targets when no `context_entity` is available.
///
/// - `Bolt` → entities with both `Bolt` and `PrimaryBolt`
/// - `Breaker` → entities with both `Breaker` and `PrimaryBreaker`
/// - `Cell` / `Wall` → no-op (empty vec)
fn resolve_default(target: Target, world: &mut World) -> Vec<Entity> {
    match target {
        Target::Bolt => {
            let mut query = world.query_filtered::<Entity, (With<Bolt>, With<PrimaryBolt>)>();
            query.iter(world).collect()
        }
        Target::Breaker => {
            let mut query = world.query_filtered::<Entity, (With<Breaker>, With<PrimaryBreaker>)>();
            query.iter(world).collect()
        }
        Target::Cell | Target::Wall | Target::AllBolts | Target::AllCells | Target::AllWalls => {
            vec![]
        }
    }
}
