//! Pulse systems — tick cooldown and fire.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use super::components::{PulseEmitter, PulseRing};
use crate::{effect_v3::effects::shockwave::*, state::types::NodeState};

/// Decrements pulse timer each frame and spawns pulse shockwaves when the timer reaches zero.
pub fn tick_pulse(
    mut query: Query<(&mut PulseEmitter, &Position2D)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (mut emitter, pos) in &mut query {
        emitter.timer -= dt;
        if emitter.timer <= 0.0 {
            emitter.timer += emitter.interval;

            // Calculate effective max radius from stacking.
            let stacks_f32 = emitter.stacks.saturating_sub(1) as f32;
            let max_radius = emitter
                .range_per_level
                .mul_add(stacks_f32, emitter.base_range);

            // Spawn a shockwave-like pulse ring at the emitter's position.
            commands.spawn((
                PulseRing,
                ShockwaveSource,
                ShockwaveRadius(0.0),
                ShockwaveMaxRadius(max_radius),
                ShockwaveSpeed(emitter.speed),
                ShockwaveDamaged(HashSet::new()),
                ShockwaveBaseDamage(10.0),
                ShockwaveDamageMultiplier(1.0),
                Position2D(pos.0),
                CleanupOnExit::<NodeState>::default(),
            ));
        }
    }
}
