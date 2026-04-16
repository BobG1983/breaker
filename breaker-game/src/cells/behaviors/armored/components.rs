//! Armored behavior components.
//!
//! Plain marker, newtype, and enum components. Behavior lives in the sibling
//! `systems/` module: `check_armor_direction` intercepts `DamageDealt<Cell>`
//! messages before `apply_damage::<Cell>` runs, drops entries for hits that
//! land on the armored face while the bolt lacks sufficient piercing, and
//! consumes piercing charges when the bolt does break through.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Permanent marker identifying a cell as an armored-type cell.
///
/// Never removed. Inserted alongside `ArmorValue` and `ArmorFacing` when
/// `CellBehavior::Armored` is resolved at spawn time. Lets systems query
/// "is this cell armored?" via `With<ArmoredCell>` without fetching the
/// value/facing data.
#[derive(Component, Debug)]
pub struct ArmoredCell;

/// Armor rating in the closed range `1..=3`. Stored as `u8`; the validation
/// at `CellTypeDefinition::validate()` rejects `0` and values `> 3` at RON
/// load time so every runtime instance is guaranteed to be in range. The
/// armor check system compares this against the bolt's
/// `piercing_remaining` at the instant of impact.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArmorValue(pub u8);

/// Which face of the cell carries the armor plating. The opposite face is
/// the weak point. `Bottom` is the default (plates face the breaker; weak
/// point is the top of the cell).
///
/// Derived in lockstep with `CellBehavior::Armored { facing }` — both
/// serialization and the default match the expected design-time shape.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum ArmorDirection {
    /// Armor plates the bottom face of the cell (faces the breaker). Weak
    /// point is the top face. Default.
    #[default]
    Bottom,
    /// Armor plates the top face of the cell. Weak point is the bottom.
    Top,
    /// Armor plates the left face of the cell. Weak point is the right.
    Left,
    /// Armor plates the right face of the cell. Weak point is the left.
    Right,
}

/// Facing of this armored cell — wraps `ArmorDirection` as a runtime
/// component so `check_armor_direction` can query for it.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArmorFacing(pub ArmorDirection);
