//! Resonance hazard — kill-chain waves slow the Breaker.
//!
//! Track `(timestamp, victim_pos)` pairs in a per-run `ResonanceTracker`
//! resource (one entry per `Destroyed<Cell>`). Each tick: prune entries
//! older than the effective window; if the count exceeds
//! `kills_to_trigger`, spawn one wave per excess kill using the positions
//! of the most-recent entries. Waves travel toward the Breaker's snapshot
//! position at spawn; on live-distance contact they fire a
//! `SpeedBoostConfig` onto the Breaker's stack and register a
//! `ResonanceSlowEntry` in `ResonanceActiveSlows`. A separate tick system
//! drains each entry's `remaining` and reverses the stack entry when the
//! timer elapses. On node teardown, all tracker state + active slows are
//! reversed and drained; wave entities are despawned by the shared
//! `CleanupOnExit<NodeState>` hook.

use std::collections::HashMap;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    breaker::{components::Breaker, sets::BreakerSystems},
    cells::components::Cell,
    effect_v3::{
        commands::EffectCommandsExt,
        effects::SpeedBoostConfig,
        types::{EffectType, ReversibleEffectType},
    },
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

// ── ResonanceConfig ──────────────────────────────────────────────────────

/// Per-run tuning extracted from [`HazardTuning::Resonance`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ResonanceConfig {
    /// Minimum kills within the window before waves start firing.
    pub(crate) kills_to_trigger:      u32,
    /// Window (seconds) at stack 1.
    pub(crate) base_window:           f32,
    /// Additional seconds per stack beyond the first.
    pub(crate) window_per_level:      f32,
    /// Wave travel speed (world units/sec).
    pub(crate) wave_speed:            f32,
    /// Base slow duration (seconds) at stack 1.
    pub(crate) base_slow_duration:    f32,
    /// Base slow strength at stack 1 (0.0..1.0).
    pub(crate) base_slow_strength:    f32,
    /// Log-scale coefficient for slow-duration diminishing returns.
    pub(crate) slow_duration_scaling: f32,
    /// Log-scale coefficient for slow-strength diminishing returns.
    pub(crate) slow_strength_scaling: f32,
    /// Distance (world units) at which a wave contacts the breaker.
    pub(crate) contact_threshold:     f32,
    /// Hard cap (seconds) after which a non-contacting wave despawns.
    pub(crate) wave_max_lifetime:     f32,
}

impl ResonanceConfig {
    /// Effective sliding-window duration for `stacks`.
    ///
    /// Returns `0.0` at stack 0 (hazard inactive — callers are gated, but
    /// the helper stays sound). Otherwise linear:
    /// `base_window + window_per_level * (stacks - 1)`.
    #[must_use]
    pub(crate) const fn effective_window(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = (stacks - 1) as f32;
        self.window_per_level.mul_add(extra, self.base_window)
    }

    /// Effective slow duration for `stacks`.
    ///
    /// `base_slow_duration * (1 + slow_duration_scaling * ln(stacks))`.
    /// Stack 0 → `0.0`; stack 1 → `base_slow_duration` (because ln(1)=0).
    #[must_use]
    pub(crate) fn effective_slow_duration(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let scaled = self
            .slow_duration_scaling
            .mul_add((stacks as f32).ln(), 1.0);
        self.base_slow_duration * scaled
    }

    /// Effective slow strength for `stacks`.
    ///
    /// `base_slow_strength * (1 + slow_strength_scaling * ln(stacks))`.
    /// Stack 0 → `0.0`; stack 1 → `base_slow_strength`.
    #[must_use]
    pub(crate) fn effective_slow_strength(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let scaled = self
            .slow_strength_scaling
            .mul_add((stacks as f32).ln(), 1.0);
        self.base_slow_strength * scaled
    }

    /// `SpeedBoostConfig` multiplier for `stacks` — always in `[0.1, 1.0]`.
    ///
    /// Test-only: production contact snapshots the wave's `slow_strength`
    /// at spawn time and derives the multiplier inline (waves must use the
    /// spawn-time stack count, not the current one). This helper exists
    /// so formula tests can assert the clamp + derivation independently.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn slow_multiplier(self, stacks: u32) -> f32 {
        (1.0 - self.effective_slow_strength(stacks)).clamp(0.1, 1.0)
    }
}

// ── ResonanceTracker ─────────────────────────────────────────────────────

/// Per-kill `(timestamp_secs, victim_pos)` pairs within the effective window.
#[derive(Resource, Debug, Default)]
pub(crate) struct ResonanceTracker {
    /// Insertion-ordered kill log. Earliest first, most-recent last.
    pub(crate) kills: Vec<(f32, Vec2)>,
}

// ── ResonanceSlowEntry + ResonanceActiveSlows ───────────────────────────

/// Value stored in [`ResonanceActiveSlows::slows`] — remaining seconds +
/// the exact `SpeedBoostConfig.multiplier` fired (required for reversal).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ResonanceSlowEntry {
    /// Seconds before this slow expires. Decremented by `resonance_tick_slows`.
    pub(crate) remaining:  f32,
    /// The exact `SpeedBoostConfig.multiplier` fired for this source.
    pub(crate) multiplier: OrderedFloat<f32>,
}

/// Active resonance-applied slows keyed by per-wave source id.
///
/// Resource over per-entity components — cleanup is centralized (one
/// drain + reverse loop on node teardown) and we avoid per-wave
/// archetype churn on the breaker.
#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct ResonanceActiveSlows {
    /// Active slows keyed by `"hazard:resonance:wave:{entity_bits}"`.
    pub(crate) slows: HashMap<String, ResonanceSlowEntry>,
}

// ── ResonanceWave ────────────────────────────────────────────────────────

/// A resonance wave traveling toward the breaker.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ResonanceWave {
    /// Travel speed (units/sec). Snapshotted at spawn.
    pub(crate) speed:             f32,
    /// Effective slow duration (seconds) when contact occurs.
    pub(crate) slow_duration:     f32,
    /// Effective slow strength (0.0..1.0) when contact occurs.
    pub(crate) slow_strength:     f32,
    /// Breaker position snapshotted at spawn time.
    pub(crate) target_pos:        Vec2,
    /// Seconds elapsed since spawn.
    pub(crate) age:               f32,
    /// Snapshotted lifetime cap (seconds).
    pub(crate) max_lifetime:      f32,
    /// Snapshotted contact distance threshold.
    pub(crate) contact_threshold: f32,
}

// ── activate + register ──────────────────────────────────────────────────

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Resonance {
        kills_to_trigger,
        base_window,
        window_per_level,
        wave_speed,
        base_slow_duration,
        base_slow_strength,
        slow_duration_scaling,
        slow_strength_scaling,
        contact_threshold,
        wave_max_lifetime,
    } = *tuning
    else {
        warn!("resonance::activate called with non-Resonance tuning");
        return;
    };
    commands.insert_resource(ResonanceConfig {
        kills_to_trigger,
        base_window,
        window_per_level,
        wave_speed,
        base_slow_duration,
        base_slow_strength,
        slow_duration_scaling,
        slow_strength_scaling,
        contact_threshold,
        wave_max_lifetime,
    });
    commands.insert_resource(ResonanceTracker::default());
    commands.insert_resource(ResonanceActiveSlows::default());
}

pub(crate) fn register(app: &mut App) {
    app.init_resource::<ResonanceTracker>()
        .init_resource::<ResonanceActiveSlows>()
        .add_systems(
            FixedUpdate,
            (
                (
                    resonance_spawn_waves,
                    resonance_wave_travel,
                    resonance_wave_contact,
                    resonance_wave_expire,
                )
                    .chain(),
                resonance_tick_slows.before(BreakerSystems::Move),
            )
                .run_if(hazard_active(HazardKind::Resonance))
                .run_if(in_state(NodeState::Playing)),
        )
        // Reader system `resonance_track_kills` is intentionally ungated at
        // the tuple level — it holds `MessageReader<Destroyed<Cell>>` and
        // enforces the `ActiveHazards` / `NodeState::Playing` gate in-body
        // via an immediate `reader.clear()` + return when inactive. A
        // `.run_if(...)` gate suppresses execution but does NOT advance
        // the reader cursor, so messages buffered during gated-off frames
        // would get retroactively consumed the tick the gate opens.
        .add_systems(
            FixedUpdate,
            resonance_track_kills
                .after(DmgSystems::ApplyKill)
                .before(resonance_spawn_waves),
        )
        .add_systems(OnEnter(NodeState::Teardown), resonance_cleanup_on_teardown);
}

// ── System A: resonance_track_kills ──────────────────────────────────────

/// Pushes `(time.elapsed_secs(), victim_pos)` onto `tracker.kills` for each
/// `Destroyed<Cell>` message.
///
/// Gated in-body: this system runs every `FixedUpdate` tick. When
/// Resonance is not active or `NodeState` is not `Playing`, it drains
/// the `MessageReader` via `reader.clear()` and returns so buffered
/// `Destroyed<Cell>` messages cannot leak retroactively when the hazard
/// activates on a later frame. Also drains when the config is absent.
pub(crate) fn resonance_track_kills(
    mut reader: MessageReader<Destroyed<Cell>>,
    active_hazards: Option<Res<ActiveHazards>>,
    node_state: Option<Res<State<NodeState>>>,
    config: Option<Res<ResonanceConfig>>,
    time: Res<Time<Fixed>>,
    mut tracker: ResMut<ResonanceTracker>,
) {
    if active_hazards
        .as_ref()
        .is_none_or(|ah| !ah.is_active(HazardKind::Resonance))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    if config.is_none() {
        reader.clear();
        return;
    }
    let now = time.elapsed_secs();
    for destroyed in reader.read() {
        tracker.kills.push((now, destroyed.victim_pos));
    }
}

// ── System B: resonance_spawn_waves ──────────────────────────────────────

/// Prunes stale tracker entries, and for each kill beyond
/// `kills_to_trigger` spawns one wave at that kill's position. After
/// spawning, exactly `kills_to_trigger` entries remain as carryover.
pub(crate) fn resonance_spawn_waves(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<ResonanceConfig>>,
    mut tracker: ResMut<ResonanceTracker>,
    breakers: Query<&Position2D, With<Breaker>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        return;
    };
    let config = *config;
    let stacks = active.stacks(HazardKind::Resonance);
    if stacks == 0 {
        return;
    }
    let effective_window = config.effective_window(stacks).max(0.0);
    let now = time.elapsed_secs();

    // Prune stale entries.
    tracker.kills.retain(|(t, _)| now - *t <= effective_window);

    if (tracker.kills.len() as u32) <= config.kills_to_trigger {
        return;
    }

    let excess = (tracker.kills.len() as u32 - config.kills_to_trigger) as usize;

    let Some(breaker_pos) = breakers.iter().next().copied() else {
        // No breaker — leave tracker as-is (already pruned); try again next tick.
        return;
    };

    let slow_duration = config.effective_slow_duration(stacks);
    let slow_strength = config.effective_slow_strength(stacks);
    let target_pos = breaker_pos.0;
    let speed = config.wave_speed;
    let max_lifetime = config.wave_max_lifetime;
    let contact_threshold = config.contact_threshold;

    // Drain the tail `excess` entries — those ARE the kills-beyond-threshold.
    // The leading entries remain as carryover "prior kills".
    let drain_start = tracker.kills.len() - excess;
    for (_, victim_pos) in tracker.kills.drain(drain_start..) {
        commands.spawn((
            ResonanceWave {
                speed,
                slow_duration,
                slow_strength,
                target_pos,
                age: 0.0,
                max_lifetime,
                contact_threshold,
            },
            Position2D(victim_pos),
            CleanupOnExit::<NodeState>::default(),
        ));
    }
    debug_assert_eq!(
        tracker.kills.len() as u32,
        config.kills_to_trigger,
        "post-spawn tracker must retain exactly kills_to_trigger entries"
    );
}

// ── System C: resonance_wave_travel ──────────────────────────────────────

/// Moves each wave toward its stored `target_pos` by `speed * dt`, clamps
/// at target (no overshoot), and ticks `age`.
pub(crate) fn resonance_wave_travel(
    time: Res<Time<Fixed>>,
    mut waves: Query<(&mut ResonanceWave, &mut Position2D)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (mut wave, mut pos) in &mut waves {
        wave.age += dt;
        let delta = wave.target_pos - pos.0;
        let distance = delta.length();
        if distance <= f32::EPSILON {
            continue;
        }
        let step = wave.speed * dt;
        if step >= distance {
            pos.0 = wave.target_pos;
        } else {
            pos.0 += (delta / distance) * step;
        }
    }
}

// ── System D: resonance_wave_contact ─────────────────────────────────────

/// Fires a `SpeedBoostConfig` entry onto the breaker's stack and registers
/// a `ResonanceSlowEntry` for any wave within `contact_threshold` of the
/// LIVE breaker position. Despawns contacting waves.
pub(crate) fn resonance_wave_contact(
    waves: Query<(Entity, &ResonanceWave, &Position2D)>,
    breakers: Query<(Entity, &Position2D), With<Breaker>>,
    mut slows: ResMut<ResonanceActiveSlows>,
    mut commands: Commands,
) {
    let Some((breaker_entity, &Position2D(breaker_pos))) = breakers.iter().next() else {
        return;
    };
    for (wave_entity, wave, pos) in &waves {
        let distance = (breaker_pos - pos.0).length();
        if distance > wave.contact_threshold {
            continue;
        }

        let mult = (1.0 - wave.slow_strength).clamp(0.1, 1.0);
        let multiplier = OrderedFloat(mult);
        let source = format!("hazard:resonance:wave:{}", wave_entity.to_bits());

        commands.fire_effect(
            breaker_entity,
            EffectType::SpeedBoost(SpeedBoostConfig { multiplier }),
            source.clone(),
        );
        slows.slows.insert(
            source,
            ResonanceSlowEntry {
                remaining: wave.slow_duration,
                multiplier,
            },
        );
        commands.entity(wave_entity).despawn();
    }
}

// ── System E: resonance_wave_expire ──────────────────────────────────────

/// Despawns waves whose `age >= max_lifetime` without applying slow.
/// Waves that contacted this tick are already queued for despawn by
/// `resonance_wave_contact`; re-queuing despawn is safe.
pub(crate) fn resonance_wave_expire(
    waves: Query<(Entity, &ResonanceWave)>,
    mut commands: Commands,
) {
    for (entity, wave) in &waves {
        if wave.age >= wave.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

// ── System F: resonance_tick_slows ───────────────────────────────────────

/// Decrements each `remaining` by `dt`. When `remaining <= 0.0`, reverses
/// the corresponding `SpeedBoostConfig` entry from the breaker's stack and
/// removes the map entry.
pub(crate) fn resonance_tick_slows(
    time: Res<Time<Fixed>>,
    mut slows: ResMut<ResonanceActiveSlows>,
    breakers: Query<Entity, With<Breaker>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let breaker_entity = breakers.iter().next();

    // Decrement remaining; collect expired sources to remove after iteration.
    let mut expired: Vec<(String, OrderedFloat<f32>)> = Vec::new();
    for (source, entry) in &mut slows.slows {
        entry.remaining -= dt;
        if entry.remaining <= 0.0 {
            expired.push((source.clone(), entry.multiplier));
        }
    }

    for (source, multiplier) in expired {
        slows.slows.remove(&source);
        if let Some(breaker_entity) = breaker_entity {
            commands.reverse_effect(
                breaker_entity,
                ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier }),
                source,
            );
        }
    }
}

// ── Cleanup: resonance_cleanup_on_teardown ───────────────────────────────

/// On `OnEnter(NodeState::Teardown)`: drain tracker, reverse every active
/// slow from the breaker's stack, clear the slows map. Wave entities are
/// despawned by the shared `CleanupOnExit<NodeState>` hook at the same
/// transition (registered by `NodePlugin`).
pub(crate) fn resonance_cleanup_on_teardown(
    mut tracker: ResMut<ResonanceTracker>,
    mut slows: ResMut<ResonanceActiveSlows>,
    breakers: Query<Entity, With<Breaker>>,
    mut commands: Commands,
) {
    tracker.kills.clear();

    if let Some(breaker_entity) = breakers.iter().next() {
        for (source, entry) in &slows.slows {
            commands.reverse_effect(
                breaker_entity,
                ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: entry.multiplier,
                }),
                source.clone(),
            );
        }
    }
    slows.slows.clear();
}
