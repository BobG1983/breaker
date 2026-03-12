//! Bump system — input, timing grades, velocity modifiers.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{
            Breaker, BreakerState, BreakerStateTimer, BumpLateWindow, BumpPerfectCooldown,
            BumpPerfectWindow, BumpState, BumpWeakCooldown, SettleDuration,
        },
        messages::{BumpGrade, BumpPerformed, BumpWhiffed},
        queries::BumpTimingQuery,
    },
    input::resources::{GameAction, InputActions},
    physics::messages::BoltHitBreaker,
};

/// Determines the forward-window grade based on remaining timer.
///
/// Called when the bolt hits while a forward bump is active.
/// Timer counts down from `early_window + perfect_window`.
pub fn forward_grade(timer: f32, perfect_window: f32) -> BumpGrade {
    if timer <= perfect_window {
        BumpGrade::Perfect
    } else {
        BumpGrade::Early
    }
}

/// Determines the retroactive grade based on time elapsed since hit.
///
/// Called when the player presses bump after the bolt has already hit.
pub fn retroactive_grade(time_since_hit: f32, perfect_window: f32) -> BumpGrade {
    if time_since_hit <= perfect_window {
        BumpGrade::Perfect
    } else {
        BumpGrade::Late
    }
}

/// Returns the grade-dependent cooldown duration.
const fn cooldown_for_grade(grade: BumpGrade, perfect_cooldown: f32, weak_cooldown: f32) -> f32 {
    match grade {
        BumpGrade::Perfect => perfect_cooldown,
        BumpGrade::Early | BumpGrade::Late => weak_cooldown,
    }
}

/// Updates bump state: handles input, ticks timers, resolves retroactive bumps.
///
/// Ticks the forward window timer but does not expire it — [`grade_bump`]
/// handles expiry after processing any same-frame hits.
/// Retroactive bumps grade and write immediately on press.
pub fn update_bump(
    actions: Res<InputActions>,
    time: Res<Time<Fixed>>,
    mut query: Query<BumpTimingQuery, With<Breaker>>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    let dt = time.delta_secs();

    for (mut bump, perfect_window, early_window, late_window, perfect_cooldown, weak_cooldown) in
        &mut query
    {
        // Tick cooldown
        if bump.cooldown > 0.0 {
            bump.cooldown = (bump.cooldown - dt).max(0.0);
        }

        // Tick post-hit timer
        if bump.post_hit_timer > 0.0 {
            bump.post_hit_timer = (bump.post_hit_timer - dt).max(0.0);
        }

        // Tick active timer — grade_bump handles expiry
        if bump.active {
            bump.timer -= dt;
        }

        // Bump input
        if actions.active(GameAction::Bump) && bump.cooldown <= 0.0 {
            if bump.post_hit_timer > 0.0 {
                // Retroactive path: bolt already hit, player pressing after
                let time_since_hit = (perfect_window.0 + late_window.0) - bump.post_hit_timer;
                let grade = retroactive_grade(time_since_hit, perfect_window.0);
                writer.write(BumpPerformed { grade });
                bump.cooldown = cooldown_for_grade(grade, perfect_cooldown.0, weak_cooldown.0);
                bump.post_hit_timer = 0.0;
                bump.active = false;
            } else if !bump.active {
                // Forward path: no recent hit, open the window
                bump.active = true;
                bump.timer = early_window.0 + perfect_window.0;
            }
        }
    }
}

/// Grades bump timing on bolt-breaker contact and sends [`BumpPerformed`].
///
/// Must run after `PhysicsSystems::BreakerCollision` to ensure messages are available.
/// If a forward bump is active, grades immediately. Otherwise, sets `post_hit_timer`
/// for the retroactive path in `update_bump`.
///
/// Also expires the forward window when the timer runs out without a hit,
/// sending [`BumpWhiffed`] and setting whiff cooldown.
pub fn grade_bump(
    mut bump_query: Query<
        (
            &mut BumpState,
            &BumpPerfectWindow,
            &BumpLateWindow,
            &BumpPerfectCooldown,
            &BumpWeakCooldown,
        ),
        With<Breaker>,
    >,
    mut hit_reader: MessageReader<BoltHitBreaker>,
    mut writer: MessageWriter<BumpPerformed>,
    mut whiff_writer: MessageWriter<BumpWhiffed>,
) {
    let Ok((mut bump, perfect_window, late_window, perfect_cooldown, weak_cooldown)) =
        bump_query.single_mut()
    else {
        return;
    };

    for _hit in hit_reader.read() {
        if bump.active {
            // Forward path: grade based on timer position
            let grade = forward_grade(bump.timer, perfect_window.0);
            writer.write(BumpPerformed { grade });
            bump.active = false;
            bump.cooldown = cooldown_for_grade(grade, perfect_cooldown.0, weak_cooldown.0);
        } else {
            // No active bump — open retroactive window for update_bump
            bump.post_hit_timer = perfect_window.0 + late_window.0;
        }
    }

    // Forward window expired without a hit — whiff
    if bump.active && bump.timer <= 0.0 {
        bump.active = false;
        bump.timer = 0.0;
        whiff_writer.write(BumpWhiffed);
        bump.cooldown = weak_cooldown.0;
    }
}

/// Cancels an ongoing dash when a perfect bump is performed.
///
/// Consumes [`BumpPerformed`] messages. When the grade is [`BumpGrade::Perfect`]
/// and the breaker is dashing, transitions directly to Settling.
pub fn perfect_bump_dash_cancel(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(&mut BreakerState, &mut BreakerStateTimer, &SettleDuration), With<Breaker>>,
) {
    for performed in reader.read() {
        if performed.grade != BumpGrade::Perfect {
            continue;
        }

        for (mut state, mut timer, settle_duration) in &mut query {
            if *state == BreakerState::Dashing {
                *state = BreakerState::Settling;
                timer.remaining = settle_duration.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::{components::BumpEarlyWindow, resources::BreakerConfig};

    // ── Pure grade helper tests ──────────────────────────────────────

    #[test]
    fn forward_just_activated_is_early() {
        // Timer at max (just pressed) — well above perfect_window
        let grade = forward_grade(0.20, 0.05);
        assert_eq!(grade, BumpGrade::Early);
    }

    #[test]
    fn forward_at_perfect_boundary_is_perfect() {
        let grade = forward_grade(0.05, 0.05);
        assert_eq!(grade, BumpGrade::Perfect);
    }

    #[test]
    fn forward_within_perfect_zone_is_perfect() {
        let grade = forward_grade(0.02, 0.05);
        assert_eq!(grade, BumpGrade::Perfect);
    }

    #[test]
    fn forward_just_outside_perfect_is_early() {
        let grade = forward_grade(0.05 + 0.001, 0.05);
        assert_eq!(grade, BumpGrade::Early);
    }

    #[test]
    fn retroactive_immediate_is_perfect() {
        let grade = retroactive_grade(0.0, 0.05);
        assert_eq!(grade, BumpGrade::Perfect);
    }

    #[test]
    fn retroactive_at_boundary_is_perfect() {
        let grade = retroactive_grade(0.05, 0.05);
        assert_eq!(grade, BumpGrade::Perfect);
    }

    #[test]
    fn retroactive_just_past_boundary_is_late() {
        let grade = retroactive_grade(0.05 + 0.001, 0.05);
        assert_eq!(grade, BumpGrade::Late);
    }

    // ── update_bump integration tests ────────────────────────────────

    #[derive(Resource)]
    struct TestInputActive(bool);

    fn set_bump_action(mut actions: ResMut<InputActions>, active: Res<TestInputActive>) {
        if active.0 {
            actions.0.push(GameAction::Bump);
        }
    }

    #[derive(Resource, Default)]
    struct CapturedBumps(Vec<BumpPerformed>);

    #[derive(Resource, Default)]
    struct CapturedWhiffs(u32);

    fn capture_bumps(
        mut reader: MessageReader<BumpPerformed>,
        mut captured: ResMut<CapturedBumps>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn capture_whiffs(
        mut reader: MessageReader<BumpWhiffed>,
        mut captured: ResMut<CapturedWhiffs>,
    ) {
        for _msg in reader.read() {
            captured.0 += 1;
        }
    }

    fn bump_param_bundle(
        config: &BreakerConfig,
    ) -> (
        BumpPerfectWindow,
        BumpEarlyWindow,
        BumpLateWindow,
        BumpPerfectCooldown,
        BumpWeakCooldown,
        SettleDuration,
    ) {
        (
            BumpPerfectWindow(config.perfect_window),
            BumpEarlyWindow(config.early_window),
            BumpLateWindow(config.late_window),
            BumpPerfectCooldown(config.perfect_bump_cooldown),
            BumpWeakCooldown(config.weak_bump_cooldown),
            SettleDuration(config.settle_duration),
        )
    }

    fn update_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<InputActions>();
        app.add_message::<BumpPerformed>();
        app.add_message::<BumpWhiffed>();
        app.init_resource::<CapturedBumps>();
        app.init_resource::<CapturedWhiffs>();
        app.insert_resource(TestInputActive(false));
        app.add_systems(
            Update,
            (
                set_bump_action.before(update_bump),
                update_bump,
                (capture_bumps, capture_whiffs).after(update_bump),
            ),
        );
        app
    }

    /// Advances `Time<Fixed>` by one default timestep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();
    }

    #[test]
    fn input_opens_forward_window() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((Breaker, BumpState::default(), bump_param_bundle(&config)))
            .id();

        app.insert_resource(TestInputActive(true));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(bump.active);
        assert!(
            (bump.timer - (config.early_window + config.perfect_window)).abs() < 0.02,
            "timer should be near early_window + perfect_window, got {}",
            bump.timer
        );
    }

    #[test]
    fn input_on_cooldown_ignored() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    cooldown: 0.5,
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestInputActive(true));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active, "bump should not activate while on cooldown");
    }

    #[test]
    fn input_while_active_ignored() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.early_window, // mid-window
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        let timer_before = config.early_window;
        app.insert_resource(TestInputActive(true));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(bump.active, "should still be active");
        // Timer should have ticked down, not been reset
        assert!(
            bump.timer < timer_before,
            "timer should tick down, not reset"
        );
    }

    #[test]
    fn forward_window_expiry_sends_whiff_and_sets_cooldown() {
        let mut app = combined_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: 0.001, // about to expire
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestInputActive(false));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active, "should have expired");
        assert!(
            (bump.cooldown - config.weak_bump_cooldown).abs() < f32::EPSILON,
            "whiff should set weak cooldown, got {}",
            bump.cooldown
        );

        let captured = app.world().resource::<CapturedBumps>();
        assert!(captured.0.is_empty(), "no BumpPerformed on whiff");

        let whiffs = app.world().resource::<CapturedWhiffs>();
        assert_eq!(whiffs.0, 1, "should send one BumpWhiffed message");
    }

    #[test]
    fn retroactive_perfect_grades_and_sets_zero_cooldown() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        // post_hit_timer at max — just hit, pressing immediately
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    post_hit_timer: config.perfect_window + config.late_window,
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestInputActive(true));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(
            (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
            "perfect retroactive should set perfect_bump_cooldown ({}), got {}",
            config.perfect_bump_cooldown,
            bump.cooldown
        );
        assert!(bump.post_hit_timer <= 0.0, "should clear post_hit_timer");

        let captured = app.world().resource::<CapturedBumps>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].grade, BumpGrade::Perfect);
    }

    #[test]
    fn retroactive_late_grades_correctly() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        // post_hit_timer low — hit happened a while ago, pressing late
        let remaining = config.late_window * 0.5; // some time left but past perfect
        app.world_mut().spawn((
            Breaker,
            BumpState {
                post_hit_timer: remaining,
                ..Default::default()
            },
            bump_param_bundle(&config),
        ));

        app.insert_resource(TestInputActive(true));
        tick(&mut app);

        let captured = app.world().resource::<CapturedBumps>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].grade, BumpGrade::Late);
    }

    #[test]
    fn post_hit_timer_ticks_down() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    post_hit_timer: 0.1,
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestInputActive(false));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(bump.post_hit_timer < 0.1, "post_hit_timer should tick down");
    }

    #[test]
    fn cooldown_ticks_down() {
        let mut app = update_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    cooldown: 0.1,
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestInputActive(false));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(bump.cooldown < 0.1, "cooldown should tick down");
    }

    // ── grade_bump integration tests ─────────────────────────────────

    #[derive(Resource)]
    struct TestHitMessage(Option<BoltHitBreaker>);

    fn enqueue_hit(msg_res: Res<TestHitMessage>, mut writer: MessageWriter<BoltHitBreaker>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn grade_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BumpPerformed>();
        app.add_message::<BumpWhiffed>();
        app.init_resource::<CapturedBumps>();
        app.insert_resource(TestHitMessage(None));
        app.add_systems(
            Update,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
            ),
        );
        app
    }

    #[test]
    fn bolt_hit_with_active_forward_perfect() {
        let mut app = grade_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.perfect_window * 0.5, // in the perfect zone
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestHitMessage(Some(BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        })));
        app.update();

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active, "should deactivate");
        assert!(
            (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
            "perfect forward should set perfect_bump_cooldown ({}), got {}",
            config.perfect_bump_cooldown,
            bump.cooldown
        );
        assert!(bump.post_hit_timer <= 0.0, "should clear post_hit_timer");

        let captured = app.world().resource::<CapturedBumps>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].grade, BumpGrade::Perfect);
    }

    #[test]
    fn bolt_hit_with_active_forward_early() {
        let mut app = grade_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.early_window + config.perfect_window, // just started
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        app.insert_resource(TestHitMessage(Some(BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        })));
        app.update();

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(
            (bump.cooldown - config.weak_bump_cooldown).abs() < f32::EPSILON,
            "early forward should set weak_bump_cooldown ({}), got {}",
            config.weak_bump_cooldown,
            bump.cooldown
        );

        let captured = app.world().resource::<CapturedBumps>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].grade, BumpGrade::Early);
    }

    #[test]
    fn bolt_hit_without_active_sets_post_hit_timer_no_message() {
        let mut app = grade_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((Breaker, BumpState::default(), bump_param_bundle(&config)))
            .id();

        app.insert_resource(TestHitMessage(Some(BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        })));
        app.update();

        let bump = app.world().get::<BumpState>(entity).unwrap();
        let expected = config.perfect_window + config.late_window;
        assert!(
            (bump.post_hit_timer - expected).abs() < f32::EPSILON,
            "post_hit_timer should be set to perfect + late window, got {}",
            bump.post_hit_timer
        );

        let captured = app.world().resource::<CapturedBumps>();
        assert!(captured.0.is_empty(), "no message when bump not active");
    }

    #[test]
    fn no_hit_no_change() {
        let mut app = grade_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((Breaker, BumpState::default(), bump_param_bundle(&config)))
            .id();

        // No hit message
        app.update();

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active);
        assert!(bump.post_hit_timer <= 0.0);

        let captured = app.world().resource::<CapturedBumps>();
        assert!(captured.0.is_empty());
    }

    // ── combined update_bump + grade_bump integration tests ─────────

    /// App that runs both `update_bump` and `grade_bump` with production ordering,
    /// plus a hit injector and message captures.
    fn combined_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<InputActions>();
        app.add_message::<BoltHitBreaker>();
        app.add_message::<BumpPerformed>();
        app.add_message::<BumpWhiffed>();
        app.init_resource::<CapturedBumps>();
        app.init_resource::<CapturedWhiffs>();
        app.insert_resource(TestInputActive(false));
        app.insert_resource(TestHitMessage(None));
        app.add_systems(
            Update,
            (
                set_bump_action.before(update_bump),
                enqueue_hit.before(grade_bump),
                update_bump,
                grade_bump.after(update_bump),
                (capture_bumps, capture_whiffs).after(grade_bump),
            ),
        );
        app
    }

    #[test]
    fn same_frame_hit_and_expiry_grades_not_whiffs() {
        let mut app = combined_bump_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: 0.001, // about to expire this tick
                    ..Default::default()
                },
                bump_param_bundle(&config),
            ))
            .id();

        // Bolt hits the same frame the window would expire
        app.insert_resource(TestHitMessage(Some(BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

        let bump = app.world().get::<BumpState>(entity).unwrap();
        assert!(!bump.active, "should deactivate");

        // Should be graded as a forward bump (perfect — timer near 0 is within perfect_window)
        let captured = app.world().resource::<CapturedBumps>();
        assert_eq!(
            captured.0.len(),
            1,
            "should grade the hit, not whiff — got {} bumps",
            captured.0.len()
        );
        assert_eq!(captured.0[0].grade, BumpGrade::Perfect);

        // Should NOT whiff
        let whiffs = app.world().resource::<CapturedWhiffs>();
        assert_eq!(whiffs.0, 0, "should not whiff when hit arrives same frame");

        // Cooldown should match grade, not whiff
        assert!(
            (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
            "cooldown should be perfect_bump_cooldown ({}), got {}",
            config.perfect_bump_cooldown,
            bump.cooldown
        );
    }

    // ── perfect_bump_dash_cancel tests ───────────────────────────────

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
                SettleDuration(config.settle_duration),
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
}
