//! `DrawLayer` trait for mapping game-defined layer enums to Z values.

use bevy::prelude::*;

/// Trait for game-defined enums that map to Z ordering values.
///
/// Implementors provide a `z()` method that returns the Z coordinate
/// used in `Transform.translation.z` for sprite sorting.
pub trait DrawLayer: Component + Copy + Send + Sync + 'static {
    /// Returns the Z value for this draw layer.
    fn z(&self) -> f32;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test enum implementing `DrawLayer` to verify the trait is usable.
    #[derive(Component, Clone, Copy, Debug)]
    enum TestDrawLayer {
        A,
        B,
    }

    impl DrawLayer for TestDrawLayer {
        fn z(&self) -> f32 {
            match self {
                Self::A => 0.0,
                Self::B => 1.0,
            }
        }
    }

    #[test]
    fn draw_layer_trait_implementable_and_returns_correct_z() {
        assert!((TestDrawLayer::A.z()).abs() < f32::EPSILON);
        assert!((TestDrawLayer::B.z() - 1.0).abs() < f32::EPSILON);
    }
}
