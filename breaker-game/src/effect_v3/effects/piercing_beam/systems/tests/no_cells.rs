//! W7 Behavior 10 (PiercingBeam): no cells / dead-only cells produces zero
//! `DamageDealt<Cell>` and does not panic. The
//! `PiercingBeamEmissionRequested` is still written.

use bevy::prelude::*;

use super::{super::super::messages::PiercingBeamEmissionRequested, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn no_cells_no_panic_zero_damage_one_request() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    // No cells spawned.

    make_config().fire(source, "", app.world_mut());

    let req_buf = app
        .world()
        .resource::<Messages<PiercingBeamEmissionRequested>>();
    let request_count = req_buf.iter_current_update_messages().count();
    assert_eq!(
        request_count, 1,
        "fire() always writes one request, even with no cells",
    );

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "no cells means zero DamageDealt<Cell>",
    );
}

#[test]
fn dead_only_cells_yield_zero_damage_no_panic() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0)), Dead));
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 150.0)), Dead));

    make_config().fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "all-dead cell set produces zero DamageDealt<Cell> (Without<Dead>)",
    );
}
