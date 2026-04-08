//! Guarded behavior components and ring position utilities.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a guarded-type cell (the parent).
#[derive(Component, Debug)]
pub struct GuardedCell;

/// Permanent marker identifying a guardian cell (child protector).
#[derive(Component, Debug)]
pub struct GuardianCell;

/// Ring slot position of a guardian (0-7, clockwise from top-left).
#[derive(Component, Debug, Clone, Copy)]
pub struct GuardianSlot(pub u8);

/// Target ring position that the guardian is currently sliding toward.
#[derive(Component, Debug, Clone, Copy)]
pub struct SlideTarget(pub u8);

/// Slide speed of a guardian cell in world units per second.
#[derive(Component, Debug, Clone, Copy)]
pub struct GuardianSlideSpeed(pub f32);

/// Grid step dimensions used to compute guardian world positions from ring slot offsets.
/// `step_x = cell_width + padding_x`, `step_y = cell_height + padding_y` (from `ScaledGridDims`).
#[derive(Component, Debug, Clone, Copy)]
pub struct GuardianGridStep {
    /// Horizontal grid step: `cell_width + padding_x`.
    pub step_x: f32,
    /// Vertical grid step: `cell_height + padding_y`.
    pub step_y: f32,
}

/// Returns the (`x_offset`, `y_offset`) in `cell_height` units for a ring slot.
/// Slot 0 is top-left, proceeding clockwise to slot 7 (middle-left).
///
/// Grid layout (x goes right, y goes up):
/// ```text
/// 0  1  2      (-1,+1) (0,+1) (+1,+1)
/// 7  X  3  ->  (-1, 0) ( X  ) (+1, 0)
/// 6  5  4      (-1,-1) (0,-1) (+1,-1)
/// ```
pub(crate) fn ring_slot_offset(slot: u8) -> (f32, f32) {
    match slot {
        0 => (-1.0, 1.0),
        1 => (0.0, 1.0),
        2 => (1.0, 1.0),
        3 => (1.0, 0.0),
        4 => (1.0, -1.0),
        5 => (0.0, -1.0),
        6 => (-1.0, -1.0),
        7 => (-1.0, 0.0),
        _ => {
            debug_assert!(false, "invalid ring slot {slot}, must be 0-7");
            (0.0, 0.0)
        }
    }
}

/// Returns the `(counterclockwise, clockwise)` neighbor slots for a given ring slot.
#[cfg(test)]
pub(crate) const fn adjacent_slots(slot: u8) -> (u8, u8) {
    ((slot + 7) % 8, (slot + 1) % 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Section D: GuardedCell Component ────────────────────────────────────

    // Behavior 17: GuardedCell is a marker component with Component + Debug
    #[test]
    fn guarded_cell_debug_contains_type_name() {
        let marker = GuardedCell;
        let debug_str = format!("{marker:?}");
        assert!(
            debug_str.contains("GuardedCell"),
            "debug output should contain 'GuardedCell', got: {debug_str}"
        );
    }

    // ── Section E: GuardianCell Component ───────────────────────────────────

    // Behavior 18: GuardianCell is a marker component with Component + Debug
    #[test]
    fn guardian_cell_debug_contains_type_name() {
        let marker = GuardianCell;
        let debug_str = format!("{marker:?}");
        assert!(
            debug_str.contains("GuardianCell"),
            "debug output should contain 'GuardianCell', got: {debug_str}"
        );
    }

    // ── Section F: GuardianSlot Component ──────────────────────────────────

    // Behavior 19: GuardianSlot wraps a u8 ring position
    #[test]
    fn guardian_slot_wraps_u8() {
        let slot = GuardianSlot(3);
        assert_eq!(slot.0, 3, "GuardianSlot.0 should be 3");
    }

    #[test]
    fn guardian_slot_max_valid_slot() {
        let slot = GuardianSlot(7);
        assert_eq!(slot.0, 7, "GuardianSlot(7) should be max valid slot");
    }

    #[test]
    fn guardian_slot_min_valid_slot() {
        let slot = GuardianSlot(0);
        assert_eq!(slot.0, 0, "GuardianSlot(0) should be min valid slot");
    }

    // Behavior 20: GuardianSlot is Clone + Copy + Debug
    #[test]
    fn guardian_slot_clone_copy_debug() {
        let slot = GuardianSlot(5);
        let cloned = slot;
        assert_eq!(slot.0, cloned.0, "clone should equal original");
        let copied = slot; // Copy — implicit
        assert_eq!(slot.0, copied.0, "copy should equal original");
        let debug_str = format!("{slot:?}");
        assert!(
            debug_str.contains("GuardianSlot"),
            "debug should contain 'GuardianSlot', got: {debug_str}"
        );
        assert!(
            debug_str.contains('5'),
            "debug should contain '5', got: {debug_str}"
        );
    }

    // ── Section K: SlideTarget Component ───────────────────────────────────

    // Behavior 40 (K): SlideTarget wraps a u8
    #[test]
    fn slide_target_wraps_u8() {
        let target = SlideTarget(5);
        assert_eq!(target.0, 5, "SlideTarget.0 should be 5");
    }

    // Behavior 41 (K): SlideTarget is Component + Debug + Clone + Copy
    #[test]
    fn slide_target_debug_clone_copy() {
        let target = SlideTarget(2);
        let debug_str = format!("{target:?}");
        assert!(
            debug_str.contains("SlideTarget"),
            "debug should contain 'SlideTarget', got: {debug_str}"
        );
        assert!(
            debug_str.contains('2'),
            "debug should contain '2', got: {debug_str}"
        );
        let cloned = target;
        assert_eq!(target.0, cloned.0, "clone should equal original");
        let copied = target;
        assert_eq!(target.0, copied.0, "copy should equal original");
    }

    // ── Section L: GuardianSlideSpeed Component ────────────────────────────

    // Behavior 42: GuardianSlideSpeed wraps slide speed value
    #[test]
    fn guardian_slide_speed_wraps_f32() {
        let speed = GuardianSlideSpeed(30.0);
        assert!(
            (speed.0 - 30.0).abs() < f32::EPSILON,
            "GuardianSlideSpeed.0 should be 30.0"
        );
    }

    #[test]
    fn guardian_slide_speed_zero_is_valid() {
        let speed = GuardianSlideSpeed(0.0);
        assert!(
            (speed.0 - 0.0).abs() < f32::EPSILON,
            "GuardianSlideSpeed(0.0) should be valid (stationary)"
        );
    }

    // ── Section O: GuardianGridStep Component ──────────────────────────────

    // Behavior 48: GuardianGridStep stores step dimensions
    #[test]
    fn guardian_grid_step_stores_dimensions() {
        let step = GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        };
        assert!(
            (step.step_x - 72.0).abs() < f32::EPSILON,
            "step_x should be 72.0"
        );
        assert!(
            (step.step_y - 26.0).abs() < f32::EPSILON,
            "step_y should be 26.0"
        );
    }

    #[test]
    fn guardian_grid_step_degenerate_zero() {
        let step = GuardianGridStep {
            step_x: 0.0,
            step_y: 0.0,
        };
        assert!(
            (step.step_x - 0.0).abs() < f32::EPSILON,
            "degenerate step_x should be 0.0"
        );
        assert!(
            (step.step_y - 0.0).abs() < f32::EPSILON,
            "degenerate step_y should be 0.0"
        );
    }

    // Behavior 49: GuardianGridStep is Component + Debug + Clone + Copy
    #[test]
    fn guardian_grid_step_clone_copy_debug() {
        let step = GuardianGridStep {
            step_x: 72.0,
            step_y: 26.0,
        };
        let cloned = step;
        assert!(
            (step.step_x - cloned.step_x).abs() < f32::EPSILON
                && (step.step_y - cloned.step_y).abs() < f32::EPSILON,
            "clone should equal original"
        );
        let copied = step;
        assert!(
            (step.step_x - copied.step_x).abs() < f32::EPSILON
                && (step.step_y - copied.step_y).abs() < f32::EPSILON,
            "copy should equal original"
        );
        let debug_str = format!("{step:?}");
        assert!(
            debug_str.contains("GuardianGridStep"),
            "debug should contain 'GuardianGridStep', got: {debug_str}"
        );
        assert!(
            debug_str.contains("72.0") || debug_str.contains("72"),
            "debug should contain '72.0', got: {debug_str}"
        );
        assert!(
            debug_str.contains("26.0") || debug_str.contains("26"),
            "debug should contain '26.0', got: {debug_str}"
        );
    }

    // ── Section G: Ring Position Utilities ──────────────────────────────────

    // Behavior 21: ring_slot_offset maps slot 0 to (-1.0, 1.0) top-left
    #[test]
    fn ring_slot_offset_slot_0_is_top_left() {
        let (x, y) = ring_slot_offset(0);
        assert!(
            (x - (-1.0)).abs() < f32::EPSILON && (y - 1.0).abs() < f32::EPSILON,
            "slot 0 should be (-1.0, 1.0), got ({x}, {y})"
        );
    }

    // Behavior 22: ring_slot_offset maps all 8 slots correctly
    #[test]
    fn ring_slot_offset_all_8_slots() {
        let expected: [(f32, f32); 8] = [
            (-1.0, 1.0),  // 0: top-left
            (0.0, 1.0),   // 1: top-center
            (1.0, 1.0),   // 2: top-right
            (1.0, 0.0),   // 3: middle-right
            (1.0, -1.0),  // 4: bottom-right
            (0.0, -1.0),  // 5: bottom-center
            (-1.0, -1.0), // 6: bottom-left
            (-1.0, 0.0),  // 7: middle-left
        ];
        for (slot, (ex, ey)) in expected.iter().enumerate() {
            let (x, y) = ring_slot_offset(u8::try_from(slot).expect("slot fits u8"));
            assert!(
                (x - ex).abs() < f32::EPSILON && (y - ey).abs() < f32::EPSILON,
                "slot {slot} should be ({ex}, {ey}), got ({x}, {y})"
            );
        }
    }

    // Behavior 23: ring_slot_offset out-of-range returns (0.0, 0.0) in release
    // NOTE: We cannot easily test the debug_assert panic in a unit test without
    // cfg(debug_assertions) branching. We test the release fallback here.
    #[test]
    #[cfg(not(debug_assertions))]
    fn ring_slot_offset_out_of_range_returns_zero_in_release() {
        let (x, y) = ring_slot_offset(8);
        assert!(
            (x - 0.0).abs() < f32::EPSILON && (y - 0.0).abs() < f32::EPSILON,
            "slot 8 should return (0.0, 0.0) in release, got ({x}, {y})"
        );
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn ring_slot_offset_u8_max_returns_zero_in_release() {
        let (x, y) = ring_slot_offset(255);
        assert!(
            (x - 0.0).abs() < f32::EPSILON && (y - 0.0).abs() < f32::EPSILON,
            "slot 255 should return (0.0, 0.0) in release, got ({x}, {y})"
        );
    }

    // Behavior 24: adjacent_slots returns (counterclockwise, clockwise) neighbors
    #[test]
    fn adjacent_slots_slot_0_returns_7_and_1() {
        let (ccw, cw) = adjacent_slots(0);
        assert_eq!(ccw, 7, "slot 0 ccw neighbor should be 7, got {ccw}");
        assert_eq!(cw, 1, "slot 0 cw neighbor should be 1, got {cw}");
    }

    #[test]
    fn adjacent_slots_slot_7_wraps_to_6_and_0() {
        let (ccw, cw) = adjacent_slots(7);
        assert_eq!(ccw, 6, "slot 7 ccw neighbor should be 6, got {ccw}");
        assert_eq!(cw, 0, "slot 7 cw neighbor should be 0, got {cw}");
    }

    #[test]
    fn adjacent_slots_slot_4_returns_3_and_5() {
        let (ccw, cw) = adjacent_slots(4);
        assert_eq!(ccw, 3, "slot 4 ccw neighbor should be 3, got {ccw}");
        assert_eq!(cw, 5, "slot 4 cw neighbor should be 5, got {cw}");
    }

    #[test]
    fn adjacent_slots_all_8_slots_use_modular_arithmetic() {
        for slot in 0u8..8 {
            let (ccw, cw) = adjacent_slots(slot);
            assert_eq!(
                ccw,
                (slot + 7) % 8,
                "slot {slot} ccw: expected {}, got {ccw}",
                (slot + 7) % 8
            );
            assert_eq!(
                cw,
                (slot + 1) % 8,
                "slot {slot} cw: expected {}, got {cw}",
                (slot + 1) % 8
            );
        }
    }
}
