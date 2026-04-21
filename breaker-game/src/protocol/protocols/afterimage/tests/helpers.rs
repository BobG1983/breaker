//! Shared test fixtures for Afterimage protocol tests.
//!
//! App builders, canonical `AfterimageConfig`, `PhantomBreaker` /
//! `PhantomBreakerLifetime` spawners, a real breaker with `BumpState` /
//! `BumpPerfectWindow` / `BumpLateWindow`, real `Bolt` with full
//! `Position2D` / `Velocity2D` / `BoltBaseDamage` / `BoltRadius`, phantom
//! bolt entity pre-seeders using the canonical bundle, `BumpPerformed`
//! writers, and the `tick_n` / `activate_now`
//! helpers.
//!
//! Mirrors the Reckless Dash / Burnout fixture shape. Canonical config is
//! `phantom_duration: 2.0, phantom_bolt_duration: 3.0` — the design-doc
//! source of truth.

use std::time::Duration;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
    time::TimeUpdateStrategy,
};
use rantzsoft_stateflow::cleanup_on_exit;

use super::super::system::{
    AfterimageConfig, PhantomBreaker, PhantomBreakerLifetime, activate, register,
};
use crate::{
    bolt::components::BoltBaseDamage,
    breaker::{
        components::{
            BaseHeight, BaseWidth, BumpLateWindow, BumpPerfectWindow, BumpState, DashState,
        },
        messages::BumpGrade,
    },
    effect_v3::{
        EffectV3Systems,
        effects::phantom_bolt::{
            components::{PhantomBolt, PhantomLifetime, PhantomOwner},
            systems::tick_phantom_lifetime,
        },
    },
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    shared::size::BaseRadius,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Afterimage test app.
///
/// State hierarchy in `NodeState::Playing`, `ActiveProtocols` initialised,
/// reader messages registered, `BumpPerformed` / `BoltImpactCell` message
/// capture installed, canonical `AfterimageConfig` inserted, and `register`
/// called. Also wires `cleanup_on_exit::<NodeState>` on `OnEnter(Teardown)`
/// so Group J transition tests see the stateflow cleanup handler despawn
/// afterimage-spawned entities.
///
/// Does NOT seed `ActiveProtocols` with Afterimage — tests that need the
/// protocol active call [`seed_active_protocols_with_afterimage`].
pub(super) fn build_afterimage_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<BumpPerformed>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut()
        .insert_resource(canonical_afterimage_config());
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
    // Wire the REUSED `tick_phantom_lifetime` directly — afterimage
    // delegates phantom-bolt lifetime tick-down to this system rather
    // than re-registering it. The production wiring is
    // `SpawnPhantomConfig::register` (via `EffectV3Plugin`); the test
    // harness mirrors the OBSERVABLE behaviour by adding the system
    // straight into `FixedUpdate` — no sets, so the spec's
    // `spawn_phantom_bolt.before(EffectV3Systems::Tick)` edge (I14)
    // survives as `spawn_phantom_bolt.before(tick_phantom_lifetime)`.
    app.add_systems(
        FixedUpdate,
        tick_phantom_lifetime.in_set(EffectV3Systems::Tick),
    );
    register(&mut app);
    app
}

/// Same as [`build_afterimage_app`] but omits `AfterimageConfig`. Used to
/// exercise the harness-safe `Option<Res<AfterimageConfig>>` early-return
/// guard paths in each reader system.
pub(super) fn build_afterimage_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<BumpPerformed>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
    app.add_systems(
        FixedUpdate,
        tick_phantom_lifetime.in_set(EffectV3Systems::Tick),
    );
    register(&mut app);
    app
}

/// Same as [`build_afterimage_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests.
pub(super) fn build_afterimage_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<BumpPerformed>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut()
        .insert_resource(canonical_afterimage_config());
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
    app.add_systems(
        FixedUpdate,
        tick_phantom_lifetime.in_set(EffectV3Systems::Tick),
    );
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Afterimage config used across all system-behaviour tests.
/// Matches the design-doc source of truth — `phantom_duration: 2.0,
/// phantom_bolt_duration: 3.0`.
pub(super) const fn canonical_afterimage_config() -> AfterimageConfig {
    AfterimageConfig {
        phantom_duration:      2.0,
        phantom_bolt_duration: 3.0,
    }
}

/// Inserts a canonical Afterimage `ProtocolDefinition` into `ActiveProtocols`
/// so the `protocol_active(Afterimage)` run-condition passes.
pub(super) fn seed_active_protocols_with_afterimage(app: &mut App) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Afterimage".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Afterimage {
                phantom_duration:      2.0,
                phantom_bolt_duration: 3.0,
            },
        });
}

/// Invokes `afterimage::activate` directly via a fresh `CommandQueue` so
/// each call has an independent, deterministic flush. Mirrors the Reckless
/// Dash / Burnout `activate_now` helpers.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Breaker spawners ────────────────────────────────────────────────────────

/// Spawns `(Breaker, DashState, Position2D)`.
pub(super) fn spawn_breaker_with_dash(app: &mut App, state: DashState, position: Vec2) -> Entity {
    app.world_mut()
        .spawn((Breaker, state, Position2D(position)))
        .id()
}

/// Spawns `(Breaker, DashState, Position2D, BumpState, BumpPerfectWindow,
/// BumpLateWindow)` where the bump state / windows are specified by the
/// caller. Used by `afterimage_check_phantom_bounce` tests to force a
/// particular grade through the emitted message.
pub(super) fn spawn_breaker_with_bump_state(
    app: &mut App,
    bump: BumpState,
    perfect_window: f32,
    late_window: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Position2D(Vec2::ZERO),
            bump,
            BumpPerfectWindow(perfect_window),
            BumpLateWindow(late_window),
        ))
        .id()
}

/// Spawns a bare phantom-breaker entity (the production `afterimage_spawn_phantom_breaker`
/// is what normally produces one — this helper is used by the tick/check
/// tests that need a pre-existing phantom-breaker without exercising the
/// spawn system).
pub(super) fn spawn_phantom_breaker_at(app: &mut App, position: Vec2, lifetime: f32) -> Entity {
    app.world_mut()
        .spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(lifetime),
            Position2D(position),
            BaseWidth(100.0),
            BaseHeight(20.0),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id()
}

// ── Bolt spawners ───────────────────────────────────────────────────────────

/// Spawns a minimally-populated real `Bolt` (no `PhantomBolt` marker) with
/// `Position2D`, `Velocity2D`, `BoltBaseDamage`, and `BoltRadius`.
pub(super) fn spawn_real_bolt(
    app: &mut App,
    position: Vec2,
    velocity: Vec2,
    base_damage: f32,
    radius: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            Position2D(position),
            Velocity2D(velocity),
            BoltBaseDamage(base_damage),
            BaseRadius(radius),
        ))
        .id()
}

/// Spawns a phantom-bolt entity matching the canonical afterimage-spawn
/// bundle. Used by tests that need a pre-existing phantom-bolt to exercise
/// cascade filtering (`Without<PhantomBolt>`), lifetime tick-down, or
/// cleanup semantics without exercising the spawn system.
pub(super) fn spawn_phantom_bolt_entity(
    app: &mut App,
    owner: Entity,
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            PhantomBolt,
            PhantomLifetime(lifetime),
            PhantomOwner(owner),
            Position2D(position),
            Velocity2D(velocity),
            BoltBaseDamage(10.0),
            BaseRadius(6.0),
            CollisionLayers::new(
                BOLT_LAYER,
                BOLT_LAYER | WALL_LAYER | BREAKER_LAYER | CELL_LAYER,
            ),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id()
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message.
pub(super) fn write_bump_performed(
    app: &mut App,
    breaker: Entity,
    bolt: Option<Entity>,
    grade: BumpGrade,
) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `BumpPerformed` message.
pub(super) fn captured_bump_performed(app: &App) -> Vec<BumpPerformed> {
    app.world()
        .resource::<MessageCollector<BumpPerformed>>()
        .0
        .clone()
}

/// Counts `(Bolt, PhantomBolt)` entities currently in the world.
pub(super) fn phantom_bolt_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<PhantomBolt>)>()
        .iter(app.world())
        .count()
}

/// Returns all phantom-bolt entities owned by `owner`.
pub(super) fn phantom_bolts_owned_by(app: &mut App, owner: Entity) -> Vec<Entity> {
    let mut q = app
        .world_mut()
        .query_filtered::<(Entity, &PhantomOwner), (With<Bolt>, With<PhantomBolt>)>();
    q.iter(app.world())
        .filter_map(|(e, o)| (o.0 == owner).then_some(e))
        .collect()
}

/// Counts `PhantomBreaker` entities currently in the world.
pub(super) fn phantom_breaker_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<PhantomBreaker>>()
        .iter(app.world())
        .count()
}

// ── Time-based helpers ──────────────────────────────────────────────────────

/// Runs `n` consecutive `FixedUpdate` ticks.
pub(super) fn tick_n(app: &mut App, n: u32) {
    for _ in 0..n {
        tick(app);
    }
}

/// Drives `NodeState` through `Playing → AnimateOut → Teardown` via two
/// `NextState::<NodeState>` writes and two updates. The second update
/// fires `OnEnter(NodeState::Teardown)` which runs
/// `cleanup_on_exit::<NodeState>`.
pub(super) fn drive_to_teardown(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();
}
