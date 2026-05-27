//! Group B — Component shape (Behavior B3).
//!
//! Pins that afterimage re-uses the canonical `PhantomBolt` from
//! `crate::bolt::components` — NOT an afterimage-local shadow.
//! `PhantomLifetime` and `PhantomOwner` were deleted in Wave 5.

use crate::bolt::components::PhantomBolt;

// ── B3 — afterimage re-uses the canonical PhantomBolt ───────────────────────

#[test]
fn afterimage_reuses_canonical_phantom_bolt() {
    // Type reference resolves — compiles only when the import matches the
    // canonical type at bolt::components.
    let _: PhantomBolt = PhantomBolt;

    // Afterimage's system module re-exports the canonical type —
    // prove it by comparing TypeIds (robust against re-export chains).
    assert_eq!(
        std::any::TypeId::of::<PhantomBolt>(),
        std::any::TypeId::of::<super::super::system::PhantomBolt>(),
        "afterimage::system::PhantomBolt must be the SAME type as \
         the canonical PhantomBolt (bolt::components::phantom)"
    );
}
