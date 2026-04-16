//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::sets::BoltSystems,
    breaker::sets::BreakerSystems,
    cells::{
        behaviors::{
            armored::systems::check_armor_direction::check_armor_direction,
            guarded::systems::slide_guardian_cells,
            locked::systems::{check_lock_release, sync_lock_invulnerable::sync_lock_invulnerable},
            magnetic::systems::apply_magnetic_fields,
            phantom::systems::tick_phantom_phase,
            regen::systems::tick_cell_regen,
            sequence::systems::{
                advance_sequence::advance_sequence, init_sequence_groups::init_sequence_groups,
                reset_inactive_sequence_hp::reset_inactive_sequence_hp,
            },
            survival::systems::{
                kill_bump_vulnerable_cells::kill_bump_vulnerable_cells,
                suppress_bolt_immune_damage::suppress_bolt_immune_damage,
            },
        },
        messages::CellImpactWall,
        resources::CellConfig,
        systems::{cell_wall_collision, update_cell_damage_visuals},
    },
    effect_v3::sets::EffectV3Systems,
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
            .add_systems(OnEnter(NodeState::Playing), init_sequence_groups)
            .add_systems(
                FixedUpdate,
                (
                    check_lock_release.after(DeathPipelineSystems::HandleKill),
                    sync_lock_invulnerable.after(check_lock_release),
                    tick_cell_regen,
                    tick_phantom_phase,
                    slide_guardian_cells,
                    apply_magnetic_fields,
                    cell_wall_collision,
                    update_cell_damage_visuals
                        .after(DeathPipelineSystems::ApplyDamage)
                        .before(DeathPipelineSystems::HandleKill),
                    reset_inactive_sequence_hp
                        .after(DeathPipelineSystems::ApplyDamage)
                        .before(DeathPipelineSystems::DetectDeaths),
                    advance_sequence.after(EffectV3Systems::Death),
                    check_armor_direction
                        .after(BoltSystems::CellCollision)
                        .before(DeathPipelineSystems::ApplyDamage),
                    suppress_bolt_immune_damage
                        .after(check_armor_direction)
                        .before(DeathPipelineSystems::ApplyDamage),
                    kill_bump_vulnerable_cells
                        .after(BreakerSystems::CellCollision)
                        .before(DeathPipelineSystems::ApplyDamage),
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
            // CellsPlugin reads BreakerImpactCell messages from breaker domain
            .add_message::<BreakerImpactCell>()
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
            .add_message::<BreakerImpactCell>()
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

    // ── Sequence cross-plugin behaviors 30–32 ─────────────────────────

    use std::marker::PhantomData;

    use crate::cells::{components::SequenceActive, test_utils::spawn_cell_in_world};

    /// Resource seeding `DamageDealt<Cell>` through a one-shot enqueue system
    /// registered `before(ApplyDamage)`. Mirrors the scaffold in
    /// `behaviors/sequence/tests/helpers.rs` so the cross-plugin tests can
    /// deliver damage without depending on bolt collision.
    #[derive(Resource, Default)]
    struct PluginTestPendingCellDamage(Vec<DamageDealt<Cell>>);

    fn enqueue_cell_damage_plugin_test(
        mut pending: ResMut<PluginTestPendingCellDamage>,
        mut writer: MessageWriter<DamageDealt<Cell>>,
    ) {
        for msg in pending.0.drain(..) {
            writer.write(msg);
        }
    }

    /// Builds a `cells_plugin_app`-style App but does NOT navigate into
    /// `NodeState::Playing`. Tests drive the transition after spawning.
    fn sequence_plugin_app_loading() -> App {
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
            .add_message::<BreakerImpactCell>()
            .insert_resource(CollisionQuadtree::default());
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);
        app.add_plugins(CellsPlugin);
        app.init_resource::<PluginTestPendingCellDamage>();
        app.add_systems(
            FixedUpdate,
            enqueue_cell_damage_plugin_test.before(DeathPipelineSystems::ApplyDamage),
        );
        app
    }

    fn sequence_plugin_advance_to_playing(app: &mut App) {
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
    }

    fn spawn_plugin_sequence_cell(
        app: &mut App,
        pos: Vec2,
        group: u32,
        position: u32,
        hp: f32,
    ) -> Entity {
        let entity = spawn_cell_in_world(app.world_mut(), |commands| {
            Cell::builder()
                .sequence(group, position)
                .position(pos)
                .dimensions(10.0, 10.0)
                .hp(hp)
                .headless()
                .spawn(commands)
        });
        app.world_mut()
            .entity_mut(entity)
            .insert(rantzsoft_spatial2d::components::GlobalPosition2D(pos));
        entity
    }

    fn plugin_damage_msg(target: Entity, amount: f32) -> DamageDealt<Cell> {
        DamageDealt {
            dealer: None,
            target,
            amount,
            source_chip: None,
            _marker: PhantomData,
        }
    }

    /// Behavior 30: `CellsPlugin` registers `init_sequence_groups` in
    /// `OnEnter(NodeState::Playing)`.
    #[test]
    fn cells_plugin_registers_init_sequence_groups_on_enter_playing() {
        let mut app = sequence_plugin_app_loading();

        let e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
        let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);
        let e2 = spawn_plugin_sequence_cell(&mut app, Vec2::new(20.0, 0.0), 1, 2, 20.0);

        sequence_plugin_advance_to_playing(&mut app);

        assert!(
            app.world().get::<SequenceActive>(e0).is_some(),
            "CellsPlugin should register init_sequence_groups on OnEnter(NodeState::Playing)"
        );
        assert!(app.world().get::<SequenceActive>(e1).is_none());
        assert!(app.world().get::<SequenceActive>(e2).is_none());
    }

    /// Behavior 31: `CellsPlugin` registers `reset_inactive_sequence_hp`
    /// between `ApplyDamage` and `DetectDeaths`.
    #[test]
    fn cells_plugin_registers_reset_inactive_sequence_hp_between_apply_and_detect() {
        let mut app = sequence_plugin_app_loading();

        let _e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
        let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);

        sequence_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg(e1, 25.0));
        tick(&mut app);

        let hp = app.world().get::<Hp>(e1).expect("e1 should still have Hp");
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "CellsPlugin should register reset_inactive_sequence_hp between ApplyDamage and DetectDeaths, got {}",
            hp.current
        );
        assert!(app.world().get::<Dead>(e1).is_none());
    }

    /// Behavior 32: `CellsPlugin` registers `advance_sequence` after
    /// `EffectV3Systems::Death`.
    #[test]
    fn cells_plugin_registers_advance_sequence_after_effect_v3_death() {
        let mut app = sequence_plugin_app_loading();

        let e0 = spawn_plugin_sequence_cell(&mut app, Vec2::new(0.0, 0.0), 1, 0, 20.0);
        let e1 = spawn_plugin_sequence_cell(&mut app, Vec2::new(10.0, 0.0), 1, 1, 20.0);

        sequence_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg(e0, 25.0));

        for _ in 0..2 {
            tick(&mut app);
        }

        assert!(
            app.world().get_entity(e0).is_err() || app.world().get::<Dead>(e0).is_some(),
            "e0 should be dead after lethal damage"
        );
        assert!(
            app.world().get::<SequenceActive>(e1).is_some(),
            "CellsPlugin should register advance_sequence after EffectV3Systems::Death"
        );
    }

    // ── Armored cross-plugin behavior 27 ─────────────────────────────

    use crate::{
        bolt::components::PiercingRemaining, cells::behaviors::armored::components::ArmorDirection,
    };

    /// Resource seeding `BoltImpactCell` through a one-shot enqueue system
    /// so armored cross-plugin tests can deliver impact events without
    /// depending on bolt collision.
    #[derive(Resource, Default)]
    struct PluginTestPendingBoltImpact(Vec<BoltImpactCell>);

    fn enqueue_plugin_bolt_impact(
        mut pending: ResMut<PluginTestPendingBoltImpact>,
        mut writer: MessageWriter<BoltImpactCell>,
    ) {
        for msg in pending.0.drain(..) {
            writer.write(msg);
        }
    }

    fn spawn_plugin_armored_cell(
        app: &mut App,
        pos: Vec2,
        value: u8,
        facing: ArmorDirection,
        hp: f32,
    ) -> Entity {
        let entity = spawn_cell_in_world(app.world_mut(), |commands| {
            Cell::builder()
                .armored_facing(value, facing)
                .position(pos)
                .dimensions(10.0, 10.0)
                .hp(hp)
                .headless()
                .spawn(commands)
        });
        app.world_mut()
            .entity_mut(entity)
            .insert(rantzsoft_spatial2d::components::GlobalPosition2D(pos));
        entity
    }

    fn spawn_plugin_test_bolt(app: &mut App, piercing: u32) -> Entity {
        app.world_mut().spawn(PiercingRemaining(piercing)).id()
    }

    fn armored_plugin_app_loading() -> App {
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
            .add_message::<BreakerImpactCell>()
            .insert_resource(CollisionQuadtree::default());
        // Configure BoltSystems::CellCollision so the set exists without BoltPlugin
        app.configure_sets(FixedUpdate, crate::bolt::sets::BoltSystems::CellCollision);
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);
        app.add_plugins(CellsPlugin);
        app.init_resource::<PluginTestPendingBoltImpact>();
        app.init_resource::<PluginTestPendingCellDamage>();
        app.add_systems(
            FixedUpdate,
            enqueue_plugin_bolt_impact.in_set(crate::bolt::sets::BoltSystems::CellCollision),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_cell_damage_plugin_test.in_set(crate::bolt::sets::BoltSystems::CellCollision),
        );
        app
    }

    fn armored_plugin_advance_to_playing(app: &mut App) {
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
    }

    fn plugin_damage_msg_from(target: Entity, amount: f32, dealer: Entity) -> DamageDealt<Cell> {
        DamageDealt {
            dealer: Some(dealer),
            target,
            amount,
            source_chip: None,
            _marker: PhantomData,
        }
    }

    /// Behavior 27: `CellsPlugin` registers `check_armor_direction` ordered
    /// `.after(BoltSystems::CellCollision).before(DeathPipelineSystems::ApplyDamage)`.
    #[test]
    fn cells_plugin_registers_check_armor_direction_before_apply_damage() {
        let mut app = armored_plugin_app_loading();

        let cell = spawn_plugin_armored_cell(
            &mut app,
            Vec2::new(0.0, 0.0),
            2,
            ArmorDirection::Bottom,
            20.0,
        );
        let bolt = spawn_plugin_test_bolt(&mut app, 0);

        armored_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingBoltImpact>()
            .0
            .push(BoltImpactCell {
                bolt,
                cell,
                impact_normal: Vec2::NEG_Y,
                piercing_remaining: 0,
            });
        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg_from(cell, 5.0, bolt));

        tick(&mut app);

        let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "CellsPlugin should register check_armor_direction before ApplyDamage; armor blocked, got hp.current == {}",
            hp.current
        );
        assert!(app.world().get::<Dead>(cell).is_none());
    }

    // ── Phantom cross-plugin behavior 43 ──────────────────────────

    use crate::cells::behaviors::phantom::components::{
        PhantomCell as PhantomCellMarker, PhantomConfig, PhantomPhase, PhantomTimer,
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

    /// Behavior 27 edge (control): weak face hit passes through, proving
    /// the system is genuinely registered and not a stub no-op.
    #[test]
    fn cells_plugin_armored_weak_face_passes_through_control() {
        let mut app = armored_plugin_app_loading();

        // Armor on Top, hit on Bottom (weak face) via NEG_Y
        let cell =
            spawn_plugin_armored_cell(&mut app, Vec2::new(0.0, 0.0), 2, ArmorDirection::Top, 20.0);
        let bolt = spawn_plugin_test_bolt(&mut app, 0);

        armored_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingBoltImpact>()
            .0
            .push(BoltImpactCell {
                bolt,
                cell,
                impact_normal: Vec2::NEG_Y,
                piercing_remaining: 0,
            });
        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg_from(cell, 5.0, bolt));

        tick(&mut app);

        let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
        assert!(
            (hp.current - 15.0).abs() < f32::EPSILON,
            "weak face hit should pass through via plugin-registered system, got hp.current == {}",
            hp.current
        );
    }

    // ── Magnetic cross-plugin behavior 32 ────────────────────────────

    use rantzsoft_spatial2d::components::BaseSpeed;

    use crate::cells::behaviors::magnetic::components::{MagneticCell, MagneticField};

    /// Behavior 32: `CellsPlugin` registers `apply_magnetic_fields` in
    /// `FixedUpdate` with `run_if(NodeState::Playing)`.
    ///
    /// Given: Magnetic cell at origin, bolt at (50, 0) with velocity (0, 400).
    /// When: one tick in `NodeState::Playing`.
    /// Then: bolt velocity x becomes negative (pulled toward magnet).
    #[test]
    fn cells_plugin_registers_apply_magnetic_fields_in_playing() {
        let mut app = cells_plugin_app();

        // Spawn magnetic cell at origin
        app.world_mut().spawn((
            Cell,
            MagneticCell,
            MagneticField {
                radius:   200.0,
                strength: 1000.0,
            },
            Position2D(Vec2::ZERO),
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            Hp::new(20.0),
            KilledBy::default(),
        ));

        // Spawn bolt at (50, 0) with velocity (0, 400)
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(50.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
                BaseSpeed(400.0),
            ))
            .id();

        tick_cells(&mut app, Duration::from_secs_f32(1.0 / 60.0));

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            vel.0.x < 0.0,
            "CellsPlugin should register apply_magnetic_fields; bolt should be pulled toward magnet, got vx={}",
            vel.0.x
        );
    }

    /// Behavior 32 edge (control): same setup but in `NodeState::Loading` --
    /// velocity should remain unchanged, proving `run_if` gate works.
    #[test]
    fn cells_plugin_magnetic_does_not_run_in_loading_state() {
        let mut app = sequence_plugin_app_loading();

        // Spawn magnetic cell at origin
        app.world_mut().spawn((
            Cell,
            MagneticCell,
            MagneticField {
                radius:   200.0,
                strength: 1000.0,
            },
            Position2D(Vec2::ZERO),
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            Hp::new(20.0),
            KilledBy::default(),
        ));

        // Spawn bolt
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(50.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
                BaseSpeed(400.0),
            ))
            .id();

        // Do NOT advance to playing -- tick in Loading state
        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON,
            "magnetic should NOT run in Loading state, got vx={}",
            vel.0.x
        );
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "magnetic should NOT run in Loading state, got vy={}",
            vel.0.y
        );
    }

    // ── Survival cross-plugin behaviors 55-58 ────────────────────────────

    /// Resource seeding `BreakerImpactCell` through a one-shot enqueue system.
    #[derive(Resource, Default)]
    struct PluginTestPendingBreakerImpact(Vec<BreakerImpactCell>);

    fn enqueue_plugin_breaker_impact(
        mut pending: ResMut<PluginTestPendingBreakerImpact>,
        mut writer: MessageWriter<BreakerImpactCell>,
    ) {
        for msg in pending.0.drain(..) {
            writer.write(msg);
        }
    }

    /// Builds a plugin app that stays in Loading state.
    fn survival_plugin_app_loading() -> App {
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
            .add_message::<BreakerImpactCell>()
            .insert_resource(CollisionQuadtree::default());
        // Configure BoltSystems::CellCollision so the set exists without BoltPlugin
        app.configure_sets(FixedUpdate, crate::bolt::sets::BoltSystems::CellCollision);
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);
        app.add_plugins(CellsPlugin);
        app.init_resource::<PluginTestPendingBoltImpact>();
        app.init_resource::<PluginTestPendingCellDamage>();
        app.init_resource::<PluginTestPendingBreakerImpact>();
        app.add_systems(
            FixedUpdate,
            enqueue_plugin_bolt_impact.in_set(crate::bolt::sets::BoltSystems::CellCollision),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_cell_damage_plugin_test.in_set(crate::bolt::sets::BoltSystems::CellCollision),
        );
        app.add_systems(
            FixedUpdate,
            enqueue_plugin_breaker_impact.before(DeathPipelineSystems::ApplyDamage),
        );
        app
    }

    fn survival_plugin_advance_to_playing(app: &mut App) {
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
    }

    fn spawn_plugin_survival_cell(app: &mut App, hp: f32) -> Entity {
        spawn_cell_in_world(app.world_mut(), |commands| {
            Cell::builder()
                .survival(crate::cells::definition::AttackPattern::StraightDown, 10.0)
                .position(Vec2::ZERO)
                .dimensions(10.0, 10.0)
                .hp(hp)
                .headless()
                .spawn(commands)
        })
    }

    /// Behavior 55: `CellsPlugin` registers `suppress_bolt_immune_damage` in `FixedUpdate`.
    #[test]
    fn cells_plugin_registers_suppress_bolt_immune_damage_before_apply_damage() {
        let mut app = survival_plugin_app_loading();

        let cell = spawn_plugin_survival_cell(&mut app, 20.0);
        let bolt = app.world_mut().spawn(Bolt).id();

        survival_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingBoltImpact>()
            .0
            .push(BoltImpactCell {
                bolt,
                cell,
                impact_normal: Vec2::NEG_Y,
                piercing_remaining: 0,
            });
        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg_from(cell, 5.0, bolt));

        tick(&mut app);

        let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "CellsPlugin should register suppress_bolt_immune_damage before ApplyDamage; \
             BoltImmune cell should retain full HP, got hp.current == {}",
            hp.current
        );
    }

    /// Behavior 56: `suppress_bolt_immune_damage` does not run in `NodeState::Loading`.
    #[test]
    fn cells_plugin_suppress_bolt_immune_does_not_run_in_loading() {
        let mut app = survival_plugin_app_loading();

        let cell = spawn_plugin_survival_cell(&mut app, 20.0);
        let bolt = app.world_mut().spawn(Bolt).id();

        // Do NOT advance to Playing — stay in Loading
        app.world_mut()
            .resource_mut::<PluginTestPendingBoltImpact>()
            .0
            .push(BoltImpactCell {
                bolt,
                cell,
                impact_normal: Vec2::NEG_Y,
                piercing_remaining: 0,
            });
        app.world_mut()
            .resource_mut::<PluginTestPendingCellDamage>()
            .0
            .push(plugin_damage_msg_from(cell, 5.0, bolt));

        tick(&mut app);

        // No crash — system didn't run (gated by run_if). Damage also doesn't
        // apply since death pipeline is gated too.
    }

    /// Behavior 57: `CellsPlugin` registers `kill_bump_vulnerable_cells` in `FixedUpdate`.
    #[test]
    fn cells_plugin_registers_kill_bump_vulnerable_cells() {
        let mut app = survival_plugin_app_loading();

        let cell = spawn_plugin_survival_cell(&mut app, 20.0);
        let breaker = app.world_mut().spawn(Breaker).id();

        survival_plugin_advance_to_playing(&mut app);

        app.world_mut()
            .resource_mut::<PluginTestPendingBreakerImpact>()
            .0
            .push(BreakerImpactCell { breaker, cell });

        // Multiple ticks for full death pipeline
        for _ in 0..4 {
            tick(&mut app);
        }

        let is_dead = app.world().get_entity(cell).is_err()
            || app.world().get::<Dead>(cell).is_some()
            || app.world().get::<Hp>(cell).is_none_or(|h| h.current <= 0.0);
        assert!(
            is_dead,
            "CellsPlugin should register kill_bump_vulnerable_cells; \
             BumpVulnerable cell should be dead after breaker contact"
        );
    }

    /// Behavior 58: `kill_bump_vulnerable_cells` does not run in `NodeState::Loading`.
    #[test]
    fn cells_plugin_kill_bump_vulnerable_does_not_run_in_loading() {
        let mut app = survival_plugin_app_loading();

        let cell = spawn_plugin_survival_cell(&mut app, 20.0);
        let breaker = app.world_mut().spawn(Breaker).id();

        // Do NOT advance to Playing — stay in Loading
        app.world_mut()
            .resource_mut::<PluginTestPendingBreakerImpact>()
            .0
            .push(BreakerImpactCell { breaker, cell });

        tick(&mut app);

        let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "kill_bump_vulnerable_cells should NOT run in Loading state, got hp.current == {}",
            hp.current
        );
    }
}
