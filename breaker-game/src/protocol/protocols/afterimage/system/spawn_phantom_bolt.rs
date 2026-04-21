use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::{components::PhantomBreaker, config::AfterimageConfig};
use crate::{
    bolt::{
        components::{BoltBaseDamage, BoltRadius},
        resources::DEFAULT_BOLT_BASE_DAMAGE,
    },
    breaker::messages::BumpGrade,
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
    protocol::{definition::ProtocolKind, systems::ProtocolGate},
};

// ── System 4 — afterimage_spawn_phantom_bolt ────────────────────────────────

/// Multiplier applied to the phantom's inherited base speed to derive its
/// `MinSpeed` (`phantom_base_speed * PHANTOM_MIN_SPEED_RATIO`).
///
/// The builder's `.with_speed(base, min, max)` transition requires min/max
/// bounds, but the afterimage phantom inherits only a single base value from
/// the real bolt (either `BaseSpeed` or the current velocity magnitude). We
/// construct matching min/max bounds around that base so the phantom
/// participates in the speed-clamp pipeline with a plausible envelope.
const PHANTOM_MIN_SPEED_RATIO: f32 = 0.5;
/// Multiplier applied to the phantom's inherited base speed to derive its
/// `MaxSpeed`. See [`PHANTOM_MIN_SPEED_RATIO`].
const PHANTOM_MAX_SPEED_RATIO: f32 = 2.0;
/// Minimum-angle-from-horizontal constraint (radians) used when the phantom
/// is built via the bolt builder. Matches the `BoltDefinition` default of
/// `5.0` degrees applied by `.definition()`.
const PHANTOM_MIN_ANGLE_RAD: f32 = 5.0_f32 * std::f32::consts::PI / 180.0;

type SpawnPhantomBoltRealBoltQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position2D,
        &'static Velocity2D,
        &'static BoltRadius,
        Option<&'static BoltBaseDamage>,
        Option<&'static BaseSpeed>,
    ),
    (With<Bolt>, Without<PhantomBolt>),
>;

/// Consumes `BumpPerformed` messages. For each Perfect-graded bump whose
/// `breaker` field references an entity carrying `PhantomBreaker`, spawns
/// a new phantom-bolt entity at the real bolt's position / velocity via
/// the canonical [`Bolt::builder`] pipeline (headless + extra) and then
/// tacks on the phantom-specific marker bundle (`PhantomBolt`,
/// `PhantomLifetime`, `PhantomOwner`, `CleanupOnExit::<NodeState>`).
///
/// The builder's default `CollisionLayers` mask is `CELL_LAYER |
/// WALL_LAYER | BREAKER_LAYER`; afterimage phantoms additionally match
/// `BOLT_LAYER` so `.with_extra_mask_bits(BOLT_LAYER)` is applied. This
/// `CELL_LAYER`-included mask is the key distinction versus
/// `SpawnPhantomConfig::fire`, which excludes `CELL_LAYER` — see
/// `effect_v3::effects::phantom_bolt::components::PhantomBolt`.
///
/// Uniqueness: iterates `existing_phantom_owners` and skips when any
/// phantom already references the same real bolt. The existing phantom's
/// `PhantomLifetime` is NOT reset.
///
/// Harness-safe: drains the reader and returns when `AfterimageConfig` is
/// absent or no `PhantomBreaker` entities exist.
pub(crate) fn afterimage_spawn_phantom_bolt(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<AfterimageConfig>>,
    gate: ProtocolGate,
    phantom_breakers: Query<(), With<PhantomBreaker>>,
    real_bolts: SpawnPhantomBoltRealBoltQuery,
    existing_phantom_owners: Query<&PhantomOwner, With<PhantomBolt>>,
    mut commands: Commands,
) {
    if gate.is_closed_for(ProtocolKind::Afterimage) {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    if phantom_breakers.is_empty() {
        reader.clear();
        return;
    }
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let Some(real_bolt_entity) = msg.bolt else {
            continue;
        };
        if !phantom_breakers.contains(msg.breaker) {
            continue;
        }
        if existing_phantom_owners
            .iter()
            .any(|owner| owner.0 == real_bolt_entity)
        {
            continue;
        }
        let Ok((pos, vel, radius, base_damage_opt, base_speed_opt)) =
            real_bolts.get(real_bolt_entity)
        else {
            continue;
        };
        let base_damage = base_damage_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        // Inherit the real bolt's `BaseSpeed` so the phantom participates in the
        // bolt collision query (which requires `BaseSpeed` via `SpatialData`) and
        // applies the canonical velocity formula at the same target speed. Fall
        // back to the real bolt's current velocity magnitude when the real bolt
        // lacks a `BaseSpeed` component (minimal harness fixtures).
        let phantom_base_speed = base_speed_opt.map_or_else(|| vel.0.length(), |bs| bs.0);

        let new_bolt = Bolt::builder()
            .at_position(pos.0)
            .with_speed(
                phantom_base_speed,
                phantom_base_speed * PHANTOM_MIN_SPEED_RATIO,
                phantom_base_speed * PHANTOM_MAX_SPEED_RATIO,
            )
            .with_angle(PHANTOM_MIN_ANGLE_RAD, PHANTOM_MIN_ANGLE_RAD)
            .with_velocity(Velocity2D(vel.0))
            .extra()
            .headless()
            .with_radius(radius.0)
            .with_base_damage(base_damage)
            .with_extra_mask_bits(BOLT_LAYER)
            .spawn(&mut commands);

        // `Bolt::builder().extra().spawn(...)` already inserts
        // `CleanupOnExit::<NodeState>::default()` on the entity — no need to
        // re-insert here.
        commands.entity(new_bolt).insert((
            PhantomBolt,
            PhantomLifetime(config.phantom_bolt_duration),
            PhantomOwner(real_bolt_entity),
        ));
    }
}
