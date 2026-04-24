//! W2 Behavior 38: Tether kill attribution forwards to original dealer via
//! `attributed_to.or(dealer)` at the `apply_damage` insertion site.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::{TetherLink, tether_emit_partner},
    helpers::{add_tether_stacks, canonical_tether_config, install_tether_config},
};
use crate::{
    hazard::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

fn build_tether_app() -> App {
    // `with_effects_pipeline()` wires `register_dmgable::<Cell>()`, which in
    // turn schedules `handle_kill::<Cell>` → `Destroyed<Cell>` emit → then
    // `process_despawn_requests` in `FixedPostUpdate`. The victim is
    // despawned within the same tick it dies, so `KilledBy`/`Dead` on the
    // entity are not observable after the tick returns. Capture
    // `Destroyed<Cell>` instead — it carries `killer` at the moment the
    // kill is confirmed.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_message_capture::<Destroyed<Cell>>()
        .with_resource::<ActiveHazards>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    app.add_systems(
        FixedUpdate,
        tether_emit_partner
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

#[test]
fn tether_kill_attribution_forwards_to_bolt() {
    let mut app = build_tether_app();
    let a = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(1000.0),
            KilledBy { killer: None },
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(30.0, 0.0)),
            Hp::new(20.0),
            KilledBy { killer: None },
        ))
        .id();
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });
    let bolt = app.world_mut().spawn_empty().id();

    // Frame N: primary on A, 100 damage.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // Frame N — primary applies to A; tether emits sibling to B.
    tick(&mut app); // Frame N+1 — sibling goes through full pipeline; B dies & despawns.

    assert!(
        app.world().get_entity(b).is_err(),
        "B must be despawned after receiving 25.0 partner damage (kills it)"
    );
    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Cell>>>()
        .0
        .iter()
        .find(|m| m.victim == b)
        .expect("Destroyed<Cell> for B must have been emitted by handle_kill");
    assert_eq!(
        destroyed.killer,
        Some(bolt),
        "kill attribution must forward to original bolt via attributed_to.or(dealer)"
    );
}

#[test]
fn tether_kill_none_when_both_dealer_and_attributed_to_none() {
    // Environmental primary → sibling has both fields None → killer = None.
    let mut app = build_tether_app();
    let a = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(1000.0),
            KilledBy { killer: None },
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(30.0, 0.0)),
            Hp::new(20.0),
            KilledBy { killer: None },
        ))
        .id();
    app.world_mut()
        .entity_mut(a)
        .insert(TetherLink { partner: b });
    app.world_mut()
        .entity_mut(b)
        .insert(TetherLink { partner: a });

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(b).is_err(),
        "B must be despawned after receiving partner damage"
    );
    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Cell>>>()
        .0
        .iter()
        .find(|m| m.victim == b)
        .expect("Destroyed<Cell> for B must have been emitted");
    assert!(
        destroyed.killer.is_none(),
        "environmental primary → sibling attribution None → killer None"
    );
}
