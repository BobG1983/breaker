//! Group I — Builder-source emit-callsite pins (Behaviors B27, B41).
//!
//! Pin the `SourceId` carried on emitted protocol artifacts (the shockwave
//! entity's `EffectSourceChip`) and on amplified `DamageDealt<Cell>` messages.
//! Every assertion here goes through `SourceIdExt` builder entry points —
//! the test fails by construction if `kind_slug()` drifts or the `action()`
//! segment changes.

use bevy::prelude::*;

use super::helpers::{
    build_burnout_app, collected_burnout_damage, count_shockwave_sources,
    install_burnout_damage_boost, seed_active_protocols_with_burnout, set_heat_state,
    shockwave_source_chip, shockwave_source_entities, spawn_bolt_with_base_damage,
    spawn_breaker_stationary, spawn_cell_empty, write_bolt_impact_cell, write_bump_performed,
};
use crate::{
    breaker::messages::BumpGrade, mutators::protocols::definition::ProtocolKind, prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── B27: dispatch_shockwave's source equals builder protocol:burnout:shockwave ──

#[test]
fn burnout_dispatch_shockwave_source_equals_builder_protocol_burnout_shockwave() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        count_shockwave_sources(&mut app),
        1,
        "exactly ONE shockwave entity must spawn on mega-bump"
    );
    let entities = shockwave_source_entities(&mut app);
    let entity = *entities.first().expect("at least one shockwave entity");

    let expected = SourceId::protocol(ProtocolKind::Burnout)
        .action("shockwave")
        .build();
    let expected_str = expected.0.into_owned();

    assert_eq!(
        shockwave_source_chip(&app, entity),
        Some(expected_str),
        "shockwave's EffectSourceChip must equal \
         SourceId::protocol(Burnout).action(\"shockwave\").build()"
    );
}

// ── B41: amplify_damage emits builder source on every cell impact ──

#[test]
fn burnout_amplify_damage_source_equals_builder_protocol_burnout() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 amplified DamageDealt");
    let expected = SourceId::protocol(ProtocolKind::Burnout).build();
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&expected),
        "amplify_damage's source must equal SourceId::protocol(Burnout).build()"
    );
}
