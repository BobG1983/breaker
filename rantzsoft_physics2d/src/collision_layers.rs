//! Bitmask-based collision layer filtering for spatial queries.

use bevy::prelude::*;

/// Bitmask pair controlling which entities can interact in spatial queries.
///
/// `membership` declares which layers this entity belongs to.
/// `mask` declares which layers this entity can see / interact with.
///
/// Filtering rule: `self.mask & other.membership != 0` means "self can see other".
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollisionLayers {
    /// Bitmask of layers this entity belongs to.
    pub membership: u32,
    /// Bitmask of layers this entity can interact with.
    pub mask: u32,
}

impl CollisionLayers {
    /// Creates a new `CollisionLayers` with the given membership and mask.
    #[must_use]
    pub const fn new(membership: u32, mask: u32) -> Self {
        Self { membership, mask }
    }

    /// Returns `true` if this entity's mask overlaps the other entity's membership.
    #[must_use]
    pub const fn interacts_with(&self, other: &Self) -> bool {
        (self.mask & other.membership) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: Default is invisible ────────────────────────

    #[test]
    fn default_has_zero_membership_and_zero_mask() {
        let layers = CollisionLayers::default();
        assert_eq!(layers.membership, 0);
        assert_eq!(layers.mask, 0);
    }

    // ── Behavior 2: new stores membership and mask ──────────────

    #[test]
    fn new_stores_membership_and_mask() {
        let layers = CollisionLayers::new(0x01, 0x06);
        assert_eq!(layers.membership, 0x01);
        assert_eq!(layers.mask, 0x06);
    }

    #[test]
    fn new_with_all_bits_set() {
        let layers = CollisionLayers::new(u32::MAX, u32::MAX);
        assert_eq!(layers.membership, u32::MAX);
        assert_eq!(layers.mask, u32::MAX);
    }

    // ── Behavior 3: interacts_with true when mask overlaps membership ──

    #[test]
    fn interacts_with_true_when_mask_overlaps_membership() {
        let a = CollisionLayers::new(0x01, 0x02);
        let b = CollisionLayers::new(0x02, 0x00);
        // A's mask 0x02 & B's membership 0x02 = 0x02 != 0
        assert!(
            a.interacts_with(&b),
            "A.mask=0x02 should overlap B.membership=0x02"
        );
    }

    #[test]
    fn interacts_with_is_asymmetric_when_only_one_masks_the_other() {
        let a = CollisionLayers::new(0x01, 0x02);
        let b = CollisionLayers::new(0x02, 0x00);
        // B's mask 0x00 & A's membership 0x01 = 0x00 => false
        assert!(
            !b.interacts_with(&a),
            "B.mask=0x00 should not overlap A.membership=0x01"
        );
    }

    // ── Behavior 4: interacts_with false when no overlap ────────

    #[test]
    fn interacts_with_false_when_no_overlap() {
        let a = CollisionLayers::new(0x01, 0x01);
        let b = CollisionLayers::new(0x02, 0x02);
        // A's mask 0x01 & B's membership 0x02 = 0x00
        assert!(
            !a.interacts_with(&b),
            "A.mask=0x01 should not overlap B.membership=0x02"
        );
    }

    #[test]
    fn interacts_with_false_both_zero_mask() {
        let a = CollisionLayers::new(0x01, 0x00);
        let b = CollisionLayers::new(0x02, 0x00);
        assert!(!a.interacts_with(&b));
        assert!(!b.interacts_with(&a));
    }

    // ── Behavior 5: symmetric when both mask each other ─────────

    #[test]
    fn interacts_with_symmetric_when_both_mask_each_other() {
        let a = CollisionLayers::new(0x01, 0x02);
        let b = CollisionLayers::new(0x02, 0x01);
        assert!(
            a.interacts_with(&b),
            "A.mask=0x02 & B.membership=0x02 should be true"
        );
        assert!(
            b.interacts_with(&a),
            "B.mask=0x01 & A.membership=0x01 should be true"
        );
    }

    #[test]
    fn interacts_with_symmetric_same_membership_and_mask() {
        let a = CollisionLayers::new(0x03, 0x03);
        let b = CollisionLayers::new(0x03, 0x03);
        assert!(a.interacts_with(&b));
        assert!(b.interacts_with(&a));
    }
}
