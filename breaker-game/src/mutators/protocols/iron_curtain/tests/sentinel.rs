//! Group D — Builder-source emit-callsite pin (Behavior B30).
//!
//! Pins the `SourceId` carried on emitted Iron Curtain wave damage
//! messages. The assertion goes through `SourceIdExt` builder entry points
//! — the test fails by construction if `kind_slug()` drifts.

use bevy::prelude::*;

use super::helpers::{
    build_iron_curtain_app, collected_iron_curtain_damage, seed_active_protocols_with_iron_curtain,
    spawn_bolt_with_base_damage, spawn_breaker_at, spawn_cell_at, write_bolt_lost,
};
use crate::{mutators::protocols::definition::ProtocolKind, prelude::*};

#[test]
fn iron_curtain_wave_damage_source_equals_builder_protocol_iron_curtain() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert!(
        !msgs.is_empty(),
        "expected at least one Iron Curtain wave damage message"
    );
    let expected = SourceId::protocol(ProtocolKind::IronCurtain).build();
    for m in &msgs {
        assert_eq!(
            m.source.as_ref(),
            Some(&expected),
            "every wave damage source must equal \
             SourceId::protocol(IronCurtain).build()"
        );
    }
}
