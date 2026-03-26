use super::helpers::*;

// =========================================================================
// update_force_bump_grade
// =========================================================================

/// `AlwaysPerfect` mode sets `ForceBumpGrade` to `Some(Perfect)`.
#[test]
fn update_force_bump_grade_always_perfect() {
    let mut app = perfect_tracking_app(0, BumpMode::AlwaysPerfect);
    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0,
        Some(BumpGrade::Perfect),
        "expected ForceBumpGrade(Some(Perfect)), got {:?}",
        grade.0
    );
}

/// `AlwaysEarly` mode sets `ForceBumpGrade` to `Some(Early)`.
#[test]
fn update_force_bump_grade_always_early() {
    let mut app = perfect_tracking_app(0, BumpMode::AlwaysEarly);
    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0,
        Some(BumpGrade::Early),
        "expected ForceBumpGrade(Some(Early)), got {:?}",
        grade.0
    );
}

/// `AlwaysLate` mode sets `ForceBumpGrade` to `Some(Late)`.
#[test]
fn update_force_bump_grade_always_late() {
    let mut app = perfect_tracking_app(0, BumpMode::AlwaysLate);
    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0,
        Some(BumpGrade::Late),
        "expected ForceBumpGrade(Some(Late)), got {:?}",
        grade.0
    );
}

/// `AlwaysWhiff` mode sets `ForceBumpGrade` to `None`.
#[test]
fn update_force_bump_grade_always_whiff() {
    let mut app = perfect_tracking_app(0, BumpMode::AlwaysWhiff);
    app.insert_resource(ForceBumpGrade(Some(BumpGrade::Perfect)));
    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0, None,
        "expected ForceBumpGrade(None) for AlwaysWhiff, got {:?}",
        grade.0
    );
}

/// `NeverBump` mode sets `ForceBumpGrade` to `None`.
#[test]
fn update_force_bump_grade_never_bump() {
    let mut app = perfect_tracking_app(0, BumpMode::NeverBump);
    app.insert_resource(ForceBumpGrade(Some(BumpGrade::Early)));
    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0, None,
        "expected ForceBumpGrade(None) for NeverBump, got {:?}",
        grade.0
    );
}

/// `Random` mode sets `ForceBumpGrade` to Some(valid grade) and is deterministic.
#[test]
fn update_force_bump_grade_random_produces_valid_and_deterministic() {
    // Run 1
    let mut app1 = perfect_tracking_app(42, BumpMode::Random);
    app1.add_systems(Update, update_force_bump_grade);

    let mut results1 = Vec::new();
    for _ in 0..10 {
        app1.update();
        let grade = app1.world().resource::<ForceBumpGrade>();
        results1.push(grade.0);
    }

    // Verify all results are Some with valid grades
    for (i, r) in results1.iter().enumerate() {
        assert!(
            matches!(
                r,
                Some(BumpGrade::Early | BumpGrade::Perfect | BumpGrade::Late)
            ),
            "run {i}: expected Some(Early|Perfect|Late), got {r:?}"
        );
    }

    // Run 2 (same seed) — must match
    let mut app2 = perfect_tracking_app(42, BumpMode::Random);
    app2.add_systems(Update, update_force_bump_grade);

    let mut results2 = Vec::new();
    for _ in 0..10 {
        app2.update();
        let grade = app2.world().resource::<ForceBumpGrade>();
        results2.push(grade.0);
    }

    assert_eq!(
        results1, results2,
        "expected deterministic results for same seed"
    );
}

/// `update_force_bump_grade` does not run when `InputDriver` is not `Perfect`.
#[test]
fn update_force_bump_grade_noop_when_driver_is_not_perfect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ForceBumpGrade(Some(BumpGrade::Perfect)));

    let driver = InputDriver::Chaos(crate::input::ChaosDriver::new(
        42,
        &ChaosParams { action_prob: 0.3 },
    ));
    app.insert_resource(ScenarioInputDriver(driver));

    app.add_systems(Update, update_force_bump_grade);

    app.update();

    let grade = app.world().resource::<ForceBumpGrade>();
    assert_eq!(
        grade.0,
        Some(BumpGrade::Perfect),
        "expected ForceBumpGrade unchanged when driver is Chaos, got {:?}",
        grade.0
    );
}
