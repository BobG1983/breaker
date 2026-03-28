//! Dispatches chip effects to target entities when a chip is selected.
//!
//! Reads [`ChipSelected`] messages, resolves `RootEffect::On { target, then }`
//! to entities, pushes children to `BoundEffects`. Bare `Do` children fire
//! immediately via `commands.fire_effect()`.

use bevy::prelude::*;

use crate::{
    chips::{inventory::ChipInventory, resources::ChipCatalog},
    effect::{BoundEffects, EffectCommandsExt, RootEffect, Target},
    ui::messages::ChipSelected,
};

/// Stub — reads [`ChipSelected`] messages and dispatches effects to target entities.
/// Real implementation in Wave 6.
pub(crate) fn dispatch_chip_effects(
    mut _reader: MessageReader<ChipSelected>,
    _registry: Option<Res<ChipCatalog>>,
    mut _inventory: Option<ResMut<ChipInventory>>,
    mut _commands: Commands,
) {
    // TODO: Wave 6 — resolve RootEffect targets, push to BoundEffects,
    // fire bare Do children via commands.fire_effect()
}
