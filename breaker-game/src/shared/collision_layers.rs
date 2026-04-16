//! Game-specific collision layer bitmasks for physics2d `CollisionLayers`.

/// Collision layer bitmask: Bolt entities.
pub const BOLT_LAYER: u32 = 1 << 0;
/// Collision layer bitmask: Cell entities.
pub const CELL_LAYER: u32 = 1 << 1;
/// Collision layer bitmask: Wall entities.
pub const WALL_LAYER: u32 = 1 << 2;
/// Collision layer bitmask: Breaker entities.
pub const BREAKER_LAYER: u32 = 1 << 3;
/// Collision layer bitmask: Salvo entities.
pub const SALVO_LAYER: u32 = 1 << 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_layer_constants_are_distinct_powers_of_two() {
        // Each layer constant is a distinct power of 2 (single bit set)
        let layers = [
            BOLT_LAYER,
            CELL_LAYER,
            WALL_LAYER,
            BREAKER_LAYER,
            SALVO_LAYER,
        ];

        // Each is a power of 2
        for &layer in &layers {
            assert!(
                layer.is_power_of_two(),
                "layer 0x{layer:02X} is not a power of 2"
            );
        }

        // All are distinct
        for (i, &a) in layers.iter().enumerate() {
            for (j, &b) in layers.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "layers at index {i} and {j} are not distinct");
                }
            }
        }

        // Specific values
        assert_eq!(BOLT_LAYER, 0x01);
        assert_eq!(CELL_LAYER, 0x02);
        assert_eq!(WALL_LAYER, 0x04);
        assert_eq!(BREAKER_LAYER, 0x08);
        assert_eq!(SALVO_LAYER, 0x10);

        // No overlap: bitwise OR of all equals bitwise sum (no shared bits)
        let combined = BOLT_LAYER | CELL_LAYER | WALL_LAYER | BREAKER_LAYER | SALVO_LAYER;
        assert_eq!(combined, 0x1F, "combined layers should be 0x1F");
    }

    #[test]
    fn salvo_layer_is_power_of_two_and_equals_0x10() {
        assert_eq!(SALVO_LAYER, 0x10, "SALVO_LAYER should be 0x10");
        assert!(
            SALVO_LAYER.is_power_of_two(),
            "SALVO_LAYER should be a power of two"
        );
    }

    #[test]
    fn salvo_layer_does_not_overlap_existing_layers() {
        let existing = BOLT_LAYER | CELL_LAYER | WALL_LAYER | BREAKER_LAYER;
        assert_eq!(
            SALVO_LAYER & existing,
            0,
            "SALVO_LAYER must not overlap existing layers"
        );
    }
}
