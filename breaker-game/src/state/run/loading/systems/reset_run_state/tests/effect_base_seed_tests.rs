use super::helpers::test_app;
use crate::shared::rng::EffectBaseSeed;

// ── Wave 2E Group B — reset_run_state body is preserved bit-identically ──

// Behavior 5: no-edit guard — the EffectBaseSeed derivation formula uses
// run_seed_value (the local captured value) and NOT stats.seed directly.
#[test]
fn reset_run_state_body_uses_run_seed_value_not_stats_seed_for_effect_base_seed() {
    let source = include_str!("../system.rs");
    // concat! avoids the literal appearing verbatim in this file, which
    // would inflate the count when this file includes itself.
    let expected = concat!("derive_seed_named(", "run_seed_value, \"effect\")");
    let forbidden = concat!("derive_seed_named(", "stats.seed, \"effect\")");
    assert_eq!(
        source.matches(expected).count(),
        1,
        "reset_run_state.rs must contain exactly one call that derives EffectBaseSeed \
         from run_seed_value — the existing formula must be preserved"
    );
    assert_eq!(
        source.matches(forbidden).count(),
        0,
        "reset_run_state.rs must NOT derive EffectBaseSeed from stats.seed directly — \
         the refactored variant is incorrect per the Wave 2E ordering decision"
    );
}

// Behavior 7: bit-identity pin — EffectBaseSeed for a given RunSeed is unchanged.
#[test]
fn derives_effect_base_seed_from_run_seed() {
    use crate::shared::rng::derive_seed_named;

    // Seeded run: RunSeed(Some(42)) → EffectBaseSeed == derive_seed_named(42, "effect")
    {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(crate::shared::RunSeed(Some(42)));
        app.update();

        let expected = EffectBaseSeed(derive_seed_named(42, "effect"));
        let actual = *app.world().resource::<EffectBaseSeed>();
        assert_eq!(
            actual.0, expected.0,
            "EffectBaseSeed must equal derive_seed_named(42, \"effect\") for RunSeed(Some(42))"
        );
    }

    // Edge case: RunSeed(Some(0)) — derive_seed_named produces a non-zero u64 per Wave 1
    {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(crate::shared::RunSeed(Some(0)));
        app.update();

        let expected = EffectBaseSeed(derive_seed_named(0, "effect"));
        let actual = *app.world().resource::<EffectBaseSeed>();
        assert_eq!(
            actual.0, expected.0,
            "EffectBaseSeed must equal derive_seed_named(0, \"effect\") for RunSeed(Some(0))"
        );
        assert_ne!(
            actual.0, 0,
            "derive_seed_named(0, \"effect\") must be non-zero per Wave 1 hash contract"
        );
    }
}
