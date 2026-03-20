//! `OverclockEffectFired` event — fired when an overclock chain resolves to a leaf.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// Fired when an overclock trigger chain fully resolves to a leaf effect.
///
/// Consumed by per-effect observers (shockwave, multi-bolt, shield, etc.).
#[derive(Event, Clone, Debug)]
pub(crate) struct OverclockEffectFired(pub TriggerChain);
