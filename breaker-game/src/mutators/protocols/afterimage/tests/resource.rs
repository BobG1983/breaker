//! Group B — Component shape (Behavior B3).
//!
//! Pins that afterimage re-uses the existing `PhantomBolt` / `PhantomLifetime` /
//! `PhantomOwner` from `crate::effect_v3::effects::phantom_bolt::components`
//! — NOT an afterimage-local shadow.
//!
//! B1, B2, B4 tested `PhantomBreakerLifetime` which was deleted in Wave 5.

use crate::{
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
};

// ── B3 — afterimage re-uses effect_v3 PhantomBolt / PhantomLifetime / ──────
//       PhantomOwner (not a local shadow).

#[test]
fn afterimage_reuses_effect_v3_phantom_components() {
    // (a) Type references resolve — these lines compile only when the
    //     imports match the canonical types.
    let _: PhantomBolt = PhantomBolt;
    let _: PhantomLifetime = PhantomLifetime(1.0);

    let mut app = TestAppBuilder::new().build();
    let dummy = app.world_mut().spawn_empty().id();
    let _: PhantomOwner = PhantomOwner(dummy);

    // (b) Afterimage's system module re-exports the canonical types —
    //     prove it by comparing type_name strings.
    let name = std::any::type_name::<super::super::system::PhantomBolt>();
    assert!(
        name.contains("effect_v3::effects::phantom_bolt"),
        "afterimage::system::PhantomBolt must be the canonical effect_v3 type, \
         got type_name = {name}"
    );
    assert_eq!(
        std::any::type_name::<PhantomBolt>(),
        std::any::type_name::<super::super::system::PhantomBolt>(),
        "afterimage's PhantomBolt import must be the SAME type as \
         effect_v3::effects::phantom_bolt::components::PhantomBolt"
    );
}
