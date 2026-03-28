//! Bridge systems for targeted `Impacted` triggers — one per collision type.
//!
//! Each system reads a collision message and fires `Impacted(X)` on entity A
//! and `Impacted(Y)` on entity B, evaluating only those specific entities.
use bevy::prelude::*;

use crate::{
    bolt::{
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
        sets::BoltSystems,
    },
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::playing_state::PlayingState,
};

/// `BoltImpactCell` -> `Impacted(Cell)` on bolt + `Impacted(Bolt)` on cell.
fn bridge_impacted_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// `BoltImpactWall` -> `Impacted(Wall)` on bolt + `Impacted(Bolt)` on wall.
fn bridge_impacted_bolt_wall(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// `BoltImpactBreaker` -> `Impacted(Breaker)` on bolt + `Impacted(Bolt)` on breaker.
fn bridge_impacted_bolt_breaker(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// `BreakerImpactCell` -> `Impacted(Cell)` on breaker + `Impacted(Breaker)` on cell.
fn bridge_impacted_breaker_cell(
    mut reader: MessageReader<BreakerImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// `BreakerImpactWall` -> `Impacted(Wall)` on breaker + `Impacted(Breaker)` on wall.
fn bridge_impacted_breaker_wall(
    mut reader: MessageReader<BreakerImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// `CellImpactWall` -> `Impacted(Wall)` on cell + `Impacted(Cell)` on wall.
fn bridge_impacted_cell_wall(
    mut reader: MessageReader<CellImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
            );
        }
    }
}

/// Register all targeted impacted bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridge_impacted_bolt_cell.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_wall.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_breaker.after(BoltSystems::BreakerCollision),
            bridge_impacted_breaker_cell,
            bridge_impacted_breaker_wall,
            bridge_impacted_cell_wall,
        )
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

    // -- BoltImpactCell helper --

    #[derive(Resource)]
    struct TestBoltImpactCellMsg(Option<BoltImpactCell>);

    fn enqueue_bolt_impact_cell(
        msg_res: Res<TestBoltImpactCellMsg>,
        mut writer: MessageWriter<BoltImpactCell>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_bolt_cell() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltImpactCell>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_bolt_impact_cell.before(bridge_impacted_bolt_cell),
                    bridge_impacted_bolt_cell,
                ),
            );
        app
    }

    // -- BoltImpactWall helper --

    #[derive(Resource)]
    struct TestBoltImpactWallMsg(Option<BoltImpactWall>);

    fn enqueue_bolt_impact_wall(
        msg_res: Res<TestBoltImpactWallMsg>,
        mut writer: MessageWriter<BoltImpactWall>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_bolt_wall() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltImpactWall>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_bolt_impact_wall.before(bridge_impacted_bolt_wall),
                    bridge_impacted_bolt_wall,
                ),
            );
        app
    }

    // -- BoltImpactBreaker helper --

    #[derive(Resource)]
    struct TestBoltImpactBreakerMsg(Option<BoltImpactBreaker>);

    fn enqueue_bolt_impact_breaker(
        msg_res: Res<TestBoltImpactBreakerMsg>,
        mut writer: MessageWriter<BoltImpactBreaker>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_bolt_breaker() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltImpactBreaker>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_bolt_impact_breaker.before(bridge_impacted_bolt_breaker),
                    bridge_impacted_bolt_breaker,
                ),
            );
        app
    }

    // -- BreakerImpactCell helper --

    #[derive(Resource)]
    struct TestBreakerImpactCellMsg(Option<BreakerImpactCell>);

    fn enqueue_breaker_impact_cell(
        msg_res: Res<TestBreakerImpactCellMsg>,
        mut writer: MessageWriter<BreakerImpactCell>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_breaker_cell() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BreakerImpactCell>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_breaker_impact_cell.before(bridge_impacted_breaker_cell),
                    bridge_impacted_breaker_cell,
                ),
            );
        app
    }

    // -- BreakerImpactWall helper --

    #[derive(Resource)]
    struct TestBreakerImpactWallMsg(Option<BreakerImpactWall>);

    fn enqueue_breaker_impact_wall(
        msg_res: Res<TestBreakerImpactWallMsg>,
        mut writer: MessageWriter<BreakerImpactWall>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_breaker_wall() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BreakerImpactWall>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_breaker_impact_wall.before(bridge_impacted_breaker_wall),
                    bridge_impacted_breaker_wall,
                ),
            );
        app
    }

    // -- CellImpactWall helper --

    #[derive(Resource)]
    struct TestCellImpactWallMsg(Option<CellImpactWall>);

    fn enqueue_cell_impact_wall(
        msg_res: Res<TestCellImpactWallMsg>,
        mut writer: MessageWriter<CellImpactWall>,
    ) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app_cell_wall() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellImpactWall>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_cell_impact_wall.before(bridge_impacted_cell_wall),
                    bridge_impacted_cell_wall,
                ),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn impacted_cell_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Cell),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    fn impacted_bolt_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    fn impacted_wall_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Wall),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    fn impacted_breaker_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Breaker),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    // =========================================================================
    // bridge_impacted_bolt_cell — targeted on both participants
    // =========================================================================

    #[test]
    fn bridge_impacted_bolt_cell_fires_impacted_cell_on_bolt_entity() {
        let mut app = test_app_bolt_cell();

        // Bolt entity has Impacted(Cell) chain — should fire
        let bolt = app
            .world_mut()
            .spawn((
                impacted_cell_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Cell entity — no matching chain, should not fire
        let cell = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

        tick(&mut app);

        let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
        assert_eq!(
            bolt_active.0.len(),
            1,
            "bridge_impacted_bolt_cell should fire Impacted(Cell) on the bolt entity"
        );
        assert!(
            (bolt_active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );

        let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
        assert_eq!(
            cell_active.0.len(),
            0,
            "Cell entity has no Impacted(Cell) chain and should not be affected"
        );
    }

    #[test]
    fn bridge_impacted_bolt_cell_fires_impacted_bolt_on_cell_entity() {
        let mut app = test_app_bolt_cell();

        // Bolt entity — no matching chain for Impacted(Bolt)
        let bolt = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Cell entity has Impacted(Bolt) chain — should fire
        let cell = app
            .world_mut()
            .spawn((
                impacted_bolt_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

        tick(&mut app);

        let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
        assert_eq!(
            cell_active.0.len(),
            1,
            "bridge_impacted_bolt_cell should fire Impacted(Bolt) on the cell entity"
        );

        let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
        assert_eq!(
            bolt_active.0.len(),
            0,
            "Bolt entity has no Impacted(Bolt) chain and should not be affected"
        );
    }

    #[test]
    fn bridge_impacted_bolt_cell_does_not_fire_on_uninvolved_entity() {
        let mut app = test_app_bolt_cell();

        let bolt = app
            .world_mut()
            .spawn((
                impacted_cell_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn((
                impacted_bolt_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        // Third entity — not involved in the collision, should NOT fire
        app.world_mut().spawn((
            impacted_cell_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

        tick(&mut app);

        // Count entities with non-empty ActiveSpeedBoosts
        let mut affected_count = 0;
        for active in app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .iter(app.world())
        {
            if !active.0.is_empty() {
                affected_count += 1;
            }
        }
        assert_eq!(
            affected_count, 2,
            "Only the bolt and cell from the message should be affected (targeted, not global)"
        );
    }

    // =========================================================================
    // bridge_impacted_bolt_wall
    // =========================================================================

    #[test]
    fn bridge_impacted_bolt_wall_fires_impacted_wall_on_bolt() {
        let mut app = test_app_bolt_wall();

        let bolt = app
            .world_mut()
            .spawn((
                impacted_wall_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let wall = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBoltImpactWallMsg(Some(BoltImpactWall { bolt, wall })));

        tick(&mut app);

        let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
        assert_eq!(
            bolt_active.0.len(),
            1,
            "bridge_impacted_bolt_wall should fire Impacted(Wall) on the bolt entity"
        );
    }

    // =========================================================================
    // bridge_impacted_bolt_breaker
    // =========================================================================

    #[test]
    fn bridge_impacted_bolt_breaker_fires_impacted_breaker_on_bolt() {
        let mut app = test_app_bolt_breaker();

        let bolt = app
            .world_mut()
            .spawn((
                impacted_breaker_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let breaker = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBoltImpactBreakerMsg(Some(BoltImpactBreaker {
            bolt,
            breaker,
        })));

        tick(&mut app);

        let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
        assert_eq!(
            bolt_active.0.len(),
            1,
            "bridge_impacted_bolt_breaker should fire Impacted(Breaker) on the bolt entity"
        );
    }

    // =========================================================================
    // bridge_impacted_breaker_cell
    // =========================================================================

    #[test]
    fn bridge_impacted_breaker_cell_fires_impacted_cell_on_breaker() {
        let mut app = test_app_breaker_cell();

        let breaker = app
            .world_mut()
            .spawn((
                impacted_cell_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBreakerImpactCellMsg(Some(BreakerImpactCell {
            breaker,
            cell,
        })));

        tick(&mut app);

        let breaker_active = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
        assert_eq!(
            breaker_active.0.len(),
            1,
            "bridge_impacted_breaker_cell should fire Impacted(Cell) on the breaker entity"
        );
    }

    // =========================================================================
    // bridge_impacted_breaker_wall
    // =========================================================================

    #[test]
    fn bridge_impacted_breaker_wall_fires_impacted_wall_on_breaker() {
        let mut app = test_app_breaker_wall();

        let breaker = app
            .world_mut()
            .spawn((
                impacted_wall_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let wall = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestBreakerImpactWallMsg(Some(BreakerImpactWall {
            breaker,
            wall,
        })));

        tick(&mut app);

        let breaker_active = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
        assert_eq!(
            breaker_active.0.len(),
            1,
            "bridge_impacted_breaker_wall should fire Impacted(Wall) on the breaker entity"
        );
    }

    // =========================================================================
    // bridge_impacted_cell_wall
    // =========================================================================

    #[test]
    fn bridge_impacted_cell_wall_fires_impacted_wall_on_cell() {
        let mut app = test_app_cell_wall();

        let cell = app
            .world_mut()
            .spawn((
                impacted_wall_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let wall = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

        tick(&mut app);

        let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
        assert_eq!(
            cell_active.0.len(),
            1,
            "bridge_impacted_cell_wall should fire Impacted(Wall) on the cell entity"
        );
    }

    #[test]
    fn bridge_impacted_cell_wall_fires_impacted_cell_on_wall() {
        let mut app = test_app_cell_wall();

        let cell = app
            .world_mut()
            .spawn((
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        let wall = app
            .world_mut()
            .spawn((
                impacted_cell_bound_effects(),
                StagedEffects::default(),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();

        app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

        tick(&mut app);

        let wall_active = app.world().get::<ActiveSpeedBoosts>(wall).unwrap();
        assert_eq!(
            wall_active.0.len(),
            1,
            "bridge_impacted_cell_wall should fire Impacted(Cell) on the wall entity"
        );
    }
}
