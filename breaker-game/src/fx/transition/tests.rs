use std::time::Duration;

use bevy::{prelude::*, state::app::StatesPlugin, time::TimeUpdateStrategy};

use super::*;
use crate::shared::{GameRng, GameState};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<GameState>()
        .insert_resource(TransitionConfig::default())
        .insert_resource(GameRng::from_seed(42));
    app
}

#[test]
fn spawn_transition_out_creates_overlay_entity() {
    let mut app = test_app();
    app.add_systems(Update, spawn_transition_out);
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "expected exactly 1 TransitionOverlay entity");
}

#[test]
fn spawn_transition_out_sets_timer_direction_out() {
    let mut app = test_app();
    app.add_systems(Update, spawn_transition_out);
    app.update();

    let timer = app
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app.world())
        .next()
        .expect("expected a TransitionTimer entity");
    assert_eq!(
        timer.direction,
        TransitionDirection::Out,
        "transition-out should set direction to Out"
    );
    assert!(
        (timer.duration - 0.5).abs() < f32::EPSILON,
        "expected duration 0.5 from default config, got {}",
        timer.duration
    );
}

#[test]
fn spawn_transition_in_creates_overlay_entity() {
    let mut app = test_app();
    app.add_systems(Update, spawn_transition_in);
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "expected exactly 1 TransitionOverlay entity");
}

#[test]
fn spawn_transition_in_sets_timer_direction_in() {
    let mut app = test_app();
    app.add_systems(Update, spawn_transition_in);
    app.update();

    let timer = app
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app.world())
        .next()
        .expect("expected a TransitionTimer entity");
    assert_eq!(
        timer.direction,
        TransitionDirection::In,
        "transition-in should set direction to In"
    );
    assert!(
        (timer.duration - 0.3).abs() < f32::EPSILON,
        "expected duration 0.3 from default config, got {}",
        timer.duration
    );
}

#[test]
fn animate_transition_ticks_timer_down() {
    let mut app = test_app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        0.1,
    )));
    app.add_systems(Update, animate_transition);

    app.world_mut().spawn((
        TransitionTimer {
            remaining: 0.5,
            duration: 0.5,
            style: TransitionStyle::Flash,
            direction: TransitionDirection::Out,
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
        Node::default(),
    ));

    // First update initializes time, second advances it by 0.1s
    app.update();
    app.update();

    let timer = app
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app.world())
        .next()
        .expect("timer entity should still exist");
    assert!(
        (timer.remaining - 0.4).abs() < 0.02,
        "expected remaining ~0.4 after 0.1s tick, got {}",
        timer.remaining
    );
}

#[test]
fn animate_transition_out_completion_transitions_to_chip_select() {
    let mut app = test_app();
    app.add_systems(Update, animate_transition);

    app.world_mut().spawn((
        TransitionTimer {
            remaining: 0.0,
            duration: 0.5,
            style: TransitionStyle::Flash,
            direction: TransitionDirection::Out,
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
        Node::default(),
    ));

    app.update();

    let next = app.world().resource::<NextState<GameState>>();
    assert!(
        format!("{next:?}").contains("ChipSelect"),
        "expected ChipSelect after out-transition completes, got: {next:?}"
    );
}

#[test]
fn animate_transition_in_completion_transitions_to_playing() {
    let mut app = test_app();
    app.add_systems(Update, animate_transition);

    app.world_mut().spawn((
        TransitionTimer {
            remaining: 0.0,
            duration: 0.3,
            style: TransitionStyle::Flash,
            direction: TransitionDirection::In,
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
        Node::default(),
    ));

    app.update();

    let next = app.world().resource::<NextState<GameState>>();
    assert!(
        format!("{next:?}").contains("Playing"),
        "expected Playing after in-transition completes, got: {next:?}"
    );
}

#[test]
fn cleanup_transition_despawns_overlay_entities() {
    let mut app = test_app();
    app.add_systems(Update, cleanup_transition);

    // Spawn one overlay entity and one non-overlay entity
    app.world_mut().spawn(TransitionOverlay);
    let other = app.world_mut().spawn(Name::new("not-an-overlay")).id();

    app.update();

    let overlay_count = app
        .world_mut()
        .query_filtered::<Entity, With<TransitionOverlay>>()
        .iter(app.world())
        .count();
    assert_eq!(
        overlay_count, 0,
        "all TransitionOverlay entities should be despawned"
    );
    assert!(
        app.world().get_entity(other).is_ok(),
        "non-overlay entity should still exist"
    );
}

#[test]
fn same_seed_produces_same_style() {
    // First run
    let mut app1 = test_app();
    app1.add_systems(Update, spawn_transition_out);
    app1.update();

    let style1 = app1
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app1.world())
        .next()
        .expect("expected a TransitionTimer entity from first run")
        .style;

    // Second run with same seed
    let mut app2 = test_app();
    app2.add_systems(Update, spawn_transition_out);
    app2.update();

    let style2 = app2
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app2.world())
        .next()
        .expect("expected a TransitionTimer entity from second run")
        .style;

    assert_eq!(
        style1, style2,
        "same seed should produce the same TransitionStyle"
    );
}

#[test]
fn transition_duration_configurable() {
    let mut app = test_app();
    app.insert_resource(TransitionConfig {
        out_duration: 1.5,
        in_duration: 0.3,
        flash_color_rgb: [1.0, 1.0, 1.0],
        sweep_color_rgb: [0.0, 0.8, 1.0],
    });
    app.add_systems(Update, spawn_transition_out);
    app.update();

    let timer = app
        .world_mut()
        .query::<&TransitionTimer>()
        .iter(app.world())
        .next()
        .expect("expected a TransitionTimer entity");
    assert!(
        (timer.duration - 1.5).abs() < f32::EPSILON,
        "expected duration 1.5 from custom config, got {}",
        timer.duration
    );
}

// -- A12: overlay_color returns correct alpha for Flash/Sweep x In/Out ─────

#[test]
fn overlay_color_returns_correct_alpha_for_all_style_direction_combinations() {
    let config = TransitionConfig::default();

    // Flash + Out: starts transparent (alpha 0.0)
    let flash_out = overlay_color(&config, TransitionStyle::Flash, TransitionDirection::Out);
    assert_eq!(
        flash_out,
        Color::srgba(1.0, 1.0, 1.0, 0.0),
        "Flash+Out should start transparent"
    );

    // Flash + In: starts opaque (alpha 1.0)
    let flash_in = overlay_color(&config, TransitionStyle::Flash, TransitionDirection::In);
    assert_eq!(
        flash_in,
        Color::srgba(1.0, 1.0, 1.0, 1.0),
        "Flash+In should start opaque"
    );

    // Sweep + Out: always starts opaque
    let sweep_out = overlay_color(&config, TransitionStyle::Sweep, TransitionDirection::Out);
    assert_eq!(
        sweep_out,
        Color::srgba(0.0, 0.8, 1.0, 1.0),
        "Sweep+Out should start opaque"
    );

    // Sweep + In: always starts opaque
    let sweep_in = overlay_color(&config, TransitionStyle::Sweep, TransitionDirection::In);
    assert_eq!(
        sweep_in,
        Color::srgba(0.0, 0.8, 1.0, 1.0),
        "Sweep+In should start opaque"
    );
}

#[test]
fn overlay_color_threads_custom_rgb_values_correctly() {
    let config = TransitionConfig {
        out_duration: 0.5,
        in_duration: 0.3,
        flash_color_rgb: [0.5, 0.3, 0.1],
        sweep_color_rgb: [0.9, 0.1, 0.0],
    };

    let flash_out = overlay_color(&config, TransitionStyle::Flash, TransitionDirection::Out);
    assert_eq!(
        flash_out,
        Color::srgba(0.5, 0.3, 0.1, 0.0),
        "Custom Flash+Out should use custom RGB with alpha 0.0"
    );

    let flash_in = overlay_color(&config, TransitionStyle::Flash, TransitionDirection::In);
    assert_eq!(
        flash_in,
        Color::srgba(0.5, 0.3, 0.1, 1.0),
        "Custom Flash+In should use custom RGB with alpha 1.0"
    );

    let sweep_out = overlay_color(&config, TransitionStyle::Sweep, TransitionDirection::Out);
    assert_eq!(
        sweep_out,
        Color::srgba(0.9, 0.1, 0.0, 1.0),
        "Custom Sweep+Out should use custom RGB with alpha 1.0"
    );

    let sweep_in = overlay_color(&config, TransitionStyle::Sweep, TransitionDirection::In);
    assert_eq!(
        sweep_in,
        Color::srgba(0.9, 0.1, 0.0, 1.0),
        "Custom Sweep+In should use custom RGB with alpha 1.0"
    );
}
