//! System that tracks the last bump outcome for debug display.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    debug::resources::LastBumpResult,
};

/// Updates [`LastBumpResult`] from bump messages.
pub(crate) fn track_bump_result(
    mut result: ResMut<LastBumpResult>,
    mut performed: MessageReader<BumpPerformed>,
    mut whiffed: MessageReader<BumpWhiffed>,
) {
    for msg in performed.read() {
        result.0 = match msg.grade {
            BumpGrade::Perfect => "Perfect".into(),
            BumpGrade::Early => "Early".into(),
            BumpGrade::Late => "Late".into(),
        };
    }
    for _ in whiffed.read() {
        result.0 = "Whiff".into();
    }
}
