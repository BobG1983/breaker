//! Tests for `ActiveDamageBoosts` interaction with piercing lookahead.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::helpers::*;
use crate::{
    bolt::components::PiercingRemaining,
    effect::effects::{damage_boost::ActiveDamageBoosts, piercing::ActivePiercings},
};

/// Spec behavior 3: Piercing lookahead uses `ActiveDamageBoosts` — pierce succeeds.
/// Bolt with `ActivePiercings(vec![1])`, `PiercingRemaining(1)`, `ActiveDamageBoosts(vec![1.5])`,
/// cell with `CellHealth(12.0)`.
/// Boosted damage = 10.0 * 1.5 = 15.0 >= 12.0 => would destroy => bolt pierces.
/// `PiercingRemaining` decremented to 0.
#[test]
fn piercing_with_effective_damage_multiplier_uses_boosted_damage_for_lookahead() {
    let mut app = test_app();
    let bc = super::super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 12.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        ActivePiercings(vec![1]),
        PiercingRemaining(1),
        ActiveDamageBoosts(vec![1.5]),
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
        "bolt with ActiveDamageBoosts(1.5) should pierce 12-HP cell (boosted damage=15), got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement from 1 to 0 after piercing"
    );
}

/// Spec behavior 4: Piercing lookahead without `ActiveDamageBoosts` — pierce fails, bolt reflects.
/// Bolt with `ActivePiercings(vec![1])`, `PiercingRemaining(1)`, NO `ActiveDamageBoosts`,
/// cell with `CellHealth(12.0)`.
/// Base damage = 10.0 < 12.0 => cell not destroyed => bolt reflects.
/// `PiercingRemaining` unchanged at 1.
#[test]
fn piercing_without_effective_damage_multiplier_reflects_off_tough_cell() {
    let mut app = test_app();
    let bc = super::super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 12.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));
    // NO ActiveDamageBoosts => default base damage 10.0

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt without ActiveDamageBoosts should reflect off 12-HP cell (base damage=10), got vy={}",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should remain 1 when pierce fails (cell not destroyed)"
    );
}
