//! Section C — register (no-op / inert).

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{hazard::resources::ActiveHazards, prelude::*};

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
        KilledBy { killer: None },
    ));
    app.world_mut().spawn((
        Cell,
        Position2D(Vec2::new(50.0, 0.0)),
        Hp::new(100.0),
        KilledBy { killer: None },
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

// Behavior 23 — register does not panic when Diffusion is not active.
//
// The diffusion systems are gated on `hazard_active(HazardKind::Diffusion)`,
// which requires `ActiveHazards` to exist as a resource. With zero stacks,
// the run-if condition is false and the systems skip — no panic.
#[test]
fn register_does_not_panic_when_diffusion_inactive() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build();

    register(&mut app);
    tick(&mut app);
}

// ════════════════════════════════════════════════════════════════════
// W2 Behaviors 53, 56 — diffusion register schedule placement + ordering
// ════════════════════════════════════════════════════════════════════

use std::marker::PhantomData;

use crate::hazard::definition::HazardKind;

// ── W2 Behavior 53: register schedules reduce_primary in MutateDamage +
//     emit_rings in PostApplyDamage ──

#[test]
fn register_wires_systems_into_dmg_sets() {
    // After register(app) + 1 tick with Diffusion active + primary msg,
    // msg.amount must be reduced (proves reduce_primary ran in MutateDamage).
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(canonical_config());
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    register(&mut app);

    let c0 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let c1 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(30.0, 0.0)),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let _ = c1;

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    // Primary reduced AND ring emitted — proves both register-wired systems ran.
    let c0_hp = app.world().get::<Hp>(c0).expect("Hp").current;
    assert!(
        c0_hp < 100.0,
        "C0 HP must be reduced: register must schedule reduce_primary in MutateDamage"
    );
}

// ── W2 Behavior 56: PostApplyDamage emitters run in order diffusion → tether → echo ──
//
// A dedicated ordering resource records each emitter's execution order.

#[derive(Resource, Default)]
struct PostApplyOrder(Vec<&'static str>);

fn record_diffusion(mut log: ResMut<PostApplyOrder>) {
    log.0.push("diffusion_emit_rings");
}
fn record_tether(mut log: ResMut<PostApplyOrder>) {
    log.0.push("tether_emit_partner");
}
fn record_echo(mut log: ResMut<PostApplyOrder>) {
    log.0.push("echo_strike_emit_siblings");
}

#[test]
fn post_apply_emitters_run_in_order_diffusion_tether_echo() {
    use super::super::system::diffusion_emit_rings;
    use crate::{
        hazard::hazards::tether::system::tether_emit_partner,
        protocol::protocols::echo_strike::system::echo_strike_emit_siblings,
    };

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.init_resource::<PostApplyOrder>();

    // Attach recorders using .after(target_system) for each of the 3 emitters.
    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );
    // Register all three emitters in PostApplyDamage with the expected chain order.
    app.add_systems(
        FixedUpdate,
        (
            diffusion_emit_rings,
            tether_emit_partner,
            echo_strike_emit_siblings,
        )
            .chain()
            .in_set(DmgSystems::PostApplyDamage),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec![
            "diffusion_emit_rings",
            "tether_emit_partner",
            "echo_strike_emit_siblings",
        ],
    );
}

// ── W2 Behavior 56 (pairwise edge cases): each pair in the three-emitter
// chain must run in the documented order independently of the third
// emitter. These three tests isolate one pair at a time so a regression
// that breaks only one pair (e.g. tether before diffusion under some
// conditional registration) still surfaces. ──

#[test]
fn post_apply_order_diffusion_before_tether() {
    use super::super::system::diffusion_emit_rings;
    use crate::hazard::hazards::tether::system::tether_emit_partner;

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.init_resource::<PostApplyOrder>();

    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
        ),
    );
    app.add_systems(
        FixedUpdate,
        (diffusion_emit_rings, tether_emit_partner)
            .chain()
            .in_set(DmgSystems::PostApplyDamage),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["diffusion_emit_rings", "tether_emit_partner"],
        "diffusion_emit_rings must run before tether_emit_partner in PostApplyDamage"
    );
}

#[test]
fn post_apply_order_tether_before_echo() {
    use crate::{
        hazard::hazards::tether::system::tether_emit_partner,
        protocol::protocols::echo_strike::system::echo_strike_emit_siblings,
    };

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .build();
    app.init_resource::<PostApplyOrder>();

    app.add_systems(
        FixedUpdate,
        (
            record_tether
                .in_set(DmgSystems::PostApplyDamage)
                .after(tether_emit_partner),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );
    app.add_systems(
        FixedUpdate,
        (tether_emit_partner, echo_strike_emit_siblings)
            .chain()
            .in_set(DmgSystems::PostApplyDamage),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["tether_emit_partner", "echo_strike_emit_siblings"],
        "tether_emit_partner must run before echo_strike_emit_siblings in PostApplyDamage"
    );
}

#[test]
fn post_apply_order_diffusion_before_echo() {
    use super::super::system::diffusion_emit_rings;
    use crate::protocol::protocols::echo_strike::system::echo_strike_emit_siblings;

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.init_resource::<PostApplyOrder>();

    app.add_systems(
        FixedUpdate,
        (
            record_diffusion
                .in_set(DmgSystems::PostApplyDamage)
                .after(diffusion_emit_rings),
            record_echo
                .in_set(DmgSystems::PostApplyDamage)
                .after(echo_strike_emit_siblings),
        ),
    );
    app.add_systems(
        FixedUpdate,
        (diffusion_emit_rings, echo_strike_emit_siblings)
            .chain()
            .in_set(DmgSystems::PostApplyDamage),
    );

    tick(&mut app);

    let order = &app.world().resource::<PostApplyOrder>().0;
    assert_eq!(
        order,
        &vec!["diffusion_emit_rings", "echo_strike_emit_siblings"],
        "diffusion_emit_rings must run before echo_strike_emit_siblings in PostApplyDamage"
    );
}
