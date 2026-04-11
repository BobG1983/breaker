//! System sets for the effect domain.

use bevy::prelude::*;

/// System sets exported by the effect domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectSystems {
    /// Bridge systems that evaluate effect chains in response to triggers.
    Bridge,
}
