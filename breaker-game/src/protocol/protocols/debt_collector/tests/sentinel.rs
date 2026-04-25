//! Group J — Builder-source emit-callsite pin (Behavior B28).
//!
//! Pins the `SourceId` carried on emitted bonus `DamageDealt<Cell>`
//! messages. The assertion goes through `SourceIdExt` builder entry points
//! — the test fails by construction if `kind_slug()` drifts.

use super::helpers::{
    build_debt_collector_app, collected_bonus_damage, seed_active_protocols_with_debt_collector,
    spawn_bolt_with_base_damage_and_cashout, write_bolt_impact_cell,
};
use crate::{prelude::*, protocol::definition::ProtocolKind};

#[test]
fn debt_collector_bonus_source_equals_builder_protocol_debt_collector() {
    let mut app = build_debt_collector_app();
    seed_active_protocols_with_debt_collector(&mut app, 0.5);
    let bolt = spawn_bolt_with_base_damage_and_cashout(&mut app, 10.0, 1.5);
    let cell = app.world_mut().spawn_empty().id();

    write_bolt_impact_cell(&mut app, bolt, cell);
    tick(&mut app);

    let msgs = collected_bonus_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 bonus DamageDealt");
    let expected = SourceId::protocol(ProtocolKind::DebtCollector).build();
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&expected),
        "bonus damage source must equal SourceId::protocol(DebtCollector).build()"
    );
}
