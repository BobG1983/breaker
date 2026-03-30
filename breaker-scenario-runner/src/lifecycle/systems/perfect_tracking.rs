//! Perfect tracking: positions breaker under bolt with random offset, triggers bumps.

use bevy::prelude::*;
use breaker::{
    breaker::{messages::BumpGrade, resources::ForceBumpGrade},
    input::resources::InputActions,
};
use rand::{Rng, prelude::IndexedRandom};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    input::map_action,
    types::{
        BreakerTrackingQuery, PERFECT_TRACKING_BUMP_THRESHOLD, PERFECT_TRACKING_WIDTH_FACTOR,
        ScenarioInputDriver,
    },
};
use crate::{
    input::InputDriver,
    invariants::{ScenarioStats, ScenarioTagBolt},
    types::{BumpMode, GameAction as ScenarioGameAction},
};

/// Positions breaker under bolt with random offset at all times.
///
/// Writes `GameAction::Bump` when:
/// - Bolt is serving (velocity magnitude near zero) -- to launch it
/// - Bolt is descending and within [`PERFECT_TRACKING_BUMP_THRESHOLD`] world
///   units of the breaker
///
/// Bump is suppressed when mode is [`BumpMode::NeverBump`].
/// Only active when [`ScenarioInputDriver`] wraps [`InputDriver::Perfect`].
pub fn apply_perfect_tracking(
    mut driver: Option<ResMut<ScenarioInputDriver>>,
    bolt_query: Query<(&Position2D, &Velocity2D), With<ScenarioTagBolt>>,
    mut breaker_query: BreakerTrackingQuery,
    mut actions: ResMut<InputActions>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let Some(ref mut driver) = driver else {
        return;
    };
    let InputDriver::Perfect(ref mut perfect) = driver.0 else {
        return;
    };

    let Some((bolt_pos, bolt_vel)) = bolt_query.iter().next() else {
        return;
    };
    let bolt_position = bolt_pos.0;
    let bolt_velocity = bolt_vel.0;
    let bolt_is_serving = bolt_velocity.length_squared() < 1.0;

    let mut should_bump = false;

    // Always position breaker under bolt (regardless of bolt direction)
    for (mut breaker_pos, breaker_width) in &mut breaker_query {
        let half_width = breaker_width.half_width();
        let offset = perfect.rng.random_range(
            -PERFECT_TRACKING_WIDTH_FACTOR * half_width
                ..=PERFECT_TRACKING_WIDTH_FACTOR * half_width,
        );
        breaker_pos.0.x = bolt_position.x + offset;

        // Bump when bolt is near breaker and descending
        if bolt_velocity.y < 0.0
            && bolt_position.y > breaker_pos.0.y
            && bolt_position.y - breaker_pos.0.y <= PERFECT_TRACKING_BUMP_THRESHOLD
            && perfect.bump_mode != BumpMode::NeverBump
        {
            should_bump = true;
        }
    }

    // Also bump to launch serving bolt
    if bolt_is_serving && perfect.bump_mode != BumpMode::NeverBump {
        should_bump = true;
    }

    if should_bump {
        actions.0.push(map_action(ScenarioGameAction::Bump));
        if let Some(ref mut s) = stats {
            s.actions_injected += 1;
        }
    }
}

/// Updates [`ForceBumpGrade`] every frame based on `PerfectDriver.bump_mode`.
pub fn update_force_bump_grade(
    mut driver: Option<ResMut<ScenarioInputDriver>>,
    mut force_grade: ResMut<ForceBumpGrade>,
) {
    let Some(ref mut driver) = driver else {
        return;
    };
    let InputDriver::Perfect(ref mut perfect) = driver.0 else {
        return;
    };

    match perfect.bump_mode {
        BumpMode::AlwaysPerfect => force_grade.0 = Some(BumpGrade::Perfect),
        BumpMode::AlwaysEarly => force_grade.0 = Some(BumpGrade::Early),
        BumpMode::AlwaysLate => force_grade.0 = Some(BumpGrade::Late),
        BumpMode::AlwaysWhiff | BumpMode::NeverBump => force_grade.0 = None,
        BumpMode::Random => {
            let choices = [BumpGrade::Early, BumpGrade::Perfect, BumpGrade::Late];
            if let Some(&chosen) = choices.choose(&mut perfect.rng) {
                force_grade.0 = Some(chosen);
            }
        }
    }
}
