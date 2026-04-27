//! Tests for `DamageBoostStack` interaction with piercing lookahead.

use crate::{
    bolt::{
        components::PiercingRemaining,
        systems::bolt_cell_collision::tests::helpers::*,
        test_utils::{damage_stack, piercing_stack},
    },
    cells::resources::CellConfig,
    prelude::*,
};

/// Spec behavior 3: Piercing lookahead uses `DamageBoostStack` — pierce succeeds.
/// Bolt with `piercing_stack(&[1])`, `PiercingRemaining(1)`, `damage_stack(&[1.5])`,
/// cell with `CellHealth(12.0)`.
/// Boosted damage = 10.0 * 1.5 = 15.0 >= 12.0 => would destroy => bolt pierces.
/// `PiercingRemaining` decremented to 0.
#[test]
fn piercing_with_effective_damage_multiplier_uses_boosted_damage_for_lookahead() {
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 12.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        piercing_stack(&[1]),
        PiercingRemaining(1),
        damage_stack(&[1.5]),
    ));

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt with DamageBoostStack(1.5) should pierce 12-HP cell (boosted damage=15), got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement from 1 to 0 after piercing"
    );
}

/// Spec behavior 4: Piercing lookahead without `DamageBoostStack` — pierce fails, bolt reflects.
/// Bolt with `piercing_stack(&[1])`, `PiercingRemaining(1)`, NO `DamageBoostStack`,
/// cell with `CellHealth(12.0)`.
/// Base damage = 10.0 < 12.0 => cell not destroyed => bolt reflects.
/// `PiercingRemaining` unchanged at 1.
#[test]
fn piercing_without_effective_damage_multiplier_reflects_off_tough_cell() {
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 12.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[1]), PiercingRemaining(1)));
    // NO DamageBoostStack => default base damage 10.0

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt without DamageBoostStack should reflect off 12-HP cell (base damage=10), got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should remain 1 when pierce fails (cell not destroyed)"
    );
}

/// Wave 2 behavior change: pierce decision counts the bolt's `DamageBoostStack`
/// **one-shot** lane (in addition to the persistent lane it already counted).
///
/// Bolt with `piercing_stack(&[1])`, `PiercingRemaining(1)`, and a
/// `DamageBoostStack` containing only one one-shot of `10.0` (persistent lane
/// empty). Cell at HP 50.0, no `VulnerableStack`. Predicted damage =
/// `10.0 * 1.0 (boost-persistent) * 10.0 (boost-one-shots) * 1.0 (vuln) = 100.0`,
/// which is >= 50.0 → pierce. The full pipeline runs in the same tick, so the
/// one-shot is consumed and the cell's `Hp.current` reflects the post-pipeline
/// result.
#[test]
fn piercing_one_shot_boost_counts_for_pierce_decision() {
    let mut app = test_app_with_full_pipeline();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let starting_hp = 50.0;
    let cell = spawn_cell_with_health(&mut app, 0.0, cell_y, starting_hp);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    // `DamageBoostStack` does NOT derive `Clone` and `add_one_shot()` returns
    // `()` — must be built as a separate binding, then inserted by move.
    let mut boost_stack = DamageBoostStack::default();
    boost_stack.add_one_shot(10.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        piercing_stack(&[1]),
        PiercingRemaining(1),
        boost_stack,
    ));

    tick(&mut app);

    // Pierces — velocity remains positive in y (NOT reflected).
    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt with one-shot boost 10.0 should pierce 50-HP cell (predicted damage=100.0), got vy={}",
        vel.0.y
    );

    // PiercingRemaining decremented from 1 to 0.
    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement from 1 to 0 after piercing"
    );

    // Cell destroyed by overkill (50.0 starting - 100.0 damage). The full
    // pipeline despawns the entity on fatal damage in the same tick, so the
    // observable signal is that the `Hp` component is no longer reachable.
    assert!(
        app.world().get::<Hp>(cell).is_none(),
        "cell should be destroyed (overkill 100.0 vs 50.0 HP — entity despawned)"
    );

    // Boost stack fully drained — both lanes empty after one-shot consumption.
    // `is_empty()` is the load-bearing assertion: `aggregate_one_shots() == 1.0`
    // is ambiguous (returns 1.0 for both an empty lane AND a surviving 1.0
    // one-shot), so it cannot prove the drain on its own.
    let stack = app
        .world()
        .get::<DamageBoostStack>(bolt_entity)
        .expect("bolt should still have DamageBoostStack");
    assert!(
        stack.is_empty(),
        "DamageBoostStack should be fully empty after one-shot consumption"
    );

    // One DamageDealt<Cell> message at the post-pipeline amount of 100.0.
    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 100.0).abs() < f32::EPSILON,
        "post-pipeline DamageDealt<Cell>.amount should be 10.0 * 10.0 = 100.0, got {}",
        msgs.0[0].amount
    );
}

/// Wave 2 behavior change (cell-side symmetric): pierce decision counts the
/// cell's `VulnerableStack` **one-shot** lane (in addition to the persistent
/// lane it already counted).
///
/// Bolt with `piercing_stack(&[1])`, `PiercingRemaining(1)`, NO
/// `DamageBoostStack` (base damage `10.0`). Cell at HP 50.0, with a
/// `VulnerableStack` containing only one one-shot of `10.0` (persistent lane
/// empty — `spawn_vulnerable_cell` only seeds the persistent lane and so is
/// not used here). Predicted damage =
/// `10.0 * 1.0 (boost-persistent) * 1.0 (boost-one-shots) * 1.0
/// (vuln-persistent) * 10.0 (vuln-one-shots) = 100.0`, which is >= 50.0 →
/// pierce. The full pipeline runs in the same tick.
#[test]
fn piercing_one_shot_vulnerability_counts_for_pierce_decision() {
    let mut app = test_app_with_full_pipeline();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let starting_hp = 50.0;
    let cell = spawn_cell_with_health(&mut app, 0.0, cell_y, starting_hp);
    // `VulnerableStack` does NOT derive `Clone` and `add_one_shot()` returns
    // `()` — must be built as a separate binding, then inserted by move.
    // `spawn_vulnerable_cell` is unsuitable: it seeds the persistent lane.
    let mut vuln_stack = VulnerableStack::default();
    vuln_stack.add_one_shot(10.0);
    app.world_mut().entity_mut(cell).insert(vuln_stack);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[1]), PiercingRemaining(1)));

    tick(&mut app);

    // Pierces — velocity remains positive in y (NOT reflected).
    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt against cell with one-shot vulnerability 10.0 should pierce 50-HP cell (predicted damage=100.0), got vy={}",
        vel.0.y
    );

    // PiercingRemaining decremented from 1 to 0.
    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement from 1 to 0 after piercing"
    );

    // Cell destroyed by overkill (50.0 starting - 100.0 damage). Same
    // pipeline-despawn behavior as the boost-side test. The cell entity is
    // gone, so its `VulnerableStack` is unreachable — the
    // `DamageDealt<Cell>.amount == 100.0` assertion below carries the
    // one-shot-consumption proof (amount would be 10.0 if the one-shot was
    // skipped instead of applied).
    assert!(
        app.world().get::<Hp>(cell).is_none(),
        "cell should be destroyed (overkill 100.0 vs 50.0 HP — entity despawned)"
    );

    // One DamageDealt<Cell> message at the post-pipeline amount of 100.0.
    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 100.0).abs() < f32::EPSILON,
        "post-pipeline DamageDealt<Cell>.amount should be 10.0 * 10.0 = 100.0, got {}",
        msgs.0[0].amount
    );
}
