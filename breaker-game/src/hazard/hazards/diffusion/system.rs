use bevy::prelude::*;

use crate::hazard::definition::HazardTuning;

/// Module-level cap on diffusion share — prevents the "surrounded cell takes 0
/// damage" degenerate case at high stacks. Percentage units. Lives at module
/// scope (not `impl DiffusionConfig`) so callers (including tests) can import
/// it directly without going through the resource type.
pub(crate) const DIFFUSION_SHARE_CAP_PERCENT: f32 = 95.0;

/// Per-run Diffusion tuning, in percentage units.
///
/// Translates from [`HazardTuning::Diffusion`]'s fractional authoring fields
/// via a `* 100.0` conversion at activation time; this resource stores percents
/// because the design-doc formula uses percent-unit math.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DiffusionConfig {
    /// Share percent added by the hazard at stack 1 (e.g. `20.0` for 20%).
    pub(crate) base_share_percent:      f32,
    /// Additional share percent per stack beyond the first.
    pub(crate) share_per_level_percent: f32,
    /// Stacks required to escalate the BFS depth by one. Must be > 0 in config;
    /// pathological `0` is clamped to `1` inside [`DiffusionConfig::depth`].
    pub(crate) depth_increase_interval: u32,
}

impl DiffusionConfig {
    /// Share percent for the given stack count.
    ///
    /// - `stacks == 0` → `0.0` (hazard inactive short-circuit).
    /// - `stacks >= 1` → `base_share_percent + share_per_level_percent *
    ///   (stacks - 1)`, clamped to [`DIFFUSION_SHARE_CAP_PERCENT`].
    ///
    /// Not `const fn` — `f32::min` is not `const` on Bevy 0.18's MSRV.
    #[must_use]
    pub(crate) fn share_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let raw = self
            .share_per_level_percent
            .mul_add(extra, self.base_share_percent);
        if raw > DIFFUSION_SHARE_CAP_PERCENT {
            DIFFUSION_SHARE_CAP_PERCENT
        } else {
            raw
        }
    }

    /// BFS cascade depth for the given stack count.
    ///
    /// - `stacks == 0` → `0` (hazard inactive — short-circuits the BFS).
    /// - `stacks >= 1` → `1 + (stacks - 1) / depth_increase_interval.max(1)`
    ///   so a pathological `depth_increase_interval == 0` pins depth at 1
    ///   rather than dividing by zero.
    ///
    /// `const fn` — integer arithmetic only (`saturating_sub`, `max`, integer
    /// division).
    #[must_use]
    pub(crate) const fn depth(self, stacks: u32) -> u32 {
        if stacks == 0 {
            return 0;
        }
        if self.depth_increase_interval == 0 {
            // Pathological config: pin depth at 1 regardless of stack.
            return 1;
        }
        1 + stacks.saturating_sub(1) / self.depth_increase_interval
    }
}

/// Inserts [`DiffusionConfig`] from [`HazardTuning::Diffusion`], translating
/// the fractional authoring fields to percentage units (× 100). Called each
/// time the player picks Diffusion from `hazards::activate`; last write wins
/// (overwrites any prior [`DiffusionConfig`]; stack count is owned by
/// `ActiveHazards`, not the config). Warns and no-ops on a non-Diffusion
/// tuning variant, leaving any existing [`DiffusionConfig`] intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Diffusion {
        base_share_frac,
        per_level_share_frac,
        depth_every_levels,
    } = *tuning
    else {
        warn!("diffusion::activate called with non-Diffusion tuning");
        return;
    };
    commands.insert_resource(DiffusionConfig {
        base_share_percent:      base_share_frac * 100.0,
        share_per_level_percent: per_level_share_frac * 100.0,
        depth_increase_interval: depth_every_levels,
    });
}

/// No runtime system is registered by the HAZARD domain for Diffusion.
///
/// Per the design doc (`docs/design/hazards/diffusion.md`
/// §Systems and §Cross-Domain Dependencies): damage redistribution is handled
/// INSIDE `apply_damage_to_cells` in the **cells domain** by reading
/// `Option<Res<DiffusionConfig>>` + `Option<Res<ActiveHazards>>`. The hazard
/// domain provides the config resource (via [`activate`]) and nothing else at
/// runtime. This function exists to keep `hazards::register`'s fan-out
/// uniform — it is a deliberate no-op.
pub(crate) const fn register(_app: &mut App) {}
