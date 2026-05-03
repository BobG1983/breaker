//! Phantom-breaker collision gating tests.
//!
//! Phantoms receive only a core velocity flip; tilt, spread, `LastImpact` and
//! `PiercingRemaining` effects are suppressed. In a mixed world the current
//! `single()` call returns `Err(MultipleEntities)` so most tests fail today.
//! The regression group is the "do not break the real breaker" guard.

mod helpers;
mod mixed_world;
mod phantom_only_gating;
mod regression;
