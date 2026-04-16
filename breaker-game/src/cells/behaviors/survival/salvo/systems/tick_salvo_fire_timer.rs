//! Decrements `SalvoFireTimer.0` by delta time each fixed tick.
//! This system ONLY handles the countdown. Firing logic is in `fire_survival_turret`.

use bevy::prelude::*;

use crate::cells::behaviors::survival::{
    components::SurvivalTurret, salvo::components::SalvoFireTimer,
};

/// Decrements salvo fire timers by delta time.
pub(crate) fn tick_salvo_fire_timer(
    time: Res<Time<Fixed>>,
    mut query: Query<&mut SalvoFireTimer, With<SurvivalTurret>>,
) {
    let dt = time.delta_secs();
    for mut timer in &mut query {
        timer.0 -= dt;
    }
}
