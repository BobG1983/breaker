//! W2 Behaviors 27–29 + B35: `diffusion_emit_rings` drains the pending queue
//! in `DmgSystems::PostApplyDamage`.
//!
//! Tests are grouped by behavior topic across sibling files:
//!
//! - [`splitting`]          — flat split of `shared` across candidates (B27)
//! - [`invulnerable`]       — invulnerable target → no emission (B28)
//! - [`attenuation_floor`]  — per-neighbor 1.0 floor (inclusive, B29)
//! - [`source`]             — builder-produced `SourceId` with instance (B35)

mod helpers;

mod attenuation_floor;
mod invulnerable;
mod source;
mod splitting;
