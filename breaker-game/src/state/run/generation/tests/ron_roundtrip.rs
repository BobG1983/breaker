//! Group G — fixture-style RON round-trip (#37–39)

use crate::state::run::{generation::*, node::definition::NodePool};

// ── Behavior #37: FrameDef deserializes from a minimal RON literal ────────────

#[test]
fn frame_def_deserializes_from_minimal_ron_literal() {
    let ron_str = r#"FrameDef(
        node_type: Passive,
        grid: (8, 6),
        fixed: [
            ((0, 0), Any),
            ((7, 5), MustInclude([Armored])),
        ],
        slots: [
            Slot(
                id: "main",
                origin: (0, 1),
                size: S8x4,
            ),
        ],
        portal: false,
    )"#;

    let frame: FrameDef = ron::de::from_str(ron_str).expect("FrameDef should deserialize");
    assert_eq!(frame.node_type, NodePool::Passive);
    assert_eq!(frame.grid, (8, 6));
    assert_eq!(frame.fixed.len(), 2);
    assert_eq!(frame.slots.len(), 1);
    assert_eq!(frame.slots[0].id, "main");
    assert_eq!(frame.slots[0].size, BlockSize::S8x4);
    assert!(!frame.portal);

    // Constraint inside struct is correct
    assert_eq!(
        frame.fixed[1].1,
        CellConstraint::MustInclude(vec![BehaviorKind::Armored]),
    );
}

// ── Behavior #38: BlockDef deserializes from a minimal RON literal ────────────

#[test]
fn block_def_deserializes_from_minimal_ron_literal() {
    let ron_str = r"BlockDef(
        size: S4x3,
        min_tier: 1,
        max_tier: 3,
        cells: [
            ((0, 0), Any),
            ((1, 0), MustNotInclude(Any)),
            ((2, 0), PlainCell),
            ((3, 0), MustInclude([Volatile])),
        ],
        sequences: [
            Sequence(
                id: 1,
                cells: [(0, 0), (3, 0)],
            ),
        ],
    )";

    let block: BlockDef = ron::de::from_str(ron_str).expect("BlockDef should deserialize");
    assert_eq!(block.size, BlockSize::S4x3);
    assert_eq!(block.min_tier, 1);
    assert_eq!(block.max_tier, 3);
    assert_eq!(block.cells.len(), 4);
    assert_eq!(block.sequences.len(), 1);

    // MustNotInclude(Any) normalizes to PlainCell
    assert_eq!(
        block.cells[1].1,
        CellConstraint::PlainCell,
        "MustNotInclude(Any) must deserialize to PlainCell",
    );

    // Bare PlainCell also deserializes to PlainCell
    assert_eq!(
        block.cells[2].1,
        CellConstraint::PlainCell,
        "PlainCell bare token must deserialize to PlainCell",
    );

    // Both shorthand and bare form are equal
    assert_eq!(
        block.cells[1].1, block.cells[2].1,
        "MustNotInclude(Any) and PlainCell must be equal after deserialization",
    );

    // MustInclude([Volatile]) correct
    assert_eq!(
        block.cells[3].1,
        CellConstraint::MustInclude(vec![BehaviorKind::Volatile]),
    );

    // Sequence correct
    assert_eq!(
        block.sequences[0],
        Sequence {
            id:    1,
            cells: vec![(0, 0), (3, 0)],
        },
    );

    // Round-trip + validate
    assert!(
        block.validate().is_ok(),
        "deserialized BlockDef must pass validate()",
    );
}

// ── Behavior #39: TierPools deserializes from full 7-tier RON literal ─────────

#[test]
fn tier_pools_deserializes_from_full_7_tier_ron_literal() {
    let ron_str = r"TierPools([
      TierModifierPool(
        tier: 1,
        modifiers: [
          (Volatile, 0.3),
          // 70% chance of no modifier (plain cell)
        ],
      ),
      TierModifierPool(
        tier: 2,
        modifiers: [
          (Volatile, 0.25),
        ],
      ),
      TierModifierPool(
        tier: 3,
        modifiers: [
          (Volatile, 0.2),
          (Sequence, 0.15),
        ],
      ),
      TierModifierPool(
        tier: 4,
        modifiers: [
          (Volatile, 0.2),
          (Sequence, 0.12),
          (Survival, 0.12),
        ],
      ),
      TierModifierPool(
        tier: 5,
        modifiers: [
          (Volatile, 0.15),
          (Sequence, 0.12),
          (Survival, 0.12),
          (Armored,  0.15),
        ],
      ),
      TierModifierPool(
        tier: 6,
        modifiers: [
          (Volatile, 0.15),
          (Sequence, 0.10),
          (Survival, 0.10),
          (Armored,  0.12),
          (Phantom,  0.10),
        ],
      ),
      TierModifierPool(
        tier: 7,
        modifiers: [
          (Volatile, 0.12),
          (Sequence, 0.10),
          (Survival, 0.10),
          (Armored,  0.12),
          (Phantom,  0.08),
          (Magnetic, 0.08),
        ],
      ),
      // Tier 8+ uses tier 7 pool with escalating weights (handled in code)
    ])";

    let pools: TierPools = ron::de::from_str(ron_str).expect("TierPools should deserialize");

    assert_eq!(pools.0.len(), 7);
    assert_eq!(pools.0[0].tier, 1);
    assert_eq!(pools.0[0].modifiers, vec![(BehaviorKind::Volatile, 0.3)],);

    assert_eq!(pools.0[6].tier, 7);
    assert_eq!(pools.0[6].modifiers.len(), 6);
    assert_eq!(
        pools.0[6].modifiers,
        vec![
            (BehaviorKind::Volatile, 0.12),
            (BehaviorKind::Sequence, 0.10),
            (BehaviorKind::Survival, 0.10),
            (BehaviorKind::Armored, 0.12),
            (BehaviorKind::Phantom, 0.08),
            (BehaviorKind::Magnetic, 0.08),
        ],
    );

    assert_eq!(pools.0[3].tier, 4);
    let tier4_sum: f32 = pools.0[3].modifiers.iter().map(|(_, w)| w).sum();
    assert!(
        tier4_sum < 1.0 + 1e-6,
        "tier 4 weight sum must be < 1.0, got {tier4_sum}",
    );

    // validate() passes for the published example
    assert!(
        pools.validate().is_ok(),
        "published 7-tier example must pass validate()",
    );
}
