//! System to increment `ChipSelectCount` on each chip-select entry.

use bevy::prelude::*;

use crate::shared::rng::ChipSelectCount;

/// Increments `ChipSelectCount` by 1 (wrapping) on `OnEnter(ChipSelectState::Selecting)`,
/// after `generate_chip_offerings`.
pub(crate) fn tick_chip_select_count(mut count: ResMut<ChipSelectCount>) {
    count.0 = count.0.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn test_app_with_count(initial: u32) -> App {
        let mut app = TestAppBuilder::new()
            .with_resource::<ChipSelectCount>()
            .with_system(Update, tick_chip_select_count)
            .build();
        app.insert_resource(ChipSelectCount(initial));
        app
    }

    // ── Behavior 4 — increments from 0 to 1 (sanity) ────────────────────────

    #[test]
    fn tick_chip_select_count_increments_from_zero_to_one() {
        let mut app = test_app_with_count(0);
        app.update();

        assert_eq!(
            app.world().resource::<ChipSelectCount>().0,
            1,
            "tick_chip_select_count must increment ChipSelectCount(0) to 1"
        );
    }

    #[test]
    fn tick_chip_select_count_is_monotonically_increasing() {
        let mut app = test_app_with_count(0);
        app.update();
        assert_eq!(app.world().resource::<ChipSelectCount>().0, 1);
        app.update();
        assert_eq!(app.world().resource::<ChipSelectCount>().0, 2);
        app.update();
        assert_eq!(
            app.world().resource::<ChipSelectCount>().0,
            3,
            "tick_chip_select_count must increment monotonically (0→1→2→3)"
        );
    }

    // ── Behavior 5 — increments from 1 to 2 (subsequent visits) ─────────────

    #[test]
    fn tick_chip_select_count_increments_from_one_to_two() {
        let mut app = test_app_with_count(1);
        app.update();

        assert_eq!(
            app.world().resource::<ChipSelectCount>().0,
            2,
            "tick_chip_select_count must increment ChipSelectCount(1) to 2"
        );
    }

    #[test]
    fn tick_chip_select_count_overflow_does_not_panic() {
        // u32::MAX chip-selects: primary contract is no panic.
        let mut app = test_app_with_count(u32::MAX);
        app.update(); // must not panic

        let result = app.world().resource::<ChipSelectCount>().0;
        // The result must be either 0 (wrapping_add) or u32::MAX (saturating_add).
        assert!(
            result == 0 || result == u32::MAX,
            "tick_chip_select_count from u32::MAX must produce 0 (wrap) or u32::MAX (saturate), got {result}"
        );
    }
}
