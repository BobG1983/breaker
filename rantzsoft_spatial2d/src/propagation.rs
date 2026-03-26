//! Parent/child propagation mode enums: `PositionPropagation`, `RotationPropagation`,
//! `ScalePropagation`.

use bevy::prelude::*;

/// How a child's `Position2D` relates to its parent.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum PositionPropagation {
    /// Position is offset from parent.
    #[default]
    Relative,
    /// Position is in world space, ignoring parent.
    Absolute,
}

/// How a child's `Rotation2D` relates to its parent.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum RotationPropagation {
    /// Rotation is offset from parent.
    #[default]
    Relative,
    /// Rotation is in world space, ignoring parent.
    Absolute,
}

/// How a child's `Scale2D` relates to its parent.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ScalePropagation {
    /// Scale is multiplied by parent.
    #[default]
    Relative,
    /// Scale is in world space, ignoring parent.
    Absolute,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_propagation_default_is_relative() {
        assert_eq!(
            PositionPropagation::default(),
            PositionPropagation::Relative
        );
    }

    #[test]
    fn rotation_propagation_default_is_relative() {
        assert_eq!(
            RotationPropagation::default(),
            RotationPropagation::Relative
        );
    }

    #[test]
    fn scale_propagation_default_is_relative() {
        assert_eq!(ScalePropagation::default(), ScalePropagation::Relative);
    }
}
