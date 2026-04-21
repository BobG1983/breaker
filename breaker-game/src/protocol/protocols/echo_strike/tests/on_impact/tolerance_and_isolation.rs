use super::{
    super::{
        super::system::EchoPrimed,
        helpers::{
            build_echo_strike_app, collected_echo_strike_damage, find_amount_for,
            read_echo_network, spawn_bolt_primed_with_network, spawn_bolt_with_echo_network,
            spawn_cell_empty, write_bolt_impact_cell,
        },
    },
    helpers::seed_canonical,
};
use crate::prelude::*;

// ── Behavior 27 — impact for despawned bolt is tolerated ───────────────────-

#[test]
fn impact_for_despawned_bolt_is_tolerated() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![a]);
    app.world_mut().entity_mut(bolt).despawn();
    let b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, b, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_echo_strike_damage(&app).is_empty(),
        "no echo damage for a despawned bolt (network gone)"
    );
}

// ── Behavior 28 — multi-bolt isolation ─────────────────────────────────────-

#[test]
fn primed_impact_does_not_touch_other_bolts_networks() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_p = spawn_cell_empty(&mut app);
    let cell_q = spawn_cell_empty(&mut app);
    let primed_bolt = spawn_bolt_primed_with_network(&mut app, 10.0, vec![cell_a]);
    let other_bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_p, cell_q]);
    let cell_b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_b, primed_bolt);
    tick(&mut app);

    assert_eq!(
        read_echo_network(&app, primed_bolt),
        vec![cell_a, cell_b],
        "primed_bolt's network should have A + B"
    );
    assert_eq!(
        read_echo_network(&app, other_bolt),
        vec![cell_p, cell_q],
        "other_bolt's network must be untouched"
    );

    let msgs = collected_echo_strike_damage(&app);
    assert_eq!(msgs.len(), 1, "exactly one echo-damage message");
    assert_eq!(
        msgs[0].target, cell_a,
        "target should be A (primed_bolt's sole echo)"
    );
    assert_eq!(
        msgs[0].dealer,
        Some(primed_bolt),
        "dealer should be primed_bolt"
    );
    assert!(
        find_amount_for(&msgs, cell_p).is_none() && find_amount_for(&msgs, cell_q).is_none(),
        "no echo-damage should target P or Q (other_bolt's echoes)"
    );

    assert!(
        app.world().get::<EchoPrimed>(primed_bolt).is_none(),
        "primed_bolt no longer primed"
    );
    assert!(
        app.world().get::<EchoPrimed>(other_bolt).is_none(),
        "other_bolt still not primed"
    );
}
