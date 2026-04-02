//! Tests that `bridge_impact_*` (global broadcast) passes the correct
//! `context_entity` so that inner `On(target)` nodes resolve to the specific
//! collision participant — not all entities of that type.
//!
//! Unlike `impacted` bridges (targeted), `impact` bridges fire on ALL entities
//! with `BoundEffects`. The `context_entity` ensures retargeting still resolves
//! to the specific entity from the collision message.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{
        components::Bolt,
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    },
    breaker::{
        components::Breaker,
        messages::{BreakerImpactCell, BreakerImpactWall},
    },
    cells::{components::Cell, messages::CellImpactWall},
    effect::core::*,
    wall::components::Wall,
};

/// Build `BoundEffects` with
/// `When(Impact(X), [On(Y, permanent=false, [When(Died, [Do(SpeedBoost)])])])`.
///
/// Uses `When(Died, ...)` as the inner content so that after retargeting, the
/// transferred effect lands in the target entity's `StagedEffects` (rather than
/// being fired immediately via `Do`). This lets us positively assert the target
/// entity received the staged effect AND negatively assert non-targets didn't.
fn retarget_on_impact(impact_target: ImpactTarget, retarget_to: Target) -> BoundEffects {
    BoundEffects(vec![(
        "context_test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(impact_target),
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

// ── bolt-cell: observer retargets to specific cell ─────────────────

#[test]
fn impact_bolt_cell_context_resolves_to_specific_cell() {
    let mut app = test_app_bolt_cell();

    let bolt = app.world_mut().spawn((Bolt,)).id();

    // Three cells — only cell_b is in the collision
    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    // Observer entity: listens for Impact(Cell), retargets to Cell
    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Cell, Target::Cell),
        StagedEffects::default(),
    ));

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
        "cell_a should have no staged effects — not the impacted cell"
    );
    assert!(
        staged_c.0.is_empty(),
        "cell_c should have no staged effects — not the impacted cell"
    );
}

// ── bolt-cell: observer retargets to specific bolt ─────────────────

#[test]
fn impact_bolt_cell_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_cell();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let cell = app.world_mut().spawn((Cell,)).id();

    // Observer entity: listens for Impact(Bolt), retargets to Bolt
    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Bolt, Target::Bolt),
        StagedEffects::default(),
    ));

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
        "bolt_a should have no staged effects — not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — not the impacting bolt"
    );
}

// ── breaker-cell: observer retargets to specific cell ──────────────

#[test]
fn impact_breaker_cell_context_resolves_to_specific_cell() {
    let mut app = test_app_breaker_cell();

    let breaker = app.world_mut().spawn((Breaker,)).id();

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Cell, Target::Cell),
        StagedEffects::default(),
    ));

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

// ── cell-wall: observer retargets to specific wall ─────────────────

#[test]
fn impact_cell_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_cell_wall();

    let cell = app.world_mut().spawn((Cell,)).id();

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Wall, Target::Wall),
        StagedEffects::default(),
    ));

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

// ── breaker-cell: observer retargets to specific breaker ───────────

#[test]
fn impact_breaker_cell_context_resolves_to_specific_breaker() {
    let mut app = test_app_breaker_cell();

    let breaker_a = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();
    let breaker_b = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();

    let cell = app.world_mut().spawn((Cell,)).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Breaker, Target::Breaker),
        StagedEffects::default(),
    ));

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

// ── bolt-wall: observer retargets to specific wall ─────────────────

#[test]
fn impact_bolt_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_bolt_wall();

    let bolt = app.world_mut().spawn((Bolt,)).id();

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Wall, Target::Wall),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactWallMsg(Some(BoltImpactWall {
        bolt,
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

// ── bolt-wall: observer retargets to specific bolt ─────────────────

#[test]
fn impact_bolt_wall_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_wall();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let wall = app.world_mut().spawn((Wall,)).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Bolt, Target::Bolt),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactWallMsg(Some(BoltImpactWall {
        bolt: bolt_b,
        wall,
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
        "bolt_a should have no staged effects — not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — not the impacting bolt"
    );
}

// ── bolt-breaker: observer retargets to specific breaker ───────────

#[test]
fn impact_bolt_breaker_context_resolves_to_specific_breaker() {
    let mut app = test_app_bolt_breaker();

    let bolt = app.world_mut().spawn((Bolt,)).id();

    let breaker_a = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();
    let breaker_b = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Breaker, Target::Breaker),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactBreakerMsg(Some(BoltImpactBreaker {
        bolt,
        breaker: breaker_b,
    })));

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

// ── bolt-breaker: observer retargets to specific bolt ──────────────

#[test]
fn impact_bolt_breaker_context_resolves_to_specific_bolt() {
    let mut app = test_app_bolt_breaker();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_c = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let breaker = app.world_mut().spawn((Breaker,)).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Bolt, Target::Bolt),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactBreakerMsg(Some(BoltImpactBreaker {
        bolt: bolt_b,
        breaker,
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
        "bolt_a should have no staged effects — not the impacting bolt"
    );
    assert!(
        staged_c.0.is_empty(),
        "bolt_c should have no staged effects — not the impacting bolt"
    );
}

// ── breaker-wall: observer retargets to specific wall ──────────────

#[test]
fn impact_breaker_wall_context_resolves_to_specific_wall() {
    let mut app = test_app_breaker_wall();

    let breaker = app.world_mut().spawn((Breaker,)).id();

    let wall_a = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_b = app.world_mut().spawn((Wall, StagedEffects::default())).id();
    let wall_c = app.world_mut().spawn((Wall, StagedEffects::default())).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Wall, Target::Wall),
        StagedEffects::default(),
    ));

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

// ── breaker-wall: observer retargets to specific breaker ───────────

#[test]
fn impact_breaker_wall_context_resolves_to_specific_breaker() {
    let mut app = test_app_breaker_wall();

    let breaker_a = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();
    let breaker_b = app
        .world_mut()
        .spawn((Breaker, StagedEffects::default()))
        .id();

    let wall = app.world_mut().spawn((Wall,)).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Breaker, Target::Breaker),
        StagedEffects::default(),
    ));

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

// ── cell-wall: observer retargets to specific cell ─────────────────

#[test]
fn impact_cell_wall_context_resolves_to_specific_cell() {
    let mut app = test_app_cell_wall();

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_c = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    let wall = app.world_mut().spawn((Wall,)).id();

    app.world_mut().spawn((
        retarget_on_impact(ImpactTarget::Cell, Target::Cell),
        StagedEffects::default(),
    ));

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

// ── Corner case: Impact dual retarget — both participants from one collision ──

#[test]
fn impact_bolt_cell_dual_retarget_resolves_both() {
    let mut app = test_app_bolt_cell();

    let bolt_a = app.world_mut().spawn((Bolt, StagedEffects::default())).id();
    let bolt_b = app.world_mut().spawn((Bolt, StagedEffects::default())).id();

    let cell_a = app.world_mut().spawn((Cell, StagedEffects::default())).id();
    let cell_b = app.world_mut().spawn((Cell, StagedEffects::default())).id();

    // Observer: When(Impact(Cell), [On(Cell, ...), On(Bolt, ...)])
    app.world_mut().spawn((
        BoundEffects(vec![
            (
                "dual_cell".into(),
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::On {
                        target: Target::Cell,
                        permanent: false,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                        }],
                    }],
                },
            ),
            (
                "dual_bolt".into(),
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::On {
                        target: Target::Bolt,
                        permanent: false,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
                        }],
                    }],
                },
            ),
        ]),
        StagedEffects::default(),
    ));

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell {
        bolt: bolt_b,
        cell: cell_b,
    })));

    tick(&mut app);

    assert!(
        !app.world()
            .get::<StagedEffects>(cell_b)
            .unwrap()
            .0
            .is_empty(),
        "cell_b SHOULD have staged effects from On(Cell) retarget"
    );
    assert!(
        app.world()
            .get::<StagedEffects>(cell_a)
            .unwrap()
            .0
            .is_empty(),
        "cell_a should be empty"
    );
    assert!(
        !app.world()
            .get::<StagedEffects>(bolt_b)
            .unwrap()
            .0
            .is_empty(),
        "bolt_b SHOULD have staged effects from On(Bolt) retarget"
    );
    assert!(
        app.world()
            .get::<StagedEffects>(bolt_a)
            .unwrap()
            .0
            .is_empty(),
        "bolt_a should be empty"
    );
}
