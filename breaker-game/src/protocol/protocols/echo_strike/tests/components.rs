//! Group B — component shape (Behaviors 5–7).
//!
//! Pins:
//! - `EchoNetwork::default()` is an empty `VecDeque`.
//! - `EchoStrikeConfig` derive set includes `Resource`, `Debug`, `Clone`,
//!   `Copy`, and `PartialEq` (proven via `==`, `Clone::clone`, and
//!   pass-by-value).
//! - `EchoPrimed` is a unit-struct marker (insert/remove via `Commands`).

use bevy::{ecs::world::CommandQueue, prelude::Commands};

use super::super::system::{EchoNetwork, EchoPrimed, EchoStrikeConfig};
use crate::prelude::*;

// ── Behavior 5 — EchoNetwork::default() is an empty deque ───────────────────

#[test]
fn echo_network_default_is_empty() {
    let n = EchoNetwork::default();
    assert!(
        n.echoes.is_empty(),
        "default EchoNetwork.echoes must be empty"
    );
    assert_eq!(
        n.echoes.len(),
        0,
        "default EchoNetwork.echoes.len() must be 0, got {}",
        n.echoes.len()
    );
}

// ── Behavior 5 (edge case) — two default networks are both empty ───────────-

#[test]
fn two_echo_network_defaults_are_both_empty() {
    let a = EchoNetwork::default();
    let b = EchoNetwork::default();
    assert!(
        a.echoes.is_empty() && b.echoes.is_empty(),
        "both default EchoNetwork values should be empty"
    );
}

// ── Behavior 6 — EchoStrikeConfig is Copy + Clone + PartialEq ──────────────-

#[test]
fn echo_strike_config_is_copy_clone_partial_eq() {
    fn takes_by_value(cfg: EchoStrikeConfig) -> u32 {
        cfg.max_echoes
    }
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = EchoStrikeConfig {
        max_echoes:      3,
        newest_fraction: 0.5,
        middle_fraction: 0.25,
        oldest_fraction: 0.1,
    };

    // PartialEq: two identical values compare equal.
    let same = EchoStrikeConfig {
        max_echoes:      3,
        newest_fraction: 0.5,
        middle_fraction: 0.25,
        oldest_fraction: 0.1,
    };
    assert_eq!(orig, same, "identical configs must compare equal");

    // Copy: two pass-by-value assignments both succeed (original not moved).
    let copy1 = orig;
    let copy2 = orig;
    assert_eq!(orig, copy1, "copy must equal original");
    assert_eq!(copy1, copy2, "both copies must equal each other");

    // Copy: pass-by-value into a function compiles and original remains usable.
    assert_eq!(takes_by_value(orig), 3);
    assert_eq!(orig.max_echoes, 3);

    // Clone: routed through a generic that requires `T: Clone` — proves the
    // derive exists at compile time without tripping `clippy::clone_on_copy`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");
}

// ── Behavior 7 — EchoPrimed is a unit-struct marker ────────────────────────-

#[test]
fn echo_primed_marker_inserts_and_is_queryable() {
    let mut app = TestAppBuilder::new().build();
    let bolt = app.world_mut().spawn_empty().id();

    // Insert via Commands (the production system's only insertion path).
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands.entity(bolt).insert(EchoPrimed);
    }
    queue.apply(app.world_mut());

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_some(),
        "EchoPrimed should be attached to the bolt after insert"
    );
}

// ── Behavior 7 (edge case) — EchoPrimed can be removed ─────────────────────-

#[test]
fn echo_primed_marker_can_be_removed() {
    let mut app = TestAppBuilder::new().build();
    let bolt = app.world_mut().spawn(EchoPrimed).id();
    assert!(app.world().get::<EchoPrimed>(bolt).is_some());

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands.entity(bolt).remove::<EchoPrimed>();
    }
    queue.apply(app.world_mut());

    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed should be removable via Commands"
    );
}
