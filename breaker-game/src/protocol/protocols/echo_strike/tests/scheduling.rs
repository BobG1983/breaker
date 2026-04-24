//! W5 — whitelist regression pin for `echo_strike_emit_siblings`.
//!
//! `echo_strike_emit_siblings` is a ripple emitter that fires in
//! `DmgSystems::PostApplyDamage` after a primary `DamageDealt<Cell>` applies.
//! It must NEVER be moved into `DmgSystems::EmitDamage`.

use bevy::prelude::*;

use super::super::system::{echo_strike_emit_siblings, register};
use crate::{prelude::*, protocol::resources::ActiveProtocols};

fn echo_strike_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_effects_pipeline()
        .build();
    register(&mut app);
    app
}

#[test]
fn echo_strike_emit_siblings_is_pinned_to_post_apply_damage() {
    let mut app = echo_strike_scheduling_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            echo_strike_emit_siblings,
            DmgSystems::PostApplyDamage,
        ),
        "echo_strike_emit_siblings must remain in DmgSystems::PostApplyDamage."
    );
}

#[test]
fn echo_strike_emit_siblings_is_not_in_emit_damage() {
    let mut app = echo_strike_scheduling_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            echo_strike_emit_siblings,
            DmgSystems::EmitDamage,
        ),
        "echo_strike_emit_siblings must NOT be a member of DmgSystems::EmitDamage."
    );
}
