//! Diffusion hazard — stateless damage-sharing redistribution.
//!
//! Per the design doc at `docs/todos/detail/mod-system-design/hazards/diffusion.md`,
//! redistribution lives INSIDE the cells-domain system `apply_damage_to_cells`.
//! The hazard domain owns only the per-run `DiffusionConfig` resource populated
//! by [`activate`] and a no-op [`register`] — there is no hazard-domain runtime
//! system for the BFS / HP math.

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
/// Per the design doc (`docs/todos/detail/mod-system-design/hazards/diffusion.md`
/// §Systems and §Cross-Domain Dependencies): damage redistribution is handled
/// INSIDE `apply_damage_to_cells` in the **cells domain** by reading
/// `Option<Res<DiffusionConfig>>` + `Option<Res<ActiveHazards>>`. The hazard
/// domain provides the config resource (via [`activate`]) and nothing else at
/// runtime. This function exists to keep `hazards::register`'s fan-out
/// uniform — it is a deliberate no-op.
pub(crate) const fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use bevy::{ecs::world::CommandQueue, prelude::*};

    use super::*;
    use crate::{
        hazard::{
            definition::{HazardDefinition, HazardKind, HazardTuning},
            resources::ActiveHazards,
        },
        prelude::*,
    };

    // ── Helpers (Fracture 5-helper pattern) ──────────────────────────────

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<DamageDealt<Cell>>()
            .build()
    }

    const fn canonical_config() -> DiffusionConfig {
        DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        }
    }

    fn add_diffusion_stacks(app: &mut App, count: u32) {
        let mut active = app.world_mut().resource_mut::<ActiveHazards>();
        for _ in 0..count {
            active.add_stack(HazardKind::Diffusion);
        }
    }

    fn activate_now(app: &mut App, tuning: &HazardTuning) {
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, app.world());
            activate(tuning, &mut commands);
        }
        queue.apply(app.world_mut());
    }

    // ════════════════════════════════════════════════════════════════════
    // Section A — DiffusionConfig pure formulas
    // ════════════════════════════════════════════════════════════════════

    // Behavior 1 — share_percent(0) == 0.0 (inactive short-circuit).
    #[test]
    fn share_percent_zero_stacks_is_zero() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        };
        assert!((cfg.share_percent(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn share_percent_zero_stacks_short_circuits_before_cap() {
        let cfg = DiffusionConfig {
            base_share_percent:      95.0,
            share_per_level_percent: 0.0,
            depth_increase_interval: 5,
        };
        // Pins that stacks == 0 returns 0.0 BEFORE cap logic applies.
        assert!((cfg.share_percent(0) - 0.0).abs() < f32::EPSILON);
    }

    // Behavior 2 — share_percent(1) == base_share_percent.
    #[test]
    fn share_percent_stack_one_is_base() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        };
        assert!((cfg.share_percent(1) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn share_percent_stack_one_ignores_per_level_multiplier() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 999.0,
            depth_increase_interval: 5,
        };
        // per_level must not leak into stack 1 (pins extra = stacks - 1 = 0).
        assert!((cfg.share_percent(1) - 20.0).abs() < f32::EPSILON);
    }

    // Behavior 3 — share_percent(3) == base + 2 * per_level.
    #[test]
    fn share_percent_stack_three_adds_two_levels() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 5,
        };
        // 20 + 2 * 10 = 40.
        assert!((cfg.share_percent(3) - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn share_percent_stack_three_with_zero_base_still_grows() {
        let cfg = DiffusionConfig {
            base_share_percent:      0.0,
            share_per_level_percent: 20.0,
            depth_increase_interval: 5,
        };
        assert!((cfg.share_percent(3) - 40.0).abs() < f32::EPSILON);
    }

    // Behavior 4 — share_percent(5) == 60.0 (design doc example).
    #[test]
    fn share_percent_stack_five_is_sixty() {
        let cfg = canonical_config();
        assert!((cfg.share_percent(5) - 60.0).abs() < f32::EPSILON);
    }

    // Behavior 5 — share_percent(6) == 70.0 (design doc example, also depth=2).
    #[test]
    fn share_percent_stack_six_is_seventy() {
        let cfg = canonical_config();
        assert!((cfg.share_percent(6) - 70.0).abs() < f32::EPSILON);
    }

    // Behavior 6 — share_percent capped at DIFFUSION_SHARE_CAP_PERCENT (95.0).
    #[test]
    fn share_percent_stack_nine_is_capped_at_ninety_five() {
        let cfg = canonical_config();
        // Raw would be 20 + 8 * 10 = 100.0, capped to 95.0.
        assert!((cfg.share_percent(9) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
    }

    #[test]
    fn share_percent_cap_engages_at_large_stack_counts() {
        let cfg = canonical_config();
        // Raw would be 20 + 99 * 10 = 1010.0, capped to 95.0.
        assert!((cfg.share_percent(100) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
    }

    // Behavior 7 — cap applies to crushing base configs too.
    #[test]
    fn share_percent_cap_applies_post_sum_to_large_base() {
        let cfg = DiffusionConfig {
            base_share_percent:      99.0,
            share_per_level_percent: 0.0,
            depth_increase_interval: 5,
        };
        // 99.0 > 95.0; result must be the cap.
        assert!((cfg.share_percent(1) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
    }

    // Behavior 8 — inclusive-boundary: exactly 95.0 returns 95.0 unchanged.
    #[test]
    fn share_percent_exactly_at_cap_returns_cap() {
        let cfg = DiffusionConfig {
            base_share_percent:      95.0,
            share_per_level_percent: 0.0,
            depth_increase_interval: 5,
        };
        assert!((cfg.share_percent(1) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
    }

    // Behavior 9 — depth(0) == 0 (inactive short-circuit).
    #[test]
    fn depth_zero_stacks_is_zero() {
        let cfg = canonical_config();
        assert_eq!(cfg.depth(0), 0);
    }

    // Behavior 10 — depth(1..=5) with interval 5 == 1 for every stack in range.
    #[test]
    fn depth_stack_one_through_five_is_one() {
        let cfg = canonical_config();
        for k in 1..=5u32 {
            assert_eq!(cfg.depth(k), 1, "stack {k} should have depth 1");
        }
    }

    #[test]
    fn depth_stack_five_boundary_is_one_not_two() {
        // Pins integer-division boundary: 1 + (5 - 1) / 5 == 1 + 0 == 1.
        let cfg = canonical_config();
        assert_eq!(cfg.depth(5), 1);
    }

    // Behavior 11 — depth(6..=10) with interval 5 == 2.
    #[test]
    fn depth_stack_six_is_two() {
        let cfg = canonical_config();
        assert_eq!(cfg.depth(6), 2);
    }

    #[test]
    fn depth_stack_ten_is_two() {
        let cfg = canonical_config();
        assert_eq!(cfg.depth(10), 2);
    }

    // Behavior 12 — depth(11) with interval 5 == 3.
    #[test]
    fn depth_stack_eleven_is_three() {
        let cfg = canonical_config();
        assert_eq!(cfg.depth(11), 3);
    }

    // Behavior 13 — depth(u32::MAX) saturates and does not panic.
    #[test]
    fn depth_u32_max_does_not_panic_and_is_at_least_one() {
        let cfg = canonical_config();
        let d = cfg.depth(u32::MAX);
        assert!(d >= 1, "depth(u32::MAX) should be >= 1, got {d}");
    }

    // Behavior 14 — depth with depth_increase_interval = 0 returns 1 regardless.
    #[test]
    fn depth_with_zero_interval_returns_one_at_stack_one() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 0,
        };
        assert_eq!(cfg.depth(1), 1);
    }

    #[test]
    fn depth_with_zero_interval_returns_one_at_stack_one_hundred() {
        let cfg = DiffusionConfig {
            base_share_percent:      20.0,
            share_per_level_percent: 10.0,
            depth_increase_interval: 0,
        };
        assert_eq!(cfg.depth(100), 1);
    }

    // ════════════════════════════════════════════════════════════════════
    // Section B — activate lifecycle (frac → percent translation)
    // ════════════════════════════════════════════════════════════════════

    // Behavior 15 — activate with matching tuning inserts translated config.
    #[test]
    fn activate_inserts_config_with_percent_translated_fields() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Diffusion {
                base_share_frac:      0.20,
                per_level_share_frac: 0.10,
                depth_every_levels:   5,
            },
        );

        let cfg = app.world().resource::<DiffusionConfig>();
        assert!(
            (cfg.base_share_percent - 20.0).abs() < f32::EPSILON,
            "base_share_percent expected 20.0, got {}",
            cfg.base_share_percent
        );
        assert!(
            (cfg.share_per_level_percent - 10.0).abs() < f32::EPSILON,
            "share_per_level_percent expected 10.0, got {}",
            cfg.share_per_level_percent
        );
        assert_eq!(cfg.depth_increase_interval, 5);
    }

    // Behavior 16 — non-trivial fraction translations are preserved within epsilon.
    #[test]
    fn activate_translates_nontrivial_fractions() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Diffusion {
                base_share_frac:      0.275,
                per_level_share_frac: 0.125,
                depth_every_levels:   7,
            },
        );

        let cfg = app.world().resource::<DiffusionConfig>();
        assert!(
            (cfg.base_share_percent - 27.5).abs() < f32::EPSILON,
            "base_share_percent expected 27.5, got {}",
            cfg.base_share_percent
        );
        assert!(
            (cfg.share_per_level_percent - 12.5).abs() < f32::EPSILON,
            "share_per_level_percent expected 12.5, got {}",
            cfg.share_per_level_percent
        );
        assert_eq!(cfg.depth_increase_interval, 7);
    }

    // Behavior 17 — Decay tuning does nothing.
    #[test]
    fn activate_with_decay_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
        );
        assert!(app.world().get_resource::<DiffusionConfig>().is_none());
    }

    // Behavior 18 — Sympathy tuning (most-similar field shape) does nothing.
    #[test]
    fn activate_with_sympathy_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Sympathy {
                base_heal_frac:      0.25,
                per_level_heal_frac: 0.05,
                depth_every_levels:  5,
            },
        );
        assert!(app.world().get_resource::<DiffusionConfig>().is_none());
    }

    // Behavior 19 — second activate with matching tuning overwrites (last-write-wins).
    #[test]
    fn second_activate_overwrites_prior_diffusion_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Diffusion {
                base_share_frac:      0.20,
                per_level_share_frac: 0.10,
                depth_every_levels:   5,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Diffusion {
                base_share_frac:      0.50,
                per_level_share_frac: 0.25,
                depth_every_levels:   3,
            },
        );

        let cfg = app.world().resource::<DiffusionConfig>();
        assert!((cfg.base_share_percent - 50.0).abs() < f32::EPSILON);
        assert!((cfg.share_per_level_percent - 25.0).abs() < f32::EPSILON);
        assert_eq!(cfg.depth_increase_interval, 3);
    }

    // Behavior 20 — mismatch after successful activate preserves existing config.
    #[test]
    fn activate_mismatch_after_match_preserves_existing_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Diffusion {
                base_share_frac:      0.20,
                per_level_share_frac: 0.10,
                depth_every_levels:   5,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
        );

        let cfg = app.world().resource::<DiffusionConfig>();
        assert!((cfg.base_share_percent - 20.0).abs() < f32::EPSILON);
        assert!((cfg.share_per_level_percent - 10.0).abs() < f32::EPSILON);
        assert_eq!(cfg.depth_increase_interval, 5);
    }

    // ════════════════════════════════════════════════════════════════════
    // Section C — register (no-op / inert)
    // ════════════════════════════════════════════════════════════════════

    // Behavior 21 — register does NOT emit any DamageDealt<Cell> messages.
    #[test]
    fn register_does_not_emit_damage_dealt_cell_messages() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message_capture::<DamageDealt<Cell>>()
            .build();

        // Install config + 1 Diffusion stack + two cells.
        app.world_mut().insert_resource(canonical_config());
        add_diffusion_stacks(&mut app, 1);
        app.world_mut().spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy::default(),
        ));
        app.world_mut().spawn((
            Cell,
            Position2D(Vec2::new(50.0, 0.0)),
            Hp::new(100.0),
            KilledBy::default(),
        ));

        register(&mut app);
        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            collector.0.len(),
            0,
            "register must not schedule any system that writes DamageDealt<Cell>, got {}",
            collector.0.len()
        );
    }

    // Behavior 22 — register does not panic when DiffusionConfig is absent.
    #[test]
    fn register_does_not_panic_without_diffusion_config() {
        let mut app = test_app_playing();
        register(&mut app);
        tick(&mut app);
    }

    // Behavior 23 — register does not panic when ActiveHazards is absent.
    #[test]
    fn register_does_not_panic_without_active_hazards() {
        // Deliberately omit .with_resource::<ActiveHazards>(). If register wired
        // a system gated on `hazard_active(...)` (which takes a non-optional
        // Res<ActiveHazards>), Bevy 0.18 would panic during schedule execution.
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_message::<DamageDealt<Cell>>()
            .build();

        register(&mut app);
        tick(&mut app);
    }

    // ════════════════════════════════════════════════════════════════════
    // Section E — RON asset shape
    // ════════════════════════════════════════════════════════════════════

    // Behavior 47 — diffusion.hazard.ron parses to a HazardDefinition.
    #[test]
    fn diffusion_ron_asset_deserializes_to_hazard_definition() {
        let ron_str = include_str!("../../../assets/hazards/diffusion.hazard.ron");
        let def: HazardDefinition =
            ron::de::from_str(ron_str).expect("diffusion.hazard.ron should parse");

        assert_eq!(def.kind(), HazardKind::Diffusion);

        // Field accessors live inside the definition module; match the tuning
        // variant to extract fractional values and compare.
        let HazardTuning::Diffusion {
            base_share_frac,
            per_level_share_frac,
            depth_every_levels,
        } = match_definition_tuning_for_test(&def)
        else {
            panic!("expected HazardTuning::Diffusion");
        };
        assert!((base_share_frac - 0.2).abs() < f32::EPSILON);
        assert!((per_level_share_frac - 0.1).abs() < f32::EPSILON);
        assert_eq!(depth_every_levels, 5);
    }

    // Behavior 48 — description matches the design-doc drift guard.
    #[test]
    fn diffusion_ron_description_matches_design_doc_drift_guard() {
        let ron_str = include_str!("../../../assets/hazards/diffusion.hazard.ron");
        let def: HazardDefinition =
            ron::de::from_str(ron_str).expect("diffusion.hazard.ron should parse");
        assert_eq!(
            name_of(&def),
            "Diffusion",
            "hazard name should be 'Diffusion'"
        );
        assert_eq!(
            description_of(&def),
            "Damage spreads to neighboring cells, but each hit leaves the target less damaged."
        );
        assert_eq!(unlock_tier_of(&def), 0);
    }

    // `HazardDefinition`'s `name`/`description`/`tuning`/`unlock_tier` are
    // `pub(crate)` — reachable from this test module because we are inside the
    // same crate. Access them through thin helpers kept here so the test
    // bodies read as data-oriented assertions.
    fn name_of(def: &HazardDefinition) -> &str {
        &def.name
    }

    fn description_of(def: &HazardDefinition) -> &str {
        &def.description
    }

    fn unlock_tier_of(def: &HazardDefinition) -> u32 {
        def.unlock_tier
    }

    fn match_definition_tuning_for_test(def: &HazardDefinition) -> HazardTuning {
        def.tuning.clone()
    }
}
