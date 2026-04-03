//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    cells::{
        messages::{CellDestroyedAt, CellImpactWall, DamageCell, RequestCellDestroyed},
        resources::CellConfig,
        systems::{
            cell_wall_collision, check_lock_release::check_lock_release, cleanup_cell,
            handle_cell_hit, rotate_shield_cells::rotate_shield_cells,
            sync_orbit_cell_positions::sync_orbit_cell_positions, tick_cell_regen::tick_cell_regen,
        },
    },
    effect::EffectSystems,
    shared::{GameState, PlayingState},
    state::run::node::{sets::NodeSystems, systems::dispatch_cell_effects},
};

/// Plugin for the cells domain.
///
/// Owns cell components, damage handling, and destruction logic.
pub(crate) struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .add_message::<DamageCell>()
            .add_message::<CellImpactWall>()
            .init_resource::<CellConfig>()
            .add_systems(
                OnEnter(GameState::Playing),
                dispatch_cell_effects.after(NodeSystems::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (
                    handle_cell_hit,
                    check_lock_release.after(handle_cell_hit),
                    tick_cell_regen,
                    rotate_shield_cells,
                    sync_orbit_cell_positions.after(rotate_shield_cells),
                    cleanup_cell.after(EffectSystems::Bridge),
                    cell_wall_collision,
                )
                    .run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::resources::CollisionQuadtree;

    use super::*;
    use crate::shared::GameState;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // CellsPlugin reads BoltImpactCell messages from bolt domain
            .add_message::<crate::bolt::messages::BoltImpactCell>()
            .insert_resource(CollisionQuadtree::default())
            .add_plugins(CellsPlugin)
            .update();
    }

    // ── Orbit system registration tests ───────────────────────────
    //
    // These verify that CellsPlugin registers rotate_shield_cells and
    // sync_orbit_cell_positions. They will FAIL until the plugin wires
    // the orbit systems.

    use std::time::Duration;

    use rantzsoft_spatial2d::{
        components::{Position2D, Spatial2D},
        propagation::PositionPropagation,
    };

    use crate::cells::components::{Cell, OrbitAngle, OrbitCell, OrbitConfig, ShieldParent};

    fn cells_plugin_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_message::<crate::bolt::messages::BoltImpactCell>()
            .insert_resource(CollisionQuadtree::default())
            .add_plugins(CellsPlugin);
        // Enter Playing -> Active so run_if(in_state(PlayingState::Active)) gates pass
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app
    }

    fn tick_cells(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    /// Behavior 11: `CellsPlugin` registers `rotate_shield_cells`.
    ///
    /// Given: shield parent with orbit child at `OrbitAngle(0.0)`,
    ///        `OrbitConfig { radius: 60.0, speed: PI/2 }`
    /// When: `CellsPlugin` tick at dt=1.0s
    /// Then: `OrbitAngle` ~ PI/2
    #[test]
    fn cells_plugin_registers_rotate_shield_cells() {
        use std::f32::consts::FRAC_PI_2;

        let mut app = cells_plugin_app();

        let shield = app
            .world_mut()
            .spawn((
                Cell,
                ShieldParent,
                Spatial2D,
                Position2D(Vec2::new(100.0, 200.0)),
            ))
            .id();

        let orbit = app
            .world_mut()
            .spawn((
                Cell,
                OrbitCell,
                Spatial2D,
                PositionPropagation::Absolute,
                Position2D(Vec2::ZERO),
                OrbitAngle(0.0),
                OrbitConfig {
                    radius: 60.0,
                    speed: FRAC_PI_2,
                },
                ChildOf(shield),
            ))
            .id();

        tick_cells(&mut app, Duration::from_secs(1));

        let angle = app.world().get::<OrbitAngle>(orbit).unwrap();
        assert!(
            (angle.0 - FRAC_PI_2).abs() < 1e-4,
            "orbit angle should be ~PI/2 ({FRAC_PI_2}) after 1s at speed PI/2 via CellsPlugin, got {}",
            angle.0
        );
    }

    /// Behavior 12: `CellsPlugin` registers `sync_orbit_cell_positions`.
    ///
    /// Given: shield at (100.0, 200.0), orbit at angle 0.0, radius 60.0
    /// When: `CellsPlugin` systems run
    /// Then: orbit `Position2D` = (160.0, 200.0)
    #[test]
    fn cells_plugin_registers_sync_orbit_cell_positions() {
        let mut app = cells_plugin_app();

        let shield = app
            .world_mut()
            .spawn((
                Cell,
                ShieldParent,
                Spatial2D,
                Position2D(Vec2::new(100.0, 200.0)),
            ))
            .id();

        let orbit = app
            .world_mut()
            .spawn((
                Cell,
                OrbitCell,
                Spatial2D,
                PositionPropagation::Absolute,
                Position2D(Vec2::ZERO),
                OrbitAngle(0.0),
                OrbitConfig {
                    radius: 60.0,
                    speed: std::f32::consts::FRAC_PI_2,
                },
                ChildOf(shield),
            ))
            .id();

        tick_cells(&mut app, Duration::from_millis(16));

        let pos = app.world().get::<Position2D>(orbit).unwrap();
        assert!(
            (pos.0.x - 160.0).abs() < 1.0,
            "orbit x should be ~160.0 (shield 100.0 + radius 60.0 * cos(0)), got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 200.0).abs() < 3.0,
            "orbit y should be ~200.0 (shield 200.0 + radius 60.0 * sin(0)), got {}",
            pos.0.y
        );
    }
}
