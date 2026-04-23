//! Generic 2D damage primitives for Bevy 0.18 games.
//!
//! This crate provides reusable damage building blocks — HP tracking,
//! dead-state marking, invulnerability windows, kill attribution, heal
//! ceilings, and source identifiers — intended for any 2D Bevy game. It
//! obeys the zero-game-knowledge contract documented in
//! `.claude/rules/rantzsoft-crates.md`: no game-specific vocabulary,
//! entities, or assumptions leak into this crate.
//!
//! P2 ships the core value types. Messages, systems, stacks, and the plugin
//! arrive in subsequent phases as a `Dmgable`-driven pipeline.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

mod components;
mod source_id;
mod traits;

pub use components::{Dead, HealCap, Hp, Invulnerable, KilledBy};
pub use source_id::SourceId;
pub use traits::Dmgable;

#[cfg(test)]
mod tests {
    // ── Behavior 28: every P2 type is reachable via the crate root ──

    #[test]
    fn all_core_types_accessible() {
        use crate::*;

        // Dmgable: a test-local dummy type binds the trait through the
        // re-exported path.
        #[derive(bevy::prelude::Component)]
        struct TestT;
        impl Dmgable for TestT {}
        // Construct `TestT` so it is not flagged as dead code — the
        // `impl Dmgable for TestT {}` above is the reason it exists.
        let _ = TestT;

        // Each of the other six types is constructed and inspected here so
        // every imported name is used — no unused-import warnings, no
        // `#[allow(unused_imports)]`.
        let hp = Hp::new(3.0);
        assert!(
            (hp.current - 3.0).abs() < f32::EPSILON,
            "expected 3.0, got {}",
            hp.current
        );

        let _ = Dead;
        let _ = Invulnerable;

        let kb = KilledBy { dealer: None };
        assert!(kb.dealer.is_none());

        assert_ne!(HealCap::Starting, HealCap::Max);

        let source = SourceId::from("src:alpha");
        let source_again = SourceId::from("src:alpha");
        assert_eq!(source, source_again);
    }
}
