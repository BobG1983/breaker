//! Group C — `burnout_update_heat` system (Behaviors C1–C14).
//!
//! Pins the per-tick heat-gauge update:
//! - Fill at rate `1/fill_duration` while moving.
//! - Drain at rate `1/drain_duration` while stationary.
//! - Clamp to `[0.0, 1.0]`.
//! - Arm `mega_bump_charged` when `heat` reaches `1.0`.
//! - Track `still_timer`; reset to `0.0` on a moving tick.
//! - On `still_timer` crossing `still_threshold`, instant full drain +
//!   insert `BurnoutSpeedBoost { remaining: speed_boost_duration }` + clear
//!   `mega_bump_charged`.
//! - Harness-safe: no-op on absent `BurnoutConfig`.
//! - Run-if gated on `protocol_active(Burnout)` + `in_state(Playing)`.
//! - Lazy-insert `BurnoutHeat::default()` on breakers lacking it.
//!
//! Tests are grouped by behavior topic across sibling files:
//!
//! - [`fill`]             — heat fills while moving + clamp at 1.0 + mega-bump arming (C1–C3)
//! - [`drain`]            — heat drains while stationary + clamp at 0.0 + below-epsilon (C4–C5, C10)
//! - [`still_threshold`]  — `still_timer` reset, instant-drain firing, post-fire fill (C6–C9)
//! - [`harness_safety`]   — config-absent + run-if gating (C11–C13)
//! - [`lazy_insert`]      — first-tick lazy insert + no re-reset (C14, C14b)

mod common;

mod drain;
mod fill;
mod harness_safety;
mod lazy_insert;
mod still_threshold;
