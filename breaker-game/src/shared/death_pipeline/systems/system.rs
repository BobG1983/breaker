//! Death pipeline systems — damage application, death detection, and despawn processing.

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::shared::death_pipeline::{
    damage_dealt::DamageDealt,
    dead::Dead,
    despawn_entity::DespawnEntity,
    destroyed::Destroyed,
    game_entity::GameEntity,
    heal_dealt::{HealCap, HealDealt},
    hp::Hp,
    invulnerable::Invulnerable,
    kill_yourself::KillYourself,
    killed_by::KilledBy,
};

/// Processes `DamageDealt<T>` messages, decrements `Hp`, and sets `KilledBy` on
/// the killing blow. Uses `Without<Dead>` to skip entities already confirmed dead,
/// and `Without<Invulnerable>` to silently absorb damage against immune entities.
///
/// Generic over the entity marker type — monomorphized for Cell, Bolt, Wall, Breaker.
type DamageTargetQuery<'w, 's, T> = Query<
    'w,
    's,
    (&'static mut Hp, &'static mut KilledBy),
    (With<T>, Without<Dead>, Without<Invulnerable>),
>;

pub(crate) fn apply_damage<T: GameEntity>(
    mut reader: MessageReader<DamageDealt<T>>,
    mut query: DamageTargetQuery<T>,
) {
    for msg in reader.read() {
        let Ok((mut hp, mut killed_by)) = query.get_mut(msg.target) else {
            continue;
        };

        let was_positive = hp.current > 0.0;
        hp.current -= msg.amount;

        // Killing blow: Hp crossed from positive to <= 0.
        // First kill wins — do not overwrite if already set.
        if was_positive && hp.current <= 0.0 && killed_by.dealer.is_none() {
            killed_by.dealer = msg.dealer;
        }
    }
}

/// Detects entities of type `T` with `Hp <= 0` and sends `KillYourself<T>`.
///
/// Uses `Without<Dead>` to skip entities already confirmed dead by their domain
/// kill handler, preventing double-processing.
type DeathDetectionQuery<'w, 's, T> =
    Query<'w, 's, (Entity, &'static KilledBy, &'static Hp), (With<T>, Without<Dead>)>;

pub(crate) fn detect_deaths<T: GameEntity>(
    query: DeathDetectionQuery<T>,
    mut writer: MessageWriter<KillYourself<T>>,
) {
    for (entity, killed_by, hp) in &query {
        if hp.current <= 0.0 {
            writer.write(KillYourself {
                victim:  entity,
                killer:  killed_by.dealer,
                _marker: PhantomData,
            });
        }
    }
}

/// Query for live victim positions. `Without<Dead>` provides cross-frame
/// idempotency — an already-`Dead` victim is absent from the query and is
/// silently dropped. Entity id is passed via `msg.victim`, not the query.
type KillVictimQuery<'w, 's, T> = Query<'w, 's, &'static Position2D, (With<T>, Without<Dead>)>;

/// Query for killer positions. Not filtered by `T` — any entity may be a
/// killer. Read-only overlap with `KillVictimQuery<T>` is safe under Bevy
/// 0.18 because both only read `Position2D`.
type KillerPositionQuery<'w, 's> = Query<'w, 's, &'static Position2D>;

/// Reads `KillYourself<T>` messages, marks victims `Dead`, writes `Destroyed<T>`,
/// and enqueues `DespawnEntity` for deferred despawn in `FixedPostUpdate`.
///
/// Generic over the entity marker type. Monomorphized for `Cell`, `Bolt`, and
/// `Wall`. `Breaker` is handled separately by `handle_breaker_death` because
/// the breaker must survive through the end-of-run flow — it writes `RunLost`
/// instead of `DespawnEntity`.
///
/// ### Idempotency
///
/// Two layers handle idempotency:
///
/// 1. **Cross-frame**: `Without<Dead>` on the victim query. A `KillYourself<T>`
///    message for a victim that was marked `Dead` in a prior frame is filtered
///    out — no `Destroyed<T>`, no `DespawnEntity`, no second `Dead` insert.
///    Nonexistent victims (already despawned) are also dropped by the query.
///
/// 2. **Same-frame**: a local `HashSet<Entity>` that tracks victims already
///    handled during this invocation. `commands.entity(v).insert(Dead)` is
///    **deferred** until the next command flush, so within a single
///    invocation of `handle_kill<T>`, two `KillYourself<T>` messages for the
///    same victim would both see the victim in the filtered query (because
///    `Dead` has not yet been applied) and emit duplicate `Destroyed<T>` +
///    `DespawnEntity` messages. The `HashSet` closes that window.
///
/// ### Missing positions
///
/// Victims without `Position2D` are absent from `KillVictimQuery<T>` and are
/// silently skipped — the observable effect is no `Destroyed`, no
/// `DespawnEntity`, no `Dead`.
///
/// If the **killer** exists but lacks `Position2D`, `killer_pos` is recorded
/// as `None` (not a bug — walls or future unpositioned killers may be legal).
pub(crate) fn handle_kill<T: GameEntity>(
    mut reader: MessageReader<KillYourself<T>>,
    victim_query: KillVictimQuery<T>,
    killer_query: KillerPositionQuery,
    mut destroyed_writer: MessageWriter<Destroyed<T>>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
    mut commands: Commands,
) {
    let mut seen: HashSet<Entity> = HashSet::new();

    for msg in reader.read() {
        if !seen.insert(msg.victim) {
            continue;
        }

        let Ok(&Position2D(victim_pos)) = victim_query.get(msg.victim) else {
            continue;
        };

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

        despawn_writer.write(DespawnEntity { entity: msg.victim });
    }
}

/// Processes `DespawnEntity` messages — despawns entities via `try_despawn`.
///
/// This is the ONLY system that despawns entities in the death pipeline.
/// Runs in `FixedPostUpdate`.
pub(crate) fn process_despawn_requests(
    mut reader: MessageReader<DespawnEntity>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.entity).try_despawn();
    }
}

/// Processes `HealDealt<T>` messages and increments `Hp.current`, clamped to
/// a per-message ceiling chosen by the sender via `msg.cap`:
/// - `HealCap::Starting` → `hp.starting` (ignores `hp.max`)
/// - `HealCap::Max`      → `hp.max.unwrap_or(hp.starting)`
///
/// Uses `Without<Dead>` to skip entities already confirmed dead, and
/// `Without<Invulnerable>` to skip immune entities (mirrors
/// `apply_damage<T>`'s filters). The `Invulnerable` marker is the SAME
/// marker that blocks damage — a single shared shield, per design.
///
/// NOTE: under the chosen system-set ordering
/// (`ApplyDamage → DetectDeaths → HandleKill → ApplyHeal`), `handle_kill<T>`
/// has already inserted `Dead` on any entity that died this tick by the
/// time `apply_heal<T>` runs. The `Without<Dead>` filter is therefore the
/// mechanism that prevents same-tick in-tick revival — a 10-damage +
/// 5-heal combo on a 10-HP entity leaves the entity dead no matter how
/// large the heal amount.
///
/// Generic over the target marker type — monomorphized for Cell, Bolt, Wall,
/// Breaker, Salvo.
type HealTargetQuery<'w, 's, T> =
    Query<'w, 's, &'static mut Hp, (With<T>, Without<Dead>, Without<Invulnerable>)>;

pub(crate) fn apply_heal<T: GameEntity>(
    mut reader: MessageReader<HealDealt<T>>,
    mut query: HealTargetQuery<T>,
) {
    for msg in reader.read() {
        // Guard: amount <= 0.0 (covers 0.0, negatives, NEG_INFINITY).
        if msg.amount <= 0.0 {
            continue;
        }
        // Guard: NaN. `f32::NAN <= 0.0` is false in IEEE 754, so NaN requires
        // its own explicit guard.
        if msg.amount.is_nan() {
            continue;
        }

        // Target lookup; skipped entities (Dead, Invulnerable, missing T
        // marker, missing Hp, despawned) silently drop through via the
        // `Without<..>` filter + `get_mut()` failure.
        let Ok(mut hp) = query.get_mut(msg.target) else {
            continue;
        };

        // Per-message ceiling — must not be cached across messages.
        let ceiling = match msg.cap {
            HealCap::Starting => hp.starting,
            HealCap::Max => hp.max.unwrap_or(hp.starting),
        };

        // Over-cap short-circuit: never lower HP via a heal. Without this,
        // `(hp.current + amount).min(ceiling)` would turn a heal into a
        // debuff when `hp.current > ceiling`.
        if hp.current >= ceiling {
            continue;
        }

        hp.current = (hp.current + msg.amount).min(ceiling);
    }
}
