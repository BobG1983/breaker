//! System to handle `KillYourself<Breaker>` — mark the breaker `Dead`, emit
//! `Destroyed<Breaker>`, and trigger the run-lost flow.
//!
//! `LoseLife` directly decrements `Hp.current` on the aegis breaker (bypassing
//! `DamageDealt`). `detect_deaths::<Breaker>` fires `KillYourself<Breaker>`
//! when the breaker's hp reaches 0 — but nothing consumes it. This system is
//! the specialized breaker kill handler: it inserts `Dead` to prevent
//! double-processing, writes `Destroyed<Breaker>` so the existing
//! `on_breaker_destroyed` effect bridge can dispatch `Died` / `Killed` /
//! `DeathOccurred` triggers, and writes a single `RunLost` message that
//! [`handle_run_lost`](super::handle_run_lost) turns into
//! `NodeResult::LivesDepleted` + a state transition to `NodeState::AnimateOut`.
//!
//! Unlike `handle_kill<T>`, this handler does **not** write `DespawnEntity`
//! — the breaker must survive through the full end-of-run flow.

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;

use crate::{
    prelude::*, shared::death_pipeline::kill_yourself::KillYourself, state::run::messages::RunLost,
};

type BreakerVictimQuery<'w, 's> =
    Query<'w, 's, (Entity, Option<&'static Position2D>), (With<Breaker>, Without<Dead>)>;

/// Reads `KillYourself<Breaker>` messages, marks the victim `Dead`, and writes
/// a single `RunLost` message.
///
/// ### Idempotency
///
/// Two layers mirror the generic `handle_kill<T>` handler:
///
/// 1. **Cross-frame**: the victim query filters `Without<Dead>`, so a
///    `KillYourself<Breaker>` message for a victim already marked `Dead` in a
///    prior frame is dropped — no second `RunLost` is emitted.
/// 2. **Same-frame**: a local `HashSet<Entity>` tracks victims handled this
///    invocation. `commands.entity(v).insert(Dead)` is deferred until the next
///    command flush, so two `KillYourself<Breaker>` messages for the same
///    victim in one tick would both see an un-`Dead` victim without this set.
pub(crate) fn handle_breaker_death(
    mut reader: MessageReader<KillYourself<Breaker>>,
    victim_query: BreakerVictimQuery,
    killer_query: Query<&Position2D>,
    mut destroyed_writer: MessageWriter<Destroyed<Breaker>>,
    mut run_lost_writer: MessageWriter<RunLost>,
    mut commands: Commands,
) {
    let mut seen: HashSet<Entity> = HashSet::new();

    for msg in reader.read() {
        if !seen.insert(msg.victim) {
            continue;
        }

        // Cross-frame idempotency: `Without<Dead>` filter drops victims already
        // marked `Dead`. Nonexistent victims are also dropped. `Position2D` is
        // optional on the breaker — defaults to `Vec2::ZERO` for the
        // `Destroyed<Breaker>` payload if absent.
        let Ok((_, position)) = victim_query.get(msg.victim) else {
            continue;
        };
        let victim_pos = position.map_or(Vec2::ZERO, |&Position2D(p)| p);

        let killer_pos = msg
            .killer
            .and_then(|k| killer_query.get(k).ok())
            .map(|&Position2D(pos)| pos);

        commands.entity(msg.victim).insert(Dead);

        destroyed_writer.write(Destroyed {
            victim: msg.victim,
            killer: msg.killer,
            victim_pos,
            killer_pos,
            _marker: PhantomData,
        });

        run_lost_writer.write(RunLost);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::shared::death_pipeline::despawn_entity::DespawnEntity;

    // ── Test helpers ───────────────────────────────────────────────────────

    /// Pending `KillYourself<Breaker>` messages to enqueue each tick.
    #[derive(Resource, Default)]
    struct PendingBreakerKills(Vec<KillYourself<Breaker>>);

    /// System that writes `KillYourself<Breaker>` from `PendingBreakerKills`.
    fn enqueue_breaker_kills(
        pending: Res<PendingBreakerKills>,
        mut writer: MessageWriter<KillYourself<Breaker>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    fn build_app() -> App {
        TestAppBuilder::new()
            .with_message::<KillYourself<Breaker>>()
            .with_message_capture::<RunLost>()
            .with_message_capture::<DespawnEntity>()
            .with_message_capture::<Destroyed<Breaker>>()
            .with_resource::<PendingBreakerKills>()
            .with_system(
                FixedUpdate,
                enqueue_breaker_kills.before(handle_breaker_death),
            )
            .with_system(FixedUpdate, handle_breaker_death)
            .build()
    }

    fn kill_msg(victim: Entity) -> KillYourself<Breaker> {
        KillYourself {
            victim,
            killer: None,
            _marker: PhantomData,
        }
    }

    // ── Behavior 1: inserts `Dead` marker on the breaker ─────────────────

    #[test]
    fn handle_breaker_death_inserts_dead_marker_on_breaker() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default()))
            .id();

        app.insert_resource(PendingBreakerKills(vec![kill_msg(breaker)]));
        tick(&mut app);

        assert!(
            app.world().get::<Dead>(breaker).is_some(),
            "handle_breaker_death should insert Dead marker on the breaker"
        );
    }

    // ── Behavior 2: writes a single RunLost message ──────────────────────

    #[test]
    fn handle_breaker_death_writes_run_lost_message() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default()))
            .id();

        app.insert_resource(PendingBreakerKills(vec![kill_msg(breaker)]));
        tick(&mut app);

        let run_lost = app.world().resource::<MessageCollector<RunLost>>();
        assert_eq!(
            run_lost.0.len(),
            1,
            "handle_breaker_death should emit exactly one RunLost message"
        );
    }

    // ── Behavior 3: does not despawn the breaker ─────────────────────────

    #[test]
    fn handle_breaker_death_does_not_despawn_breaker() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default()))
            .id();

        app.insert_resource(PendingBreakerKills(vec![kill_msg(breaker)]));
        tick(&mut app);

        assert!(
            app.world().get_entity(breaker).is_ok(),
            "Breaker entity should still exist after handle_breaker_death"
        );

        let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
        assert_eq!(
            despawns.0.len(),
            0,
            "handle_breaker_death must NOT enqueue a DespawnEntity (the breaker \
             must survive through end-of-run flow)"
        );
    }

    // ── Behavior 4: idempotent via Dead filter ──────────────────────────

    #[test]
    fn handle_breaker_death_is_idempotent_via_dead_filter() {
        // Breaker is already marked Dead — the system must not emit a second
        // RunLost when a KillYourself<Breaker> arrives for an already-Dead
        // victim. handle_kill<T> uses `Without<Dead>` in its victim query; this
        // test enforces the same convention for handle_breaker_death.
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default(), Dead))
            .id();

        app.insert_resource(PendingBreakerKills(vec![kill_msg(breaker)]));
        tick(&mut app);

        let run_lost = app.world().resource::<MessageCollector<RunLost>>();
        assert_eq!(
            run_lost.0.len(),
            0,
            "handle_breaker_death should filter already-Dead victims and emit \
             no RunLost"
        );
    }

    // ── Behavior 5: handles empty message queue ─────────────────────────

    #[test]
    fn handle_breaker_death_handles_empty_message_queue() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(3.0), KilledBy::default()))
            .id();

        // No PendingBreakerKills populated (default = empty).
        tick(&mut app);

        assert!(
            app.world().get::<Dead>(breaker).is_none(),
            "no Dead marker should be inserted when no KillYourself<Breaker> was sent"
        );

        let run_lost = app.world().resource::<MessageCollector<RunLost>>();
        assert_eq!(
            run_lost.0.len(),
            0,
            "no RunLost should be emitted when no KillYourself<Breaker> was sent"
        );
    }

    // ── Behavior 6: multiple messages same frame → single RunLost ───────

    #[test]
    fn handle_breaker_death_handles_multiple_messages_same_frame() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default()))
            .id();

        app.insert_resource(PendingBreakerKills(vec![
            kill_msg(breaker),
            kill_msg(breaker),
        ]));
        tick(&mut app);

        let run_lost = app.world().resource::<MessageCollector<RunLost>>();
        assert_eq!(
            run_lost.0.len(),
            1,
            "two KillYourself<Breaker> messages for the same breaker in one \
             tick should produce exactly one RunLost"
        );
    }

    // ── Behavior 7: writes Destroyed<Breaker> with victim position ──────
    //
    // The effect bridge `on_breaker_destroyed` reads `Destroyed<Breaker>` to
    // dispatch `Died` / `Killed(Breaker)` / `DeathOccurred(Breaker)` triggers,
    // so `handle_breaker_death` must emit it alongside `RunLost`.

    #[test]
    fn handle_breaker_death_writes_destroyed_with_victim_position() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((
                Breaker,
                Hp::new(0.0),
                KilledBy::default(),
                Position2D(Vec2::new(15.0, 25.0)),
            ))
            .id();

        app.insert_resource(PendingBreakerKills(vec![kill_msg(breaker)]));
        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<Breaker>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "handle_breaker_death should emit exactly one Destroyed<Breaker>"
        );
        let msg = &destroyed.0[0];
        assert_eq!(
            msg.victim, breaker,
            "Destroyed.victim should be the breaker"
        );
        assert_eq!(
            msg.victim_pos,
            Vec2::new(15.0, 25.0),
            "Destroyed.victim_pos should match the breaker's Position2D"
        );
        assert_eq!(
            msg.killer, None,
            "Destroyed.killer should be None when KillYourself.killer is None"
        );
        assert_eq!(
            msg.killer_pos, None,
            "Destroyed.killer_pos should be None when killer is None"
        );
    }

    // ── Behavior 8: writes Destroyed<Breaker> with killer position ──────

    #[test]
    fn handle_breaker_death_writes_destroyed_with_killer_position() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((
                Breaker,
                Hp::new(0.0),
                KilledBy::default(),
                Position2D(Vec2::new(15.0, 25.0)),
            ))
            .id();
        let killer = app
            .world_mut()
            .spawn(Position2D(Vec2::new(-5.0, -10.0)))
            .id();

        app.insert_resource(PendingBreakerKills(vec![KillYourself {
            victim:  breaker,
            killer:  Some(killer),
            _marker: PhantomData,
        }]));
        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<Breaker>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "handle_breaker_death should emit exactly one Destroyed<Breaker>"
        );
        let msg = &destroyed.0[0];
        assert_eq!(
            msg.killer,
            Some(killer),
            "Destroyed.killer should carry the killer entity from KillYourself"
        );
        assert_eq!(
            msg.killer_pos,
            Some(Vec2::new(-5.0, -10.0)),
            "Destroyed.killer_pos should match the killer's Position2D"
        );
    }

    // ── Behavior 9: Destroyed.killer_pos is None when killer lacks Position2D ──

    #[test]
    fn handle_breaker_death_writes_destroyed_without_killer_position() {
        let mut app = build_app();
        let breaker = app
            .world_mut()
            .spawn((Breaker, Hp::new(0.0), KilledBy::default()))
            .id();
        // Killer exists but has no Position2D component.
        let killer = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingBreakerKills(vec![KillYourself {
            victim:  breaker,
            killer:  Some(killer),
            _marker: PhantomData,
        }]));
        tick(&mut app);

        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<Breaker>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "handle_breaker_death should emit exactly one Destroyed<Breaker>"
        );
        let msg = &destroyed.0[0];
        assert_eq!(
            msg.killer,
            Some(killer),
            "Destroyed.killer should still carry the killer id even without a position"
        );
        assert_eq!(
            msg.killer_pos, None,
            "Destroyed.killer_pos should be None when the killer lacks Position2D"
        );
    }
}
