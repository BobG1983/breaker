//! `apply_heal::<T>` — applies heal messages to HP with per-message ceiling.
//!
//! Reads `HealDealt<T>` messages and increases `Hp::current`, clamped per
//! message by `HealCap::Starting` or `HealCap::Max`. Skips entities that
//! are `Dead` or `Invulnerable`. Runs in `DmgSystems::ApplyHeal` AFTER
//! `handle_kill::<T>`.
//!
//! ## Numeric guards
//!
//! - `amount <= 0.0` (including `0.0`, negatives, `NEG_INFINITY`): skipped.
//! - `amount.is_nan()`: skipped.
//! - `hp.current >= ceiling`: skipped (over-cap short-circuit — a heal
//!   never lowers HP).

use bevy::prelude::*;

use crate::{
    components::{Dead, HealCap, Hp, Invulnerable},
    messages::HealDealt,
    traits::Dmgable,
};

type HealTargetQuery<'w, 's, T> =
    Query<'w, 's, &'static mut Hp, (With<T>, Without<Dead>, Without<Invulnerable>)>;

/// Apply each `HealDealt<T>` message to its target's `Hp`.
///
/// Skip conditions (silent — the message is simply discarded):
/// - Target carries `Dead` or `Invulnerable` (query filter).
/// - Target is missing `Hp` or the `T` marker (query miss).
/// - `msg.amount <= 0.0` (zero, negative, or `NEG_INFINITY`).
/// - `msg.amount.is_nan()`.
/// - `hp.current >= ceiling` (over-cap short-circuit).
pub(crate) fn apply_heal<T: Dmgable>(
    mut reader: MessageReader<HealDealt<T>>,
    mut targets: HealTargetQuery<T>,
) {
    for msg in reader.read() {
        if msg.amount <= 0.0 {
            continue;
        }
        if msg.amount.is_nan() {
            continue;
        }
        let Ok(mut hp) = targets.get_mut(msg.target) else {
            continue;
        };
        let ceiling = match msg.cap {
            HealCap::Starting => hp.starting,
            HealCap::Max => hp.max.unwrap_or(hp.starting),
        };
        if hp.current >= ceiling {
            continue;
        }
        hp.current = (hp.current + msg.amount).min(ceiling);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::*;
    use crate::{
        components::{Dead, HealCap, Hp, Invulnerable},
        messages::HealDealt,
    };

    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

    #[derive(Component)]
    struct OtherT;
    impl Dmgable for OtherT {}

    #[track_caller]
    fn assert_f32_eq(actual: f32, expected: f32) {
        if expected.is_infinite() {
            assert!(
                actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
                "expected {expected}, got {actual}"
            );
        } else {
            assert!(
                (actual - expected).abs() < f32::EPSILON,
                "expected {expected}, got {actual}"
            );
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<HealDealt<TestT>>();
        app.add_systems(FixedUpdate, apply_heal::<TestT>);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn enqueue(app: &mut App, msg: HealDealt<TestT>) {
        app.world_mut()
            .resource_mut::<Messages<HealDealt<TestT>>>()
            .write(msg);
    }

    fn mk_heal(target: Entity, amount: f32, cap: HealCap) -> HealDealt<TestT> {
        HealDealt::<TestT> {
            healer: None,
            target,
            amount,
            source: None,
            cap,
            _marker: PhantomData,
        }
    }

    // ── Behavior 125: heal increases Hp.current (below ceiling) ──

    #[test]
    fn heal_increases_hp_below_ceiling() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        tick(&mut app);

        let hp = app.world().get::<Hp>(e).unwrap();
        assert_f32_eq(hp.current, 8.0);
        assert_f32_eq(hp.starting, 10.0);
        assert!(hp.max.is_none());
    }

    #[test]
    fn heal_clamps_exactly_at_ceiling() {
        // Edge case 125a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  7.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Starting));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    // ── Behavior 126: multiple heals same tick accumulate ──

    #[test]
    fn two_heals_same_tick_accumulate() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 30.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 4.0, HealCap::Max));
        enqueue(&mut app, mk_heal(e, 6.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
    }

    #[test]
    fn three_heals_same_tick_clamp_at_ceiling() {
        // Edge case 126a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  27.0,
                    starting: 30.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 2.0, HealCap::Max));
        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        enqueue(&mut app, mk_heal(e, 4.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 30.0);
    }

    // ── Behavior 127: HealCap::Max with max: None → clamps at starting ──

    #[test]
    fn max_cap_with_no_max_falls_back_to_starting() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  9.5,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 5.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    #[test]
    fn max_cap_at_starting_plus_small_heal_stays_at_starting() {
        // Edge case 127a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  10.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 1.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    // ── Behavior 128: HealCap::Starting ignores max ──

    #[test]
    fn starting_cap_ignores_max() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      Some(20.0),
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 50.0, HealCap::Starting));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    // ── Behavior 129: HealCap::Max with max: Some → clamps at max ──

    #[test]
    fn max_cap_with_some_max_clamps_at_max() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      Some(20.0),
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 50.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
    }

    // ── Behavior 130: Dead marker is no-op ──

    #[test]
    fn dead_entity_is_noop() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  10.0,
                    starting: 10.0,
                    max:      None,
                },
                Dead,
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 7.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    #[test]
    fn dead_entity_with_negative_hp_and_large_heal_is_noop() {
        // Edge case 130a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  -5.0,
                    starting: 10.0,
                    max:      None,
                },
                Dead,
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 100.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -5.0);
    }

    // ── Behavior 131: Invulnerable marker is no-op ──

    #[test]
    fn invulnerable_entity_is_noop() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                Invulnerable,
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
    }

    #[test]
    fn invulnerable_with_starting_cap_is_noop() {
        // Edge case 131a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                Invulnerable,
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Starting));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
    }

    // ── Behavior 132: over-cap short-circuit (heal never lowers HP) ──

    #[test]
    fn over_cap_heal_does_not_lower_hp() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  15.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
    }

    #[test]
    fn exact_cap_heal_does_not_change_hp() {
        // Edge case 132a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  10.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    // ── Behavior 133: heal amount guards ──

    fn run_with_amount(amount: f32) -> f32 {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  7.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();
        enqueue(&mut app, mk_heal(e, amount, HealCap::Max));
        tick(&mut app);
        app.world().get::<Hp>(e).unwrap().current
    }

    #[test]
    fn amount_zero_is_noop() {
        assert_f32_eq(run_with_amount(0.0), 7.0);
    }

    #[test]
    fn amount_negative_is_noop() {
        assert_f32_eq(run_with_amount(-3.0), 7.0);
    }

    #[test]
    fn amount_negative_zero_is_noop() {
        assert_f32_eq(run_with_amount(-0.0), 7.0);
    }

    #[test]
    fn amount_nan_is_noop() {
        assert_f32_eq(run_with_amount(f32::NAN), 7.0);
    }

    #[test]
    fn amount_neg_infinity_is_noop() {
        assert_f32_eq(run_with_amount(f32::NEG_INFINITY), 7.0);
    }

    #[test]
    fn amount_infinity_max_cap_no_max_clamps_at_starting() {
        // Edge case 133a.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  7.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();
        enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Max));
        tick(&mut app);
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    #[test]
    fn amount_infinity_starting_cap_with_max_clamps_at_starting() {
        // Edge case 133b.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  7.0,
                    starting: 10.0,
                    max:      Some(20.0),
                },
            ))
            .id();
        enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Starting));
        tick(&mut app);
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    }

    #[test]
    fn amount_infinity_max_cap_with_max_clamps_at_max() {
        // Edge case 133c.
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  7.0,
                    starting: 10.0,
                    max:      Some(20.0),
                },
            ))
            .id();
        enqueue(&mut app, mk_heal(e, f32::INFINITY, HealCap::Max));
        tick(&mut app);
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
    }

    // ── Behavior 134: entity without T marker is excluded ──

    #[test]
    fn other_t_marker_is_excluded() {
        let mut app = test_app();
        let e = app.world_mut().spawn((OtherT, Hp::new(5.0))).id();

        enqueue(&mut app, mk_heal(e, 3.0, HealCap::Max));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
    }
}
