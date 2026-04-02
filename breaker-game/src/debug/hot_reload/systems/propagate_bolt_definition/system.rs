//! System to propagate `BoltDefinition` registry changes to bolt entity components.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, PreviousScale, Scale2D,
};
use tracing::warn;

use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltDefinitionRef},
        registry::BoltRegistry,
    },
    effect::{BoundEffects, EffectCommandsExt, EffectNode, RootEffect, Target},
    shared::size::BaseRadius,
};

/// Bundled system parameters for the bolt definition propagation system.
#[derive(SystemParam)]
pub(crate) struct BoltDefinitionChangeContext<'w, 's> {
    /// Bolt registry (rebuilt by `propagate_registry`).
    registry: Res<'w, BoltRegistry>,
    /// Combined query: bolt entities with definition ref + optional [`BoundEffects`].
    /// Single query avoids Bevy query conflict from two overlapping queries.
    bolt_query: Query<
        'w,
        's,
        (
            Entity,
            &'static BoltDefinitionRef,
            Option<&'static mut BoundEffects>,
        ),
        With<Bolt>,
    >,
    /// Command buffer for component inserts and firing Do effects.
    commands: Commands<'w, 's>,
}

/// Detects when `propagate_registry` has rebuilt the `BoltRegistry`
/// and re-stamps definition-derived components on all bolt entities.
///
/// Skips the first frame (when registry `is_added()`) to avoid double-stamping
/// components that were just set by the bolt builder.
pub(crate) fn propagate_bolt_definition(mut ctx: BoltDefinitionChangeContext) {
    if !ctx.registry.is_changed() || ctx.registry.is_added() {
        return;
    }

    for (entity, def_ref, bound_effects) in &mut ctx.bolt_query {
        let Some(def) = ctx.registry.get(&def_ref.0) else {
            warn!(
                "Bolt '{}' not found in registry during hot-reload",
                def_ref.0
            );
            continue;
        };
        let def = def.clone();

        // Re-stamp definition-derived components
        ctx.commands.entity(entity).insert((
            BaseSpeed(def.base_speed),
            MinSpeed(def.min_speed),
            MaxSpeed(def.max_speed),
            BaseRadius(def.radius),
            BoltBaseDamage(def.base_damage),
            Scale2D {
                x: def.radius,
                y: def.radius,
            },
            PreviousScale {
                x: def.radius,
                y: def.radius,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::new(def.radius, def.radius)),
            MinAngleHorizontal(def.min_angle_horizontal.to_radians()),
            MinAngleVertical(def.min_angle_vertical.to_radians()),
        ));

        // Rebuild definition-sourced BoundEffects via direct mutation
        if let Some(mut chains) = bound_effects {
            // Preserve chip-sourced entries (non-empty chip name), remove definition-sourced
            chains.0.retain(|(chip_name, _)| !chip_name.is_empty());

            // Push new definition-sourced entries for Bolt-targeted effects
            for root in &def.effects {
                let RootEffect::On { target, then } = root;
                match target {
                    Target::Bolt => {
                        for child in then {
                            match child {
                                EffectNode::Do(effect) => {
                                    ctx.commands
                                        .fire_effect(entity, effect.clone(), String::new());
                                }
                                other => {
                                    chains.0.push((String::new(), other.clone()));
                                }
                            }
                        }
                    }
                    // Non-bolt targets are not resolved at hot-reload time
                    Target::Breaker
                    | Target::AllBolts
                    | Target::Cell
                    | Target::AllCells
                    | Target::Wall
                    | Target::AllWalls => {}
                }
            }
        }
    }
}
