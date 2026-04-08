//! System to animate birthing entities by lerping scale from zero to target.
//!
//! Runs in `FixedUpdate` during `NodeState::AnimateIn` and `NodeState::Playing`.
//! Expects the entity to start with `Scale2D { x: 0.0, y: 0.0 }` (set by the
//! builder's `.birthed()` method or `begin_node_birthing`).

use bevy::prelude::*;

use crate::prelude::*;

/// Ticks [`Birthing`] timers and lerps `Scale2D` from zero toward
/// `Birthing::target_scale`. On completion, restores exact target scale
/// and stashed collision layers, then removes `Birthing`.
pub(crate) fn tick_birthing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Birthing, &mut Scale2D, &mut CollisionLayers)>,
) {
    for (entity, mut birthing, mut scale, mut layers) in &mut query {
        birthing.timer.tick(time.delta());

        let linear = birthing.fraction();
        // Ease-out: fast start, smooth finish — feels snappy and alive
        let t = linear * (2.0 - linear);

        if birthing.timer.just_finished() {
            // Snap to exact target — no lerp drift
            scale.x = birthing.target_scale.x;
            scale.y = birthing.target_scale.y;
            *layers = birthing.stashed_layers;
            commands.entity(entity).remove::<Birthing>();
        } else {
            // Ease-out lerp from zero to target
            scale.x = birthing.target_scale.x * t;
            scale.y = birthing.target_scale.y * t;
        }
    }
}
