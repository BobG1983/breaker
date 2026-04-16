use super::helpers::{sequence_plugin_advance_to_playing, sequence_plugin_app_loading};
use crate::{
    cells::behaviors::phantom::components::{
        PhantomCell as PhantomCellMarker, PhantomConfig, PhantomPhase, PhantomTimer,
    },
    prelude::*,
};

/// Behavior 43: `CellsPlugin` registers `tick_phantom_phase` in
/// `FixedUpdate` with `run_if(NodeState::Playing)`.
#[test]
fn cells_plugin_registers_tick_phantom_phase_in_fixed_update() {
    let mut app = sequence_plugin_app_loading();

    // Spawn a phantom cell with timer about to expire
    let entity = app
        .world_mut()
        .spawn((
            Cell,
            PhantomCellMarker,
            PhantomPhase::Solid,
            PhantomTimer(0.01),
            PhantomConfig {
                cycle_secs:     3.0,
                telegraph_secs: 0.5,
            },
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Hp::new(20.0),
            KilledBy::default(),
        ))
        .id();

    sequence_plugin_advance_to_playing(&mut app);
    tick(&mut app);

    let phase = app
        .world()
        .get::<PhantomPhase>(entity)
        .expect("entity should have PhantomPhase");
    assert_eq!(
        *phase,
        PhantomPhase::Telegraph,
        "CellsPlugin should register tick_phantom_phase; Solid phase with expired timer should transition to Telegraph"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.5).abs() < f32::EPSILON,
        "timer should reset to Telegraph duration 0.5, got {}",
        timer.0
    );
}

/// Behavior 43 edge (control): same setup but app stays in
/// `NodeState::Loading` — timer does NOT decrement, phase does NOT change.
#[test]
fn cells_plugin_phantom_does_not_tick_in_loading_state() {
    let mut app = sequence_plugin_app_loading();

    let entity = app
        .world_mut()
        .spawn((
            Cell,
            PhantomCellMarker,
            PhantomPhase::Solid,
            PhantomTimer(0.01),
            PhantomConfig {
                cycle_secs:     3.0,
                telegraph_secs: 0.5,
            },
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Hp::new(20.0),
            KilledBy::default(),
        ))
        .id();

    // Do NOT advance to playing — tick in Loading state
    tick(&mut app);

    let phase = app.world().get::<PhantomPhase>(entity).unwrap();
    assert_eq!(
        *phase,
        PhantomPhase::Solid,
        "phantom should NOT tick in Loading state — phase should remain Solid"
    );

    let timer = app.world().get::<PhantomTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.01).abs() < f32::EPSILON,
        "timer should NOT decrement in Loading state, should remain 0.01, got {}",
        timer.0
    );
}
