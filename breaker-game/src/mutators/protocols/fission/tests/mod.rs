//! Fission protocol test suite.
//!
//! There is NO `sentinel` submodule: Fission does not emit `DamageDealt` /
//! `HealDealt` messages, so it does not own a source-chip sentinel const
//! (unlike `IRON_CURTAIN_SENTINEL`, `MOMENTUM_SENTINEL`). See paired test
//! spec Behavior 41.

pub(super) mod helpers;

mod activate;
mod count;
mod persistence;
mod plugin;
mod ron_asset;
mod split;
mod wire;
