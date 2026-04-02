use bevy::prelude::*;

/// Marker component identifying the bolt entity.
#[derive(Component, Debug, Default)]
pub struct Bolt;

/// Marker component identifying the single primary bolt entity.
///
/// Stamped by the builder's `.primary()` method. Only ONE entity should
/// have `PrimaryBolt` at any time. Use `With<PrimaryBolt>` to query for the
/// primary bolt. `ExtraBolt` entities never have this marker.
#[derive(Component, Debug, Default)]
pub struct PrimaryBolt;

/// Marker component indicating the bolt is hovering above the breaker,
/// waiting for the player to launch it. Present only on the first node.
#[derive(Component, Debug)]
pub struct BoltServing;

/// Bolt radius in world units.
///
/// Alias for `BaseRadius` -- bolt uses the shared radius component.
pub type BoltRadius = crate::shared::size::BaseRadius;

/// Vertical offset above the breaker where the bolt spawns.
#[derive(Component, Debug)]
pub struct BoltSpawnOffsetY(pub f32);

/// Marker for extra bolts spawned by breaker consequences (e.g. Prism).
///
/// Extra bolts are despawned on loss rather than respawned. Only the
/// baseline bolt (without this marker) respawns.
#[derive(Component, Debug)]
pub struct ExtraBolt;

/// Marks a bolt as having been spawned by an evolution chip.
///
/// Used for damage attribution — cell kills by this bolt count toward the
/// named evolution's cumulative damage for the `MostPowerfulEvolution` highlight.
#[derive(Component, Debug, Clone)]
pub struct SpawnedByEvolution(pub String);

/// Remaining pierces before exhaustion. Reset to [`ActivePiercings::total()`] on
/// wall/breaker contact.
///
/// This is bolt gameplay state — decremented by `bolt_cell_collision` on each
/// pierce-through, reset by `bolt_wall_collision` and `bolt_breaker_collision`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiercingRemaining(pub u32);

/// Countdown timer that despawns the bolt when it expires.
///
/// Used by phantom bolts and other temporary bolt-like entities
/// to auto-destroy after a configured duration.
#[derive(Component, Debug)]
pub struct BoltLifespan(pub Timer);

/// Base damage per hit for this bolt.
#[derive(Component, Debug, Clone, Copy)]
pub struct BoltBaseDamage(pub f32);

/// Name of the `BoltDefinition` this bolt was created from.
/// Used to look up the bolt's definition in `BoltRegistry`.
#[derive(Component, Debug, Clone)]
pub struct BoltDefinitionRef(pub String);

/// Angle spread from vertical in radians for launch and respawn.
#[derive(Component, Debug, Clone, Copy)]
pub struct BoltAngleSpread(pub f32);

/// Which side of a surface the bolt last rebounded from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactSide {
    /// Top surface — bolt bounced downward off a top edge.
    Top,
    /// Bottom surface — bolt bounced upward off a bottom edge.
    Bottom,
    /// Left surface — bolt bounced rightward off a left edge.
    Left,
    /// Right surface — bolt bounced leftward off a right edge.
    Right,
}

/// Tracks where and how the bolt last rebounded.
///
/// Updated on wall/breaker/cell bounces (NOT on pierce-through, NOT on
/// phantom pass-through). Used by mirror-protocol and other side-dependent
/// effects.
#[derive(Component, Debug, Clone)]
pub struct LastImpact {
    /// World position of the impact point.
    pub position: Vec2,
    /// Which side of the surface was hit.
    pub side: ImpactSide,
}

/// Converts a CCD hit normal to the corresponding [`ImpactSide`].
///
/// CCD normals point away from the struck surface toward the bolt origin:
/// - `Vec2::NEG_X` (surface faces left) -> `ImpactSide::Left`
/// - `Vec2::X` (surface faces right) -> `ImpactSide::Right`
/// - `Vec2::NEG_Y` (surface faces down) -> `ImpactSide::Bottom`
/// - `Vec2::Y` (surface faces up) -> `ImpactSide::Top`
///
/// For non-axis-aligned normals, the dominant axis determines the side.
#[must_use]
pub fn ccd_normal_to_impact_side(normal: Vec2) -> ImpactSide {
    if normal.x.abs() > normal.y.abs() {
        if normal.x < 0.0 {
            ImpactSide::Left
        } else {
            ImpactSide::Right
        }
    } else if normal.y < 0.0 {
        ImpactSide::Bottom
    } else {
        ImpactSide::Top
    }
}

/// Converts a wall push-out normal to the corresponding [`ImpactSide`].
///
/// Wall push-out normals point **away** from the wall (outward), which is
/// the opposite direction from where the bolt actually impacted:
/// - `Vec2::X` (pushed right, away from left wall) -> `ImpactSide::Left`
/// - `Vec2::NEG_X` (pushed left, away from right wall) -> `ImpactSide::Right`
/// - `Vec2::Y` (pushed up, away from floor) -> `ImpactSide::Bottom`
/// - `Vec2::NEG_Y` (pushed down, away from ceiling) -> `ImpactSide::Top`
///
/// For non-axis-aligned normals, the dominant axis determines the side.
#[must_use]
pub fn wall_normal_to_impact_side(push_normal: Vec2) -> ImpactSide {
    if push_normal.x.abs() > push_normal.y.abs() {
        if push_normal.x > 0.0 {
            ImpactSide::Left
        } else {
            ImpactSide::Right
        }
    } else if push_normal.y > 0.0 {
        ImpactSide::Bottom
    } else {
        ImpactSide::Top
    }
}
