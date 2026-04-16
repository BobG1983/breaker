use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;
use rantzsoft_spatial2d::components::Spatial2D;

use super::{
    super::system::CellsPlugin,
    helpers::{cells_plugin_app, tick_cells},
};
use crate::{
    cells::components::{
        GuardedCell, GuardianCell, GuardianGridStep, GuardianSlideSpeed, GuardianSlot, SlideTarget,
    },
    effect_v3::EffectV3Plugin,
    prelude::*,
    shared::death_pipeline::{
        DeathPipelinePlugin, systems::tests::helpers::register_effect_v3_test_infrastructure,
    },
};

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<RunState>()
        .add_sub_state::<NodeState>()
        // CellsPlugin reads BoltImpactCell messages from bolt domain
        .add_message::<BoltImpactCell>()
        // CellsPlugin reads BreakerImpactCell messages from breaker domain
        .add_message::<BreakerImpactCell>()
        .insert_resource(CollisionQuadtree::default());
    app.add_plugins(DeathPipelinePlugin);
    register_effect_v3_test_infrastructure(&mut app);
    app.add_plugins(EffectV3Plugin);
    app.add_plugins(CellsPlugin);
    app.update();
}

/// Behavior 43: `CellsPlugin` registers `slide_guardian_cells` in `FixedUpdate`.
///
/// Given: guarded parent at origin, guardian at slot 3 with `SlideTarget(4)`,
///        speed 100.0, step (72.0, 26.0)
/// When: `CellsPlugin` tick at dt=0.5s
/// Then: guardian snaps to slot 4 position Vec2(72.0, -26.0) (distance 26.0 < 100*0.5=50)
#[test]
fn cells_plugin_registers_slide_guardian_cells() {
    let mut app = cells_plugin_app();

    let parent = app
        .world_mut()
        .spawn((
            Cell,
            GuardedCell,
            Spatial2D,
            Position2D(Vec2::new(0.0, 0.0)),
        ))
        .id();

    let guardian = app
        .world_mut()
        .spawn((
            Cell,
            GuardianCell,
            Spatial2D,
            Position2D(Vec2::new(72.0, 0.0)), // slot 3 position
            GuardianSlot(3),
            SlideTarget(4),
            GuardianSlideSpeed(100.0),
            GuardianGridStep {
                step_x: 72.0,
                step_y: 26.0,
            },
            ChildOf(parent),
        ))
        .id();

    tick_cells(&mut app, Duration::from_millis(500));

    // Slot 4 target position = (72.0, -26.0), distance from start = 26.0
    // Speed 100 * dt 0.5 = 50.0 > 26.0, so should snap
    let pos = app.world().get::<Position2D>(guardian).unwrap();
    assert!(
        (pos.0.x - 72.0).abs() < 1.0 && (pos.0.y - (-26.0)).abs() < 1.0,
        "guardian should snap to slot 4 position (72.0, -26.0) via CellsPlugin, got {:?}",
        pos.0
    );
    let slot = app.world().get::<GuardianSlot>(guardian).unwrap();
    assert_eq!(
        slot.0, 4,
        "GuardianSlot should update to 4 via CellsPlugin, got {}",
        slot.0
    );
}
