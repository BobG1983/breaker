//! Group B — `iron_curtain_on_bolt_lost` wave behavior (Behaviors 5–18, B30).
//!
//! Pins the full damage-wave semantics: origin damage = `bolt_base *
//! damage_fraction`; full damage inside `falloff_start`; linear falloff past
//! `falloff_start` via `(1.0 - falloff_distance/max_distance).clamp(0.0, 1.0)`;
//! zero-damage clamp emits NO message; per-bolt base-damage scaling via
//! `BoltBaseDamage` with `DEFAULT_BOLT_BASE_DAMAGE` fallback; `Dead` cells are
//! filtered out by the query; `Invulnerable` cells DO receive wave messages at
//! emit stage (post-W7) and are zeroed downstream by `invulnerable_filter::<Cell>`
//! in `DmgSystems::ApplyDamage` — see the `w7_*` tests in `scheduling.rs` for
//! the end-to-end contract. The missing-breaker case is harness-safe;
//! multi-BoltLost in the same frame produce independent waves; and degenerate
//! `max_distance <= 0.0` does not panic.
//!
//! Tests are grouped by behavior topic across three sibling files:
//!
//! - [`falloff_math`] — origin damage + falloff math + scale (Behaviors 5–8)
//! - [`multi_target`] — multi-cell wave fan-out + multi-bolt independence
//!   + abs-symmetric below-breaker damage (Behaviors 9–11)
//! - [`edge_inputs`] — gating, missing-component fallbacks, cell filtering,
//!   degenerate config, source ID (Behaviors 12–18, B30)

mod edge_inputs;
mod falloff_math;
mod multi_target;

use crate::{
    mutators::protocols::definition::ProtocolKind,
    prelude::{SourceId, SourceIdExt},
};

/// Builder-produced `SourceId` for Iron Curtain wave damage. Shared
/// across all three test files for source-equality assertions.
pub(super) fn iron_curtain_source() -> SourceId {
    SourceId::protocol(ProtocolKind::IronCurtain).build()
}
