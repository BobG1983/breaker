//! Tests that `bridge_impacted_*` passes the correct `context_entity` to the
//! evaluate pipeline, so that inner `On(target)` nodes resolve to the specific
//! collision participant — not all entities of that type.
//!
//! Each test spawns multiple entities of the target type and verifies that only
//! the entity named in the collision message receives the transferred effect.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{components::Bolt, messages::BoltImpactCell},
    breaker::{
        components::Breaker,
        messages::{BreakerImpactCell, BreakerImpactWall},
    },
    cells::{components::Cell, messages::CellImpactWall},
    effect::core::*,
    wall::components::Wall,
};

/// Build `BoundEffects` with
/// `When(Impacted(X), [On(Y, permanent=false, [When(Died, [Do(SpeedBoost)])])])`.
///
/// Uses `When(Died, ...)` as the inner content so that after retargeting, the
/// transferred effect lands in the target entity's `StagedEffects` (rather than
/// being fired immediately via `Do`). This lets us positively assert the target
/// entity received the staged effect AND negatively assert non-targets didn't.
fn retarget_on_impacted(impacted_target: ImpactTarget, retarget_to: Target) -> BoundEffects {
    BoundEffects(vec![(
        "context_test".into(),
        EffectNode::When {
            trigger: Trigger::Impacted(impacted_target),
            then: vec![EffectNode::On {
                target: retarget_to,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            }],
        },
    )])
}

// ── bolt-cell: bolt side retargets to specific cell ────────────────

#[test]
fn impacted_bolt_cell_context_resolves_to_specific_cell() {
    let mut app = test_app_bolt_cell();

    // Bolt has: When(Impacted(Cell), [On(Cell, [Do(SpeedBoost)])])
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            retarget_on_impacted(ImpactTarget::Cell, Target::Cell),
            StagedEffects::default(),
        ))
        .id();

    // Three cells — only cell_b is in the collision
    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell {
        bolt,
        cell: cell_b,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(cell_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "cell_b SHOULD have staged effects — it was the impacted cell"
    );
    let staged_a = app.world().get::<StagedEffects>(cell_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(cell_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "cell_a should have no staged effects — it was not the impacted cell"
    );
    assert!(
        staged_c.0.is_empty(),
        "cell_c should have no staged effects — it was not the impacted cell"
    );
}

// ── bolt-cell: cell side retargets to specific bolt ────────────────

#[test]
fn impacted_bolt_cell_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_cell();

    // Three bolts — only bolt_b is in the collision
    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    // Cell has: When(Impacted(Bolt), [On(Bolt, [Do(SpeedBoost)])])
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            retarget_on_impacted(ImpactTarget::Bolt, Target::Bolt),
            StagedEffects::default(),
        ))
        .id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell {
        bolt: bolt_b,
        cell,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "bolt_b SHOULD have staged effects — it was the impacting bolt"
    );
    let staged_a = app.world().get::<StagedEffects>(bolt_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(bolt_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "bolt_a should have no staged effects — it was not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — it was not the impacting bolt"
    );
}

// ── breaker-cell: breaker side retargets to specific cell ──────────

#[test]
fn impacted_breaker_cell_context_resolves_to_specific_cell() {
    let mut app = test_app_breaker_cell();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(breaker).insert((
        retarget_on_impacted(ImpactTarget::Cell, Target::Cell),
        StagedEffects::default(),
    ));

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    app.insert_resource(TestBreakerImpactCellMsg(Some(BreakerImpactCell {
        breaker,
        cell: cell_b,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(cell_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "cell_b SHOULD have staged effects — it was the impacted cell"
    );
    let staged_a = app.world().get::<StagedEffects>(cell_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(cell_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "cell_a should have no staged effects — not the impacted cell"
    );
    assert!(
        staged_c.0.is_empty(),
        "cell_c should have no staged effects — not the impacted cell"
    );
}

// ── breaker-cell: cell side retargets to specific breaker ──────────

#[test]
fn impacted_breaker_cell_context_resolves_to_specific_breaker() {
    let mut app = test_app_breaker_cell();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker_a = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_a)
        .insert(StagedEffects::default());
    let breaker_b = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .extra()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_b)
        .insert(StagedEffects::default());

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            retarget_on_impacted(ImpactTarget::Breaker, Target::Breaker),
            StagedEffects::default(),
        ))
        .id();

    app.insert_resource(TestBreakerImpactCellMsg(Some(BreakerImpactCell {
        breaker: breaker_b,
        cell,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(breaker_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "breaker_b SHOULD have staged effects — it was the impacting breaker"
    );
    let staged_a = app.world().get::<StagedEffects>(breaker_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "breaker_a should have no staged effects — not the impacting breaker"
    );
}

// ── bolt-wall: bolt side retargets to specific wall ────────────────

#[test]
fn impacted_bolt_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_bolt_wall();

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            retarget_on_impacted(ImpactTarget::Wall, Target::Wall),
            StagedEffects::default(),
        ))
        .id();

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.insert_resource(TestBoltImpactWallMsg(Some(
        crate::bolt::messages::BoltImpactWall { bolt, wall: wall_b },
    )));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(wall_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "wall_b SHOULD have staged effects — it was the impacted wall"
    );
    let staged_a = app.world().get::<StagedEffects>(wall_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(wall_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "wall_a should have no staged effects — not the impacted wall"
    );
    assert!(
        staged_c.0.is_empty(),
        "wall_c should have no staged effects — not the impacted wall"
    );
}

// ── bolt-breaker: bolt side retargets to specific breaker ──────────

#[test]
fn impacted_bolt_breaker_context_resolves_to_specific_breaker() {
    let mut app = test_app_bolt_breaker();

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            retarget_on_impacted(ImpactTarget::Breaker, Target::Breaker),
            StagedEffects::default(),
        ))
        .id();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker_a = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_a)
        .insert(StagedEffects::default());
    let breaker_b = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .extra()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_b)
        .insert(StagedEffects::default());

    app.insert_resource(TestBoltImpactBreakerMsg(Some(
        crate::bolt::messages::BoltImpactBreaker {
            bolt,
            breaker: breaker_b,
        },
    )));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(breaker_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "breaker_b SHOULD have staged effects — it was the impacted breaker"
    );
    let staged_a = app.world().get::<StagedEffects>(breaker_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "breaker_a should have no staged effects — not the impacted breaker"
    );
}

// ── cell-wall: cell side retargets to specific wall ────────────────

#[test]
fn impacted_cell_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_cell_wall();

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            retarget_on_impacted(ImpactTarget::Wall, Target::Wall),
            StagedEffects::default(),
        ))
        .id();

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall {
        cell,
        wall: wall_b,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(wall_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "wall_b SHOULD have staged effects — it was the impacted wall"
    );
    let staged_a = app.world().get::<StagedEffects>(wall_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(wall_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "wall_a should have no staged effects — not the impacted wall"
    );
    assert!(
        staged_c.0.is_empty(),
        "wall_c should have no staged effects — not the impacted wall"
    );
}

// ── bolt-wall: wall side retargets to specific bolt ────────────────

#[test]
fn impacted_bolt_wall_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_wall();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let wall = app
        .world_mut()
        .spawn((
            Wall,
            retarget_on_impacted(ImpactTarget::Bolt, Target::Bolt),
            StagedEffects::default(),
        ))
        .id();

    app.insert_resource(TestBoltImpactWallMsg(Some(
        crate::bolt::messages::BoltImpactWall { bolt: bolt_b, wall },
    )));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "bolt_b SHOULD have staged effects — it was the impacting bolt"
    );
    let staged_a = app.world().get::<StagedEffects>(bolt_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(bolt_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "bolt_a should have no staged effects — not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — not the impacting bolt"
    );
}

// ── bolt-breaker: breaker side retargets to specific bolt ──────────

#[test]
fn impacted_bolt_breaker_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_breaker();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(breaker).insert((
        retarget_on_impacted(ImpactTarget::Bolt, Target::Bolt),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactBreakerMsg(Some(
        crate::bolt::messages::BoltImpactBreaker {
            bolt: bolt_b,
            breaker,
        },
    )));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(bolt_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "bolt_b SHOULD have staged effects — it was the impacting bolt"
    );
    let staged_a = app.world().get::<StagedEffects>(bolt_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(bolt_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "bolt_a should have no staged effects — not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — not the impacting bolt"
    );
}

// ── breaker-wall: breaker side retargets to specific wall ──────────

#[test]
fn impacted_breaker_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_breaker_wall();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(breaker).insert((
        retarget_on_impacted(ImpactTarget::Wall, Target::Wall),
        StagedEffects::default(),
    ));

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.insert_resource(TestBreakerImpactWallMsg(Some(BreakerImpactWall {
        breaker,
        wall: wall_b,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(wall_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "wall_b SHOULD have staged effects — it was the impacted wall"
    );
    let staged_a = app.world().get::<StagedEffects>(wall_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(wall_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "wall_a should have no staged effects — not the impacted wall"
    );
    assert!(
        staged_c.0.is_empty(),
        "wall_c should have no staged effects — not the impacted wall"
    );
}

// ── breaker-wall: wall side retargets to specific breaker ──────────

#[test]
fn impacted_breaker_wall_context_resolves_to_specific_breaker() {
    let mut app = test_app_breaker_wall();

    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker_a = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_a)
        .insert(StagedEffects::default());
    let breaker_b = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .extra()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_b)
        .insert(StagedEffects::default());

    let wall = app
        .world_mut()
        .spawn((
            Wall,
            retarget_on_impacted(ImpactTarget::Breaker, Target::Breaker),
            StagedEffects::default(),
        ))
        .id();

    app.insert_resource(TestBreakerImpactWallMsg(Some(BreakerImpactWall {
        breaker: breaker_b,
        wall,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(breaker_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "breaker_b SHOULD have staged effects — it was the impacting breaker"
    );
    let staged_a = app.world().get::<StagedEffects>(breaker_a).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "breaker_a should have no staged effects — not the impacting breaker"
    );
}

// ── cell-wall: wall side retargets to specific cell ────────────────

#[test]
fn impacted_cell_wall_context_resolves_to_specific_cell() {
    let mut app = test_app_cell_wall();

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    let wall = app
        .world_mut()
        .spawn((
            Wall,
            retarget_on_impacted(ImpactTarget::Cell, Target::Cell),
            StagedEffects::default(),
        ))
        .id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall {
        cell: cell_b,
        wall,
    })));

    tick(&mut app);

    let staged_b = app.world().get::<StagedEffects>(cell_b).unwrap();
    assert!(
        !staged_b.0.is_empty(),
        "cell_b SHOULD have staged effects — it was the impacted cell"
    );
    let staged_a = app.world().get::<StagedEffects>(cell_a).unwrap();
    let staged_c = app.world().get::<StagedEffects>(cell_c).unwrap();
    assert!(
        staged_a.0.is_empty(),
        "cell_a should have no staged effects — not the impacted cell"
    );
    assert!(
        staged_c.0.is_empty(),
        "cell_c should have no staged effects — not the impacted cell"
    );
}

// -----------------------------------------------------------------------
// Guard: On nodes must not linger in StagedEffects across evaluations.
// walk_staged_node consumes On nodes unconditionally, so this should always
// pass. If that invariant is ever broken, this test catches it.
// -----------------------------------------------------------------------

#[test]
fn on_node_does_not_linger_across_sequential_collisions() {
    let mut app = test_app_bolt_cell();

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            retarget_on_impacted(ImpactTarget::Cell, Target::Cell),
            StagedEffects::default(),
        ))
        .id();

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    // First collision: bolt hits cell_b
    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell {
        bolt,
        cell: cell_b,
    })));
    tick(&mut app);

    assert_eq!(
        app.world().get::<StagedEffects>(cell_b).unwrap().0.len(),
        1,
        "cell_b should have exactly 1 staged entry from first collision"
    );
    assert!(
        app.world()
            .get::<StagedEffects>(cell_a)
            .unwrap()
            .0
            .is_empty(),
        "cell_a should have nothing after first collision"
    );

    // Second collision: bolt hits cell_a
    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell {
        bolt,
        cell: cell_a,
    })));
    tick(&mut app);

    assert_eq!(
        app.world().get::<StagedEffects>(cell_a).unwrap().0.len(),
        1,
        "cell_a should have exactly 1 staged entry from second collision"
    );
    assert_eq!(
        app.world().get::<StagedEffects>(cell_b).unwrap().0.len(),
        1,
        "cell_b should STILL have exactly 1 — no duplicate from lingering On node"
    );
    assert!(
        app.world()
            .get::<StagedEffects>(cell_c)
            .unwrap()
            .0
            .is_empty(),
        "cell_c should have nothing — never involved in a collision"
    );

    // Bolt's own StagedEffects must be clean
    let bolt_staged = app.world().get::<StagedEffects>(bolt).unwrap();
    assert!(
        bolt_staged.0.is_empty(),
        "bolt's StagedEffects should be empty — On nodes must be consumed immediately"
    );
}
