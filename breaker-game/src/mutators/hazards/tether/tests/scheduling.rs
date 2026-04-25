//! W5 — whitelist regression pin for `tether_emit_partner`.
//!
//! `tether_emit_partner` is a ripple emitter that fires in
//! `DmgSystems::PostApplyDamage` after a primary `DamageDealt<Cell>` applies.
//! It must NEVER be moved into `DmgSystems::EmitDamage`.

use bevy::prelude::*;

use super::super::system::{tether_emit_partner, wire};
use crate::{
    mutators::{
        hazards::resources::ActiveHazards, plugin::wire_damage_chain,
        protocols::resources::ActiveProtocols,
    },
    prelude::*,
};

fn tether_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_resource::<ActiveProtocols>()
        .with_effects_pipeline()
        .build();
    wire(&mut app);
    wire_damage_chain(&mut app);
    app
}

#[test]
fn tether_emit_partner_is_pinned_to_post_apply_damage() {
    let mut app = tether_scheduling_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            tether_emit_partner,
            DmgSystems::PostApplyDamage,
        ),
        "tether_emit_partner must remain in DmgSystems::PostApplyDamage."
    );
}

#[test]
fn tether_emit_partner_is_not_in_emit_damage() {
    let mut app = tether_scheduling_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            tether_emit_partner,
            DmgSystems::EmitDamage,
        ),
        "tether_emit_partner must NOT be a member of DmgSystems::EmitDamage."
    );
}
