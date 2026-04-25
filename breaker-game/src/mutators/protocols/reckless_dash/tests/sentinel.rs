//! Group G — Builder-source emit-callsite pin (Behavior B31).
//!
//! Pins the `SourceId` carried on emitted Reckless Dash amplified damage
//! messages. The assertion goes through `SourceIdExt` builder entry points
//! — the test fails by construction if `kind_slug()` drifts.

use super::{
    super::system::RiskyDamageBoost,
    helpers::{
        build_reckless_dash_app, collected_reckless_dash_damage,
        seed_active_protocols_with_reckless_dash, spawn_bolt_with_base_damage, spawn_cell_empty,
        write_bolt_impact_cell,
    },
};
use crate::{mutators::protocols::definition::ProtocolKind, prelude::*};

#[test]
fn reckless_dash_amplified_damage_source_equals_builder_protocol_reckless_dash() {
    let mut app = build_reckless_dash_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 4.0 });
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 amplified damage message");
    let expected = SourceId::protocol(ProtocolKind::RecklessDash).build();
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&expected),
        "amplified damage source must equal \
         SourceId::protocol(RecklessDash).build()"
    );
}
