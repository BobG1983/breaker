//! Group E — `afterimage_check_phantom_bounce` (Behaviors CB1–CB10).
//!
//! Pins the physical-overlap reflection system:
//! - Bolt overlapping a `PhantomBreaker` AABB + moving down → vertical-mirror
//!   velocity, snap position to top face + `BoltRadius`, emit exactly one
//!   `BumpPerformed { bolt: Some(b), breaker: phantom_entity, grade }`.
//! - Bolt not overlapping → no reflection, no `BumpPerformed`.
//! - Entry guard: bolt moving UP through the AABB → no reflection, no
//!   `BumpPerformed` (prevents re-bouncing a bolt that has already cleared
//!   the phantom).
//! - Grade forwarding: `Perfect` when real breaker's `BumpState.active &&
//!   timer <= perfect_window`; `Early` when `active && timer >
//!   perfect_window`; `Late` via the retroactive path when `!active` and
//!   `retroactive_grade(...)` returns `Late`. Grade is pinned through the
//!   emitted `BumpPerformed.grade` field ONLY — writer-tests MUST NOT
//!   call `retroactive_grade` directly (it is `pub(super)` in `bump` module
//!   and not reachable from this file; the test is a PURE emitted-grade
//!   assertion).
//! - `Without<PhantomBolt>` filter prevents cascade re-bounces from
//!   spawned phantom bolts.
//! - `bounced_this_frame` dedup guarantees ONE emission per bolt per frame
//!   even when the bolt overlaps multiple `PhantomBreakers`.
//! - No `PhantomBreaker` / no `Breaker` cases are no-op early returns.

use bevy::prelude::*;

use super::helpers::spawn_phantom_breaker_at;
use crate::breaker::components::BumpState;

mod dedup_multi_phantom;
mod edge_cases;
mod grade_forwarding;
mod overlap_detection;

// ── Canonical phantom-breaker factory ───────────────────────────────────────
//
// `PhantomBreaker` at origin with canonical `BaseWidth(100.0)` and
// `BaseHeight(20.0)` → AABB x ∈ [-50.0, 50.0], y ∈ [-10.0, 10.0]. Top face
// at y = 10.0.
pub(super) fn spawn_canonical_phantom_breaker(app: &mut App) -> Entity {
    // Helper already installs BaseWidth(100.0) / BaseHeight(20.0) and
    // CleanupOnExit. Override lifetime if callers need it; default 1.5s.
    spawn_phantom_breaker_at(app, Vec2::ZERO, 1.5)
}

pub(super) fn perfect_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.1, // <= perfect_window 0.2 → Perfect
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}

pub(super) fn early_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.3, // > perfect_window 0.2 → Early
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}
