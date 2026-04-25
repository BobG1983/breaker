//! W5 — whitelist regression pin for `diffusion_emit_rings`.
//!
//! `diffusion_emit_rings` is a ripple emitter that fires in
//! `DmgSystems::PostApplyDamage` after a primary `DamageDealt<Cell>` applies.
//! It must NEVER be moved into `DmgSystems::EmitDamage`. These assertions
//! catch two failure modes:
//!
//! 1. Accidental removal from `PostApplyDamage` (e.g., misregistration).
//! 2. Accidental move into `EmitDamage` (which would let Diffusion emit
//!    rings BEFORE any primary damage applies — semantically wrong).

use bevy::prelude::*;

use super::super::system::{diffusion_emit_rings, wire};
use crate::{mutators::hazards::resources::ActiveHazards, prelude::*};

fn diffusion_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_effects_pipeline()
        .build();
    wire(&mut app);
    app
}

#[test]
fn diffusion_emit_rings_is_pinned_to_post_apply_damage() {
    let mut app = diffusion_scheduling_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_emit_rings,
            DmgSystems::PostApplyDamage,
        ),
        "diffusion_emit_rings must remain in DmgSystems::PostApplyDamage. If \
         it is removed entirely, the ripple semantics break (diffusion rings \
         never fire). If it is moved into EmitDamage, rings fire BEFORE any \
         primary damage applies — semantically wrong."
    );
}

#[test]
fn diffusion_emit_rings_is_not_in_emit_damage() {
    let mut app = diffusion_scheduling_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            diffusion_emit_rings,
            DmgSystems::EmitDamage,
        ),
        "diffusion_emit_rings must NOT be a member of DmgSystems::EmitDamage. \
         That would let Diffusion emit ripple rings in the emit stage — BEFORE \
         any primary damage applies — which inverts the intended post-apply \
         semantics."
    );
}
