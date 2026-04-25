//! W7 Behavior 5 (Explode): builder-produced `SourceId` reaches every
//! emitted `DamageDealt<Cell>` byte-for-byte. Empty source string maps to
//! `None` on every emitted message.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn builder_produced_source_propagates_unchanged_to_every_emitted_message() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(30.0, 0.0), 100.0);
    let _c3 = spawn_cell_with_hp(&mut app, Vec2::new(50.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    let source_str = explode_chip_source_str();
    let expected = explode_chip_source();
    config.fire(source, &source_str, app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageDealt<Cell> messages"
    );
    for msg in &collector.0 {
        assert_eq!(
            msg.source,
            Some(expected.clone()),
            "every message must carry the same builder-produced SourceId",
        );
    }
}

#[test]
fn empty_source_string_propagates_as_none_on_every_emitted_message() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(20.0, 0.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(30.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 2);
    for msg in &collector.0 {
        assert_eq!(msg.source, None, "empty source string must produce None");
    }
}
