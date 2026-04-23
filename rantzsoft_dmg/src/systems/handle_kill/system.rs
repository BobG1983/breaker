//! `handle_kill::<T>` — resolves pending kill messages.
//!
//! Reads `KillYourself<T>` messages, inserts the `Dead` marker on the
//! victim, emits `Destroyed<T>` with victim and (optional) killer
//! positions, and enqueues a `DespawnEntity` for
//! `process_despawn_requests` to consume in `FixedPostUpdate`. Runs in
//! `DmgSystems::ApplyKill`.
//!
//! ## Idempotency
//!
//! - Cross-tick: victim query filters `Without<Dead>`.
//! - Within-invocation: `Local<HashSet<Entity>>` (cleared at top of each
//!   call) dedupes same-tick duplicate messages.

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    components::Dead,
    messages::{DespawnEntity, Destroyed, KillYourself},
    traits::Dmgable,
};

type VictimQuery<'w, 's, T> = Query<'w, 's, Option<&'static Position2D>, (With<T>, Without<Dead>)>;
type KillerQuery<'w, 's> = Query<'w, 's, &'static Position2D>;

/// Resolve each pending `KillYourself<T>`: insert `Dead`, emit
/// `Destroyed<T>`, enqueue `DespawnEntity`.
///
/// `Dead` is inserted unconditionally for every unique victim (subject to
/// same-tick dedup) so the entity cannot re-appear in `detect_deaths`'s
/// `Without<Dead>` set on subsequent ticks. `Destroyed<T>` and
/// `DespawnEntity` are only emitted when the victim has a `Position2D`
/// component — those are reporting artifacts that need a location.
///
/// The `Local<HashSet<Entity>>` is cleared at the top of the function on
/// every call (per-tick freshness) while retaining its allocated capacity
/// across ticks (amortized allocation cost).
pub(crate) fn handle_kill<T: Dmgable>(
    mut reader: MessageReader<KillYourself<T>>,
    mut seen: Local<HashSet<Entity>>,
    victim_query: VictimQuery<T>,
    killer_query: KillerQuery,
    mut destroyed_writer: MessageWriter<Destroyed<T>>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
    mut commands: Commands,
) {
    seen.clear();

    for msg in reader.read() {
        if !seen.insert(msg.victim) {
            continue;
        }

        // Query returns Ok(Option<&Position2D>) when the entity exists,
        // carries `T`, and is not Dead. The Option captures whether
        // Position2D is present. Err here means the entity was despawned,
        // lost its `T` marker, or is already Dead — skip entirely to
        // avoid panicking on `commands.entity(despawned).insert(...)`
        // and to avoid redundant Dead insertions.
        let Ok(victim_pos_opt) = victim_query.get(msg.victim) else {
            continue;
        };

        // Always transition the victim to Dead, even if Position2D is
        // absent. Without this, `detect_deaths` would re-emit
        // `KillYourself<T>` for the same entity every tick indefinitely
        // (the entity would never leave the `Without<Dead>` set).
        commands.entity(msg.victim).insert(Dead);

        // Destroyed and DespawnEntity require the victim's Position2D
        // for reporting. If it's missing, silently skip those effects.
        let Some(&Position2D(victim_pos)) = victim_pos_opt else {
            continue;
        };

        let killer_pos = msg
            .killer
            .and_then(|k| killer_query.get(k).ok())
            .map(|&Position2D(pos)| pos);

        destroyed_writer.write(Destroyed {
            victim: msg.victim,
            killer: msg.killer,
            victim_pos,
            killer_pos,
            _marker: PhantomData,
        });

        despawn_writer.write(DespawnEntity { entity: msg.victim });
    }
}
