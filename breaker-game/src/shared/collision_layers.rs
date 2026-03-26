//! Game-specific collision layer bitmasks for physics2d `CollisionLayers`.

/// Collision layer bitmask: Bolt entities.
pub const BOLT_LAYER: u32 = 1 << 0;
/// Collision layer bitmask: Cell entities.
pub const CELL_LAYER: u32 = 1 << 1;
/// Collision layer bitmask: Wall entities.
pub const WALL_LAYER: u32 = 1 << 2;
/// Collision layer bitmask: Breaker entities.
pub const BREAKER_LAYER: u32 = 1 << 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_layer_constants_are_distinct_powers_of_two() {
        // Each layer constant is a distinct power of 2 (single bit set)
        let layers = [BOLT_LAYER, CELL_LAYER, WALL_LAYER, BREAKER_LAYER];

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

        // No overlap: bitwise OR of all equals bitwise sum (no shared bits)
        let combined = BOLT_LAYER | CELL_LAYER | WALL_LAYER | BREAKER_LAYER;
        assert_eq!(combined, 0x0F, "combined layers should be 0x0F");
    }
}
