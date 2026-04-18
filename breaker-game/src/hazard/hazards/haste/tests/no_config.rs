//! Group F — No-config guard.
//!
//! When `HasteConfig` is absent, `haste_apply_speed` returns early and
//! does not touch any bolt's `EffectStack`.

use ordered_float::OrderedFloat;

use super::helpers::{
    add_haste_stacks, run_fixed_update, spawn_bolt, spawn_bolt_with_stack, test_app_playing,
    wire_apply_only,
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 24 — no-op when HasteConfig resource is absent ────────────

#[test]
fn no_apply_when_config_absent() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    // Intentionally no `install_haste_config` call.
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "no stack should be inserted without HasteConfig"
    );
}

#[test]
fn pre_existing_chip_entry_untouched_when_config_absent() {
    // Edge: a bolt spawned with a seeded chip:overclock entry is
    // neither extended nor perturbed when HasteConfig is absent — the
    // early return fires BEFORE the per-entity loop.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "pre-existing stack must not be extended");
    assert!((stack.aggregate() - 1.5).abs() < 1e-5);
}
