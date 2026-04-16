//! System to cycle phantom cells through Solid, Telegraph, and Ghost phases.

use bevy::prelude::*;

use crate::{
    cells::behaviors::phantom::components::{
        PhantomCell, PhantomConfig, PhantomPhase, PhantomTimer,
    },
    prelude::*,
};

/// Ticks phantom cell timers and transitions between phases.
///
/// Decrements `PhantomTimer.0` by `time.delta_secs()` each fixed timestep.
/// When the timer reaches zero or below, transitions to the next phase via
/// `PhantomPhase::next()`, resets the timer via `PhantomConfig::duration_for()`,
/// and toggles `CollisionLayers` (Ghost: zero; Solid: restore).
pub(crate) fn tick_phantom_phase(
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &mut PhantomPhase,
            &mut PhantomTimer,
            &PhantomConfig,
            &mut CollisionLayers,
        ),
        With<PhantomCell>,
    >,
) {
    let dt = time.delta_secs();
    for (mut phase, mut timer, config, mut layers) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            let old_phase = *phase;
            *phase = phase.next();
            timer.0 = config.duration_for(*phase);
            match old_phase {
                PhantomPhase::Telegraph => {
                    *layers = CollisionLayers::new(0, 0);
                }
                PhantomPhase::Ghost => {
                    *layers = CollisionLayers::new(CELL_LAYER, BOLT_LAYER);
                }
                PhantomPhase::Solid => {}
            }
        }
    }
}
