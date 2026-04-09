use bevy::prelude::*;

use super::{
    super::*,
    helpers::{Counter, increment},
};
use crate::{
    bolt::messages::BoltImpactCell,
    cells::{messages::DamageCell, resources::CellTypeRegistry},
    shared::{playfield::PlayfieldConfig, rng::GameRng},
};

// ════════════════════════════════════════════════════════════════════
// Section O: tick()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 32: tick() advances exactly one FixedUpdate timestep ──

#[test]
fn tick_advances_one_fixed_update() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "tick() should advance exactly one FixedUpdate (counter = 1)"
    );
}

#[test]
fn tick_five_times_increments_five() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment)
        .build();
    for _ in 0..5 {
        tick(&mut app);
    }
    assert_eq!(
        app.world().resource::<Counter>().0,
        5,
        "5 ticks should increment counter to 5"
    );
}

// ── Behavior 33: tick() reads configured timestep (not hardcoded) ──

#[test]
fn tick_reads_app_timestep_not_hardcoded() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment)
        .build();
    // Change the timestep to a non-default value
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(std::time::Duration::from_millis(100));
    // tick() should still advance exactly one FixedUpdate (it reads from the resource)
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "tick() should trigger exactly one FixedUpdate even with a custom timestep"
    );
}

#[test]
fn raw_update_without_overstep_does_not_run_fixed_update() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment)
        .build();
    // Calling app.update() directly without accumulating overstep
    app.update();
    assert_eq!(
        app.world().resource::<Counter>().0,
        0,
        "app.update() without overstep accumulation should NOT run FixedUpdate"
    );
}

// ── Behavior 34b: tick() on systemless app does not panic ──

#[test]
fn tick_on_empty_app_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    tick(&mut app);
    // No panic = pass
}

#[test]
fn tick_multiple_times_on_empty_app_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    for _ in 0..3 {
        tick(&mut app);
    }
    // No panic = pass
}

// ════════════════════════════════════════════════════════════════════
// Section P: Builder Method Chaining
// ════════════════════════════════════════════════════════════════════

// ── Behavior 35: all builder methods chain fluently ──

#[test]
fn maximal_builder_chain_compiles_and_builds() {
    use crate::{bolt::registry::BoltRegistry, breaker::registry::BreakerRegistry};

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_resource::<GameRng>()
        .insert_resource(PlayfieldConfig {
            width: 1920.0,
            ..Default::default()
        })
        .with_message::<BoltImpactCell>()
        .with_message_capture::<DamageCell>()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_system(FixedUpdate, increment)
        .build();

    // Verify critical resources
    assert!(
        app.world()
            .get_resource::<State<crate::state::types::NodeState>>()
            .is_some(),
        "State<NodeState> should be present in maximal chain"
    );
    assert!(
        app.world().get_resource::<PlayfieldConfig>().is_some(),
        "PlayfieldConfig should be present"
    );
    assert!(
        (app.world().resource::<PlayfieldConfig>().width - 1920.0).abs() < f32::EPSILON,
        "insert_resource should have overridden with_playfield's default to 1920.0"
    );
    assert!(
        app.world().get_resource::<GameRng>().is_some(),
        "GameRng should be present"
    );
    assert!(
        app.world().get_resource::<BoltRegistry>().is_some(),
        "BoltRegistry should be present"
    );
    assert!(
        app.world().get_resource::<BreakerRegistry>().is_some(),
        "BreakerRegistry should be present"
    );
    assert!(
        app.world().get_resource::<CellTypeRegistry>().is_some(),
        "CellTypeRegistry should be present"
    );
    assert!(
        app.world()
            .get_resource::<MessageCollector<DamageCell>>()
            .is_some(),
        "MessageCollector<DamageCell> should be present"
    );

    // Verify the app actually works
    app.insert_resource(Counter::default());
    tick(&mut app);
}
