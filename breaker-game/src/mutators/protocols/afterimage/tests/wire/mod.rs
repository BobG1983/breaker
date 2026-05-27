//! Group I — `wire` wiring, schedule, gates (Behaviors I1–I14).
//!
//! Pins that `wire`-wired systems run under the correct schedules,
//! gated by `protocol_active(Afterimage)` + `in_state(NodeState::Playing)`,
//! and that the full in-tick chain
//! `spawn_phantom_breaker → check_phantom_bounce → GradeBump →
//! spawn_phantom_bolt` fires in order.

mod gating;
mod integration;
mod scheduling;
