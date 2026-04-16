//! Decrements `SurvivalTimer.remaining` by delta time each fixed tick.
//! When remaining <= 0.0 and started is true, writes lethal `DamageDealt<Cell>`
//! targeting the turret itself (self-destruct).

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::behaviors::survival::components::{SurvivalTimer, SurvivalTurret},
    prelude::*,
};

type SurvivalTimerQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static mut SurvivalTimer), (With<SurvivalTurret>, Without<Dead>)>;

/// Ticks down survival timers and self-destructs expired turrets.
pub(crate) fn tick_survival_timer(
    time: Res<Time<Fixed>>,
    mut query: SurvivalTimerQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        if !timer.started {
            continue;
        }
        timer.remaining -= dt;
        if timer.remaining <= 0.0 {
            damage_writer.write(DamageDealt {
                dealer:      Some(entity),
                target:      entity,
                amount:      f32::MAX,
                source_chip: None,
                _marker:     PhantomData,
            });
        }
    }
}
