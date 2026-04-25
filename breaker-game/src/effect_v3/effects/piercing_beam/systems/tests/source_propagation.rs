//! W7 Behavior 5 (PiercingBeam): builder-produced `SourceId` reaches every
//! emitted `DamageDealt<Cell>` byte-for-byte. Empty source string maps to
//! `None`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn builder_produced_source_propagates_unchanged_to_every_message() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);
    let _c3 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 300.0), 100.0);

    let source_str = piercing_beam_chip_source_str();
    let expected = piercing_beam_chip_source();
    make_config().fire(source, &source_str, app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 3);
    for msg in &collector.0 {
        assert_eq!(
            msg.source,
            Some(expected.clone()),
            "every message must carry the same builder-produced SourceId",
        );
    }
}

#[test]
fn empty_source_string_propagates_as_none_on_every_message() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let _c1 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 50.0), 100.0);
    let _c2 = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);

    make_config().fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(collector.0.len(), 2);
    for msg in &collector.0 {
        assert_eq!(msg.source, None, "empty source string must produce None");
    }
}
