//! Regen behavior components.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a regen-type cell.
///
/// Never removed. Identifies cells that participate in the regen mechanic.
#[derive(Component, Debug)]
pub struct RegenCell;

/// State marker — cell is currently regenerating.
///
/// Present at spawn. Removed to disable regen.
#[derive(Component, Debug)]
pub struct Regen;

/// HP regenerated per second.
///
/// Newtype wrapper around `f32`. Replaces the old `CellRegen { rate }`.
#[derive(Component, Debug, Clone, Copy)]
pub struct RegenRate(pub f32);

/// State marker — inserted to disable regen.
///
/// Future use. Not currently inserted by any system.
#[derive(Component, Debug)]
pub struct NoRegen;
