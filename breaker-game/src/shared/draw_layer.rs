//! Game-specific draw layer enum for Z-ordering game entities.

use bevy::prelude::*;
use rantzsoft_spatial2d::draw_layer::DrawLayer;

/// Draw layer enum for Z-ordering game entities.
///
/// Each variant maps to a fixed Z value used by the spatial propagation
/// system to set `Transform.translation.z`.
#[derive(Component, Clone, Copy, Debug)]
pub enum GameDrawLayer {
    /// Breaker paddle (Z = 0.0).
    Breaker,
    /// Cell blocks (Z = 0.0).
    Cell,
    /// Boundary walls (Z = 0.0).
    Wall,
    /// Bolt projectile (Z = 1.0).
    Bolt,
    /// Visual effects (Z = 2.0).
    Fx,
}

impl DrawLayer for GameDrawLayer {
    fn z(&self) -> f32 {
        match self {
            Self::Breaker | Self::Cell | Self::Wall => 0.0,
            Self::Bolt => 1.0,
            Self::Fx => 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bolt_z_is_one() {
        assert!((GameDrawLayer::Bolt.z() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_z_is_zero() {
        assert!(GameDrawLayer::Breaker.z().abs() < f32::EPSILON);
    }

    #[test]
    fn cell_z_is_zero() {
        assert!(GameDrawLayer::Cell.z().abs() < f32::EPSILON);
    }

    #[test]
    fn fx_z_is_two() {
        assert!((GameDrawLayer::Fx.z() - 2.0).abs() < f32::EPSILON);
    }
}
