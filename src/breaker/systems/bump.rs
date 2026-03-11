//! Bump system — input, timing grades, velocity modifiers.

use bevy::prelude::*;

use crate::breaker::components::{Breaker, BreakerState, BreakerStateTimer, BumpState};
use crate::breaker::messages::{BumpGrade, BumpPerformed};
use crate::breaker::resources::BreakerConfig;
use crate::physics::messages::BoltHitBreaker;

/// Updates bump state: handles input, ticks timers, cools down.
pub fn update_bump(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<BreakerConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<&mut BumpState, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for mut bump in &mut query {
        // Tick cooldown
        if bump.cooldown > 0.0 {
            bump.cooldown = (bump.cooldown - dt).max(0.0);
        }

        // Tick active timer
        if bump.active {
            bump.timer -= dt;
            if bump.timer <= 0.0 {
                bump.active = false;
                bump.timer = 0.0;
            }
        }

        // Bump input: Up arrow or W
        if (keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW))
            && !bump.active
            && bump.cooldown <= 0.0
        {
            bump.active = true;
            bump.timer = config.bump_duration;
            bump.cooldown = config.bump_cooldown;
        }
    }
}

/// Grades bump timing on bolt-breaker contact and sends [`BumpPerformed`].
///
/// Consumes [`BoltHitBreaker`] messages to know when a collision occurred.
/// The bolt domain applies the velocity multiplier via a separate system.
pub fn grade_bump(
    config: Res<BreakerConfig>,
    bump_query: Query<&BumpState, With<Breaker>>,
    mut hit_reader: MessageReader<BoltHitBreaker>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    let Ok(bump) = bump_query.single() else {
        return;
    };

    for _hit in hit_reader.read() {
        let grade = calculate_bump_grade(bump, &config);
        writer.write(BumpPerformed { grade });
    }
}

/// Cancels an ongoing dash when a perfect bump is performed.
///
/// Consumes [`BumpPerformed`] messages. When the grade is [`BumpGrade::Perfect`]
/// and the breaker is dashing, transitions directly to Settling.
pub fn perfect_bump_dash_cancel(
    config: Res<BreakerConfig>,
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(&mut BreakerState, &mut BreakerStateTimer), With<Breaker>>,
) {
    for performed in reader.read() {
        if performed.grade != BumpGrade::Perfect {
            continue;
        }

        for (mut state, mut timer) in &mut query {
            if *state == BreakerState::Dashing {
                *state = BreakerState::Settling;
                timer.remaining = config.settle_duration;
            }
        }
    }
}

/// Determines the bump grade based on bump state timing.
fn calculate_bump_grade(bump: &BumpState, config: &BreakerConfig) -> BumpGrade {
    if !bump.active {
        return BumpGrade::None;
    }

    // Time elapsed since bump was activated
    let elapsed = config.bump_duration - bump.timer;

    // Zones: Early → Perfect → Late
    // Early: [0, early_window - perfect_window]
    // Perfect: (early_window - perfect_window, early_window + perfect_window]
    // Late: (early_window + perfect_window, bump_duration]
    if elapsed <= config.early_bump_window - config.perfect_bump_window {
        BumpGrade::Early
    } else if elapsed <= config.early_bump_window + config.perfect_bump_window {
        BumpGrade::Perfect
    } else {
        BumpGrade::Late
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bump_when_inactive() {
        let bump = BumpState {
            active: false,
            timer: 0.0,
            cooldown: 0.0,
        };
        let config = BreakerConfig::default();
        assert_eq!(calculate_bump_grade(&bump, &config), BumpGrade::None);
    }

    #[test]
    fn early_bump_grade() {
        let config = BreakerConfig::default();
        let bump = BumpState {
            active: true,
            timer: config.bump_duration, // just started
            cooldown: 0.0,
        };
        assert_eq!(calculate_bump_grade(&bump, &config), BumpGrade::Early);
    }

    #[test]
    fn perfect_bump_grade() {
        let config = BreakerConfig::default();
        // Timer should be at the sweet spot
        let elapsed = config.early_bump_window;
        let bump = BumpState {
            active: true,
            timer: config.bump_duration - elapsed,
            cooldown: 0.0,
        };
        assert_eq!(calculate_bump_grade(&bump, &config), BumpGrade::Perfect);
    }

    #[test]
    fn late_bump_grade() {
        let config = BreakerConfig::default();
        // Timer near the end — elapsed well past the perfect window
        let bump = BumpState {
            active: true,
            timer: 0.01,
            cooldown: 0.0,
        };
        assert_eq!(calculate_bump_grade(&bump, &config), BumpGrade::Late);
    }

    #[test]
    fn perfect_multiplier_exceeds_others() {
        let config = BreakerConfig::default();
        assert!(config.perfect_bump_multiplier > config.weak_bump_multiplier);
        assert!(config.perfect_bump_multiplier > config.no_bump_multiplier);
    }

    #[derive(Resource)]
    struct TestHitMessage(Option<BoltHitBreaker>);

    fn enqueue_hit(msg_res: Res<TestHitMessage>, mut writer: MessageWriter<BoltHitBreaker>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    #[derive(Resource, Default)]
    struct CapturedBump(Option<BumpPerformed>);

    fn capture_bump(mut reader: MessageReader<BumpPerformed>, mut captured: ResMut<CapturedBump>) {
        for msg in reader.read() {
            captured.0 = Some(msg.clone());
        }
    }

    #[test]
    fn grade_bump_sends_message_on_hit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BumpPerformed>();

        let config = app.world().resource::<BreakerConfig>().clone();

        // Spawn breaker with active bump at the perfect timing
        let elapsed = config.early_bump_window;
        app.world_mut().spawn((
            Breaker,
            BumpState {
                active: true,
                timer: config.bump_duration - elapsed,
                cooldown: 0.0,
            },
        ));

        app.insert_resource(TestHitMessage(Some(BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        })));
        app.init_resource::<CapturedBump>();

        app.add_systems(
            Update,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bump.after(grade_bump),
            ),
        );
        app.update();

        let captured = app.world().resource::<CapturedBump>();
        let msg = captured.0.as_ref().expect("BumpPerformed should have been sent");
        assert_eq!(msg.grade, BumpGrade::Perfect);
    }

    #[derive(Resource)]
    struct TestBumpMessage(Option<BumpPerformed>);

    fn enqueue_bump(msg_res: Res<TestBumpMessage>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    #[test]
    fn perfect_bump_cancels_dash() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.add_message::<BumpPerformed>();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Dashing,
                BreakerStateTimer { remaining: 0.1 },
            ))
            .id();

        app.insert_resource(TestBumpMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(
            Update,
            (
                enqueue_bump.before(perfect_bump_dash_cancel),
                perfect_bump_dash_cancel,
            ),
        );
        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(
            *state,
            BreakerState::Settling,
            "perfect bump during dash should transition to settling"
        );

        let timer = app.world().get::<BreakerStateTimer>(entity).unwrap();
        assert!(
            (timer.remaining - config.settle_duration).abs() < f32::EPSILON,
            "settle timer should be set to config.settle_duration"
        );
    }

    #[test]
    fn bump_zones_cover_full_duration() {
        let config = BreakerConfig::default();
        let early_end = config.early_bump_window - config.perfect_bump_window;
        let perfect_end = config.early_bump_window + config.perfect_bump_window;
        assert!(early_end > 0.0, "early zone should have positive duration");
        assert!(
            perfect_end < config.bump_duration,
            "perfect zone should end before bump expires, leaving room for late"
        );
    }
}
