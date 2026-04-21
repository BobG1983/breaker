//! Group D — `echo_strike_on_impact` (Behaviors 16–33).
//!
//! Pins the `BoltImpactCell` consumer:
//! - Harness-safe early return + reader drain when `EchoStrikeConfig` absent.
//! - Primed empty network registers impact target, emits zero echo damage.
//! - 1/2/3-echo networks emit the correct age-based falloff (newest / middle /
//!   oldest fractions assigned by position, not fixed slot — 2-echo SKIPS
//!   middle).
//! - FIFO eviction at `max_echoes`.
//! - Same-cell dedup + push-to-back.
//! - `BoltBaseDamage` fallback to `DEFAULT_BOLT_BASE_DAMAGE`.
//! - Zero impact damage emits zero echo-damage messages (network still updated).
//! - `source_chip` sentinel + `dealer` pinned on every emitted message.
//! - `EchoPrimed` removed after a primed impact.
//! - Despawned bolt tolerated; non-primed bolt impact is a no-op.
//! - Multi-bolt isolation.
//! - Gating: Echo Strike inactive / `NodeState != Playing` → no-op.
//! - Single-shot on same-tick pierce — only first impact processed, second
//!   ignored even though `EchoPrimed` removal is deferred.
//!
//! NOTE: Behavior 30 (echo-damage for a destroyed echo cell in the same
//! tick) is implicitly covered by Behaviors 20 and 29 — the system reads
//! the current entity list regardless of victim health. No separate test.

mod helpers;

mod base_damage;
mod config_and_empty;
mod falloff;
mod fifo_dedup;
mod gating;
mod sentinel_and_dealer;
mod tolerance_and_isolation;
