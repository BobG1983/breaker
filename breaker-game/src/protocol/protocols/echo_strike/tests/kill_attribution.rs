//! W2 Behavior 45: Echo Strike kill attribution forwards to original bolt.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::echo_strike_emit_siblings,
    helpers::{
        canonical_echo_strike_config, install_echo_strike_config,
        seed_active_protocols_with_echo_strike, spawn_bolt_primed_with_network,
    },
};
use crate::{
    cells::components::Cell,
    prelude::*,
    protocol::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, protocol_active},
    },
};

fn build_app() -> App {
    // `with_effects_pipeline()` wires the crate kill → despawn pipeline, so
    // a fatally-hit cell is despawned before the tick returns. Capture
    // `Destroyed<Cell>` to observe the `killer` field at kill time.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_message_capture::<Destroyed<Cell>>()
        .with_resource::<ActiveProtocols>()
        .build();
    install_echo_strike_config(&mut app, canonical_echo_strike_config());
    seed_active_protocols_with_echo_strike(&mut app, 3, 0.5, 0.25, 0.1);
    app.add_systems(
        FixedUpdate,
        echo_strike_emit_siblings
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(protocol_active(ProtocolKind::EchoStrike))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

#[test]
fn echo_kill_attribution_forwards_to_bolt() {
    let mut app = build_app();
    let c_newest = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(20.0),
            KilledBy { killer: None },
        ))
        .id();
    let bolt = spawn_bolt_primed_with_network(&mut app, 100.0, vec![c_newest]);
    let c_primary = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(100.0, 0.0)),
            Hp::new(1000.0),
            KilledBy { killer: None },
        ))
        .id();

    // Frame N: primary on c_primary.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c_primary,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // N
    tick(&mut app); // N+1: echo sibling on c_newest kills it

    assert!(
        app.world().get_entity(c_newest).is_err(),
        "c_newest must be despawned after the echo sibling kills it"
    );
    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Cell>>>()
        .0
        .iter()
        .find(|m| m.victim == c_newest)
        .expect("Destroyed<Cell> for c_newest must have been emitted");
    assert_eq!(
        destroyed.killer,
        Some(bolt),
        "kill must attribute to the original bolt via attributed_to"
    );
}
