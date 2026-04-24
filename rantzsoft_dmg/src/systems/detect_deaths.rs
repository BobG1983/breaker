//! `detect_deaths::<T>` — emits `KillYourself<T>` for HP-zero entities.
//!
//! Scans every non-`Dead` `T` entity; any whose `Hp::current <= 0.0`
//! produces one `KillYourself<T>` with `victim = entity` and
//! `killer = killed_by.and_then(|k| k.killer)` (flattening optional
//! `KilledBy`). Runs in `DmgSystems::EmitKill`, strictly after
//! `apply_damage::<T>`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    components::{Dead, Hp, KilledBy},
    messages::KillYourself,
    traits::Dmgable,
};

type DeathQuery<'w, 's, T> =
    Query<'w, 's, (Entity, Option<&'static KilledBy>, &'static Hp), (With<T>, Without<Dead>)>;

/// Emit `KillYourself<T>` for every non-`Dead` `T` entity whose
/// `Hp::current` is `<= 0.0`. `killer` is derived from the (optional)
/// `KilledBy` component — `Some(entity) -> Some(entity.killer)`
/// flattened, else `None`.
pub(crate) fn detect_deaths<T: Dmgable>(
    query: DeathQuery<T>,
    mut writer: MessageWriter<KillYourself<T>>,
) {
    for (entity, killed_by, hp) in &query {
        if hp.current <= 0.0 {
            writer.write(KillYourself {
                victim:  entity,
                killer:  killed_by.and_then(|k| k.killer),
                _marker: PhantomData,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::*;
    use crate::{
        components::{Dead, Hp, KilledBy},
        messages::KillYourself,
    };

    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

    #[derive(Component)]
    struct OtherT;
    impl Dmgable for OtherT {}

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<KillYourself<TestT>>();
        app.add_systems(FixedUpdate, detect_deaths::<TestT>);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn drain_kills(app: &mut App) -> Vec<KillYourself<TestT>> {
        app.world_mut()
            .resource_mut::<Messages<KillYourself<TestT>>>()
            .drain()
            .collect()
    }

    // ── Behavior 104: HP=0 + KilledBy{Some(e)} → emits with killer=Some(e) ──

    #[test]
    fn hp_zero_with_killed_by_emits_with_killer() {
        let mut app = test_app();
        let dealer = app.world_mut().spawn_empty().id();
        let victim = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  0.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy {
                    killer: Some(dealer),
                },
            ))
            .id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].victim, victim);
        assert_eq!(drained[0].killer, Some(dealer));
    }

    #[test]
    fn hp_negative_with_killed_by_still_emits() {
        // Edge case 104a.
        let mut app = test_app();
        let dealer = app.world_mut().spawn_empty().id();
        let victim = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  -5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy {
                    killer: Some(dealer),
                },
            ))
            .id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].victim, victim);
        assert_eq!(drained[0].killer, Some(dealer));
    }

    // ── Behavior 105: HP>0 does NOT emit ──

    #[test]
    fn hp_positive_does_not_emit() {
        let mut app = test_app();
        app.world_mut()
            .spawn((TestT, Hp::new(10.0), KilledBy { killer: None }));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert!(drained.is_empty());
    }

    #[test]
    fn tiny_positive_hp_does_not_emit() {
        // Edge case 105a: current 0.0001 is still positive.
        let mut app = test_app();
        app.world_mut().spawn((
            TestT,
            Hp {
                current:  0.0001,
                starting: 10.0,
                max:      None,
            },
        ));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert!(drained.is_empty());
    }

    // ── Behavior 106: Dead marker is skipped ──

    #[test]
    fn dead_marker_skipped_at_zero_hp() {
        let mut app = test_app();
        app.world_mut().spawn((TestT, Hp::new(0.0), Dead));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert!(drained.is_empty());
    }

    #[test]
    fn dead_marker_skipped_at_negative_hp() {
        // Edge case 106a.
        let mut app = test_app();
        app.world_mut().spawn((
            TestT,
            Hp {
                current:  -5.0,
                starting: 10.0,
                max:      None,
            },
            Dead,
        ));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert!(drained.is_empty());
    }

    // ── Behavior 107: HP<=0 with NO KilledBy emits killer=None ──

    #[test]
    fn zero_hp_without_killed_by_emits_none_killer() {
        let mut app = test_app();
        let victim = app.world_mut().spawn((TestT, Hp::new(0.0))).id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].victim, victim);
        assert!(drained[0].killer.is_none());
    }

    #[test]
    fn negative_hp_without_killed_by_emits_none_killer() {
        // Edge case 107a.
        let mut app = test_app();
        let victim = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  -10.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].victim, victim);
        assert!(drained[0].killer.is_none());
    }

    // ── Behavior 108: KilledBy{killer: None} → killer: None ──

    #[test]
    fn killed_by_none_dealer_emits_none_killer() {
        let mut app = test_app();
        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(0.0), KilledBy { killer: None }))
            .id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].victim, victim);
        assert!(drained[0].killer.is_none());
    }

    // ── Behavior 109: multiple dead entities ──

    #[test]
    fn three_dead_entities_emit_three_kills() {
        let mut app = test_app();
        let dealer = app.world_mut().spawn_empty().id();
        let v1 = app
            .world_mut()
            .spawn((TestT, Hp::new(0.0), KilledBy { killer: None }))
            .id();
        let v2 = app.world_mut().spawn((TestT, Hp::new(0.0))).id();
        let v3 = app
            .world_mut()
            .spawn((
                TestT,
                Hp::new(0.0),
                KilledBy {
                    killer: Some(dealer),
                },
            ))
            .id();

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 3);

        let find_killer = |victim: Entity| -> Option<Entity> {
            drained
                .iter()
                .find(|k| k.victim == victim)
                .and_then(|k| k.killer)
        };
        assert!(find_killer(v1).is_none());
        assert!(find_killer(v2).is_none());
        assert_eq!(find_killer(v3), Some(dealer));
    }

    #[test]
    fn three_dead_one_alive_emits_three_kills_only() {
        // Edge case 109a.
        let mut app = test_app();
        app.world_mut().spawn((TestT, Hp::new(0.0)));
        app.world_mut().spawn((TestT, Hp::new(0.0)));
        app.world_mut().spawn((TestT, Hp::new(0.0)));
        app.world_mut().spawn((TestT, Hp::new(5.0)));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert_eq!(drained.len(), 3);
    }

    // ── Behavior 110: entity without T marker excluded ──

    #[test]
    fn other_t_marker_is_excluded() {
        let mut app = test_app();
        app.world_mut().spawn((OtherT, Hp::new(0.0)));

        tick(&mut app);

        let drained = drain_kills(&mut app);
        assert!(drained.is_empty());
    }

    // Keep PhantomData import exercised so the use line is load-bearing even
    // if a future test is removed.
    #[test]
    fn phantom_data_marker_imports_compile() {
        let _: PhantomData<TestT> = PhantomData;
    }
}
