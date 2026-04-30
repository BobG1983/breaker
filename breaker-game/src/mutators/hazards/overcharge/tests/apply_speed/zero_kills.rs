use super::super::helpers::{
    add_overcharge_stacks, canonical_config, install_overcharge_config, run_fixed_update,
    spawn_bolt, spawn_bolt_with_count, test_app_playing, wire_apply_only,
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 22 — zero kills / no pre-existing stack → no entry ─────────

#[test]
fn apply_speed_with_zero_kills_inserts_no_entry() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "bolts with zero kills should not receive an Overcharge entry"
    );
}

#[test]
fn apply_speed_with_explicit_zero_kill_component_still_inserts_no_entry() {
    // Edge: bolt HAS OverchargeKillCount(0). Still no stack inserted.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 0);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}
