//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::{
            guarded::systems::slide_guardian_cells,
            locked::systems::{check_lock_release, sync_lock_invulnerable::sync_lock_invulnerable},
            regen::systems::tick_cell_regen,
        },
        messages::CellImpactWall,
        resources::CellConfig,
        systems::{cell_wall_collision, update_cell_damage_visuals},
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
    state::run::node::{sets::NodeSystems, systems::dispatch_cell_effects},
};

/// Plugin for the cells domain.
///
/// Owns cell components, damage handling, and destruction logic.
pub(crate) struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellImpactWall>()
            .init_resource::<CellConfig>()
            .add_systems(
                OnEnter(NodeState::Loading),
                dispatch_cell_effects.after(NodeSystems::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (
                    check_lock_release.after(DeathPipelineSystems::HandleKill),
                    sync_lock_invulnerable.after(check_lock_release),
                    tick_cell_regen,
                    slide_guardian_cells,
                    cell_wall_collision,
                    update_cell_damage_visuals
                        .after(DeathPipelineSystems::ApplyDamage)
                        .before(DeathPipelineSystems::HandleKill),
                )
                    .run_if(in_state(NodeState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::resources::CollisionQuadtree;

    use super::*;
    use crate::{
        effect_v3::EffectV3Plugin,
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
            .insert_resource(CollisionQuadtree::default());
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);
        app.add_plugins(CellsPlugin);
        app.update();
    }

    // ── Guardian system registration test ─────────────────────────

    use std::time::Duration;

    use rantzsoft_spatial2d::components::Spatial2D;

    use crate::cells::components::{
        GuardedCell, GuardianCell, GuardianGridStep, GuardianSlideSpeed, GuardianSlot, SlideTarget,
    };

    fn cells_plugin_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_message::<BoltImpactCell>()
            .insert_resource(CollisionQuadtree::default());
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);
        app.add_plugins(CellsPlugin);
        // Navigate through state hierarchy to reach NodeState::Playing
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Playing);
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
}
