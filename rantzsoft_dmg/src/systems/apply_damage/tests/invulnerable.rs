use super::helpers::{TestT, assert_f32_eq, enqueue, mk_msg, test_app, tick};
use crate::components::{Hp, Invulnerable, KilledBy};

// ── Behavior 153: apply_damage's query does NOT filter Without<Invulnerable>.
//     Tested in isolation (no invulnerable_filter upstream) ──

#[test]
fn apply_damage_applies_to_invulnerable_when_filter_bypassed() {
    // Bare app, no plugin, no register_dmgable — only apply_damage::<TestT>.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Invulnerable))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 3.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 7.0);
    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn apply_damage_inserts_killed_by_on_invulnerable_when_filter_bypassed() {
    // Edge case 153a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Invulnerable))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 5.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 0.0);
    let killed_by = app.world().get::<KilledBy>(victim).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
}
