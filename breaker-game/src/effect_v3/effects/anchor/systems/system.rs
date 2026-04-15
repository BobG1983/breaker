//! Anchor systems — tick lock/unlock and movement detection.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::components::{AnchorActive, AnchorPlanted, AnchorTimer};
use crate::effect_v3::{effects::piercing::PiercingConfig, stacking::EffectStack};

/// Detects horizontal breaker movement and resets the anchor timer.
///
/// If the breaker is moving horizontally (`velocity.0.x.abs() > f32::EPSILON`),
/// the anchor timer is reset to `plant_delay` and `AnchorPlanted` is removed.
pub fn detect_breaker_movement(
    mut query: Query<(
        Entity,
        &Velocity2D,
        &mut AnchorTimer,
        &AnchorActive,
        Option<&AnchorPlanted>,
    )>,
    mut commands: Commands,
) {
    for (entity, velocity, mut timer, active, planted) in &mut query {
        if velocity.0.x.abs() > f32::EPSILON {
            timer.0 = active.plant_delay;
            if planted.is_some() {
                commands.entity(entity).remove::<AnchorPlanted>();
            }
        }
    }
}

type TickAnchorQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut AnchorTimer,
        Option<&'static mut EffectStack<PiercingConfig>>,
    ),
    Without<AnchorPlanted>,
>;

/// Decrements anchor timer and plants the anchor when it reaches zero.
///
/// On plant, also pushes a piercing charge onto `EffectStack<PiercingConfig>`.
pub fn tick_anchor(mut query: TickAnchorQuery, time: Res<Time>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, mut timer, piercing_stack) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).insert(AnchorPlanted);
            if let Some(mut stack) = piercing_stack {
                stack.push("anchor_piercing".to_owned(), PiercingConfig { charges: 1 });
            } else {
                let mut stack = EffectStack::<PiercingConfig>::default();
                stack.push("anchor_piercing".to_owned(), PiercingConfig { charges: 1 });
                commands.entity(entity).insert(stack);
            }
        }
    }
}
