//! Group D — struct shapes & derives (#16–21)

use crate::state::run::{generation::*, node::definition::NodePool};

// ── Behavior #16: Slot struct ────────────────────────────────────────────────

#[test]
fn slot_has_documented_fields_and_derives() {
    let slot = Slot {
        id:     "center".to_owned(),
        origin: (4, 2),
        size:   BlockSize::S8x6,
    };
    assert_eq!(slot.id, "center");
    assert_eq!(slot.origin, (4, 2));
    assert_eq!(slot.size, BlockSize::S8x6);

    // PartialEq
    let slot2 = slot.clone();
    assert_eq!(slot, slot2);

    // Empty id is structurally valid
    let _empty = Slot {
        id:     String::new(),
        origin: (0, 0),
        size:   BlockSize::S4x3,
    };
}

// ── Behavior #17: Sequence struct ────────────────────────────────────────────

#[test]
fn sequence_has_documented_fields_and_derives() {
    let seq = Sequence {
        id:    1,
        cells: vec![(1, 0), (2, 0), (3, 0), (4, 0)],
    };
    assert_eq!(seq.id, 1);
    assert_eq!(seq.cells, vec![(1, 0), (2, 0), (3, 0), (4, 0)]);

    let seq2 = seq.clone();
    assert_eq!(seq, seq2);

    // Empty cells list is structurally valid
    let _empty = Sequence {
        id:    0,
        cells: vec![],
    };
}

// ── Behavior #18: FrameDef struct ────────────────────────────────────────────

#[test]
fn frame_def_has_documented_fields_and_derives() {
    let frame = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![
            ((0, 0), CellConstraint::Any),
            (
                (0, 3),
                CellConstraint::MustInclude(vec![BehaviorKind::Armored]),
            ),
        ],
        slots:     vec![Slot {
            id:     "main".to_owned(),
            origin: (0, 1),
            size:   BlockSize::S8x4,
        }],
        portal:    false,
    };
    assert_eq!(frame.node_type, NodePool::Passive);
    assert_eq!(frame.grid, (8, 6));
    assert_eq!(frame.fixed.len(), 2);
    assert_eq!(frame.slots.len(), 1);
    assert!(!frame.portal);

    // portal: true is structurally valid
    let portal_frame = FrameDef {
        node_type: NodePool::Passive,
        grid:      (4, 4),
        fixed:     vec![],
        slots:     vec![],
        portal:    true,
    };
    assert!(portal_frame.portal);

    // empty fixed and slots lists are structurally valid
    let _empty_frame = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![],
        slots:     vec![],
        portal:    false,
    };
}

// ── Behavior #19: BlockDef struct ────────────────────────────────────────────

#[test]
fn block_def_has_documented_fields_and_derives() {
    let block = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![
            ((0, 0), CellConstraint::Any),
            (
                (1, 0),
                CellConstraint::MustInclude(vec![BehaviorKind::Volatile]),
            ),
        ],
        sequences: vec![Sequence {
            id:    1,
            cells: vec![(0, 0), (1, 0)],
        }],
    };
    assert_eq!(block.size, BlockSize::S4x3);
    assert_eq!(block.min_tier, 1);
    assert_eq!(block.max_tier, 3);
    assert_eq!(block.cells.len(), 2);
    assert_eq!(block.sequences.len(), 1);

    // single-tier block
    let single_tier = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  5,
        max_tier:  5,
        cells:     vec![],
        sequences: vec![],
    };
    assert_eq!(single_tier.min_tier, 5);
    assert_eq!(single_tier.max_tier, 5);
}

// ── Behavior #20: TierModifierPool struct ────────────────────────────────────

#[test]
fn tier_modifier_pool_has_documented_fields_and_derives() {
    let pool = TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, 0.3)],
    };
    assert_eq!(pool.tier, 1);
    assert_eq!(pool.modifiers, vec![(BehaviorKind::Volatile, 0.3)]);

    // tier-7 example from format-block.md
    let tier7 = TierModifierPool {
        tier:      7,
        modifiers: vec![
            (BehaviorKind::Volatile, 0.12),
            (BehaviorKind::Sequence, 0.10),
            (BehaviorKind::Survival, 0.10),
            (BehaviorKind::Armored, 0.12),
            (BehaviorKind::Phantom, 0.08),
            (BehaviorKind::Magnetic, 0.08),
        ],
    };
    assert_eq!(tier7.tier, 7);
    assert_eq!(tier7.modifiers.len(), 6);

    // empty modifiers (plain-cell-only tier) is structurally valid
    let empty = TierModifierPool {
        tier:      2,
        modifiers: vec![],
    };
    assert_eq!(empty.modifiers.len(), 0);
}

// ── Behavior #21: TierPools tuple struct ─────────────────────────────────────

#[test]
fn tier_pools_tuple_struct_has_documented_field_and_derives() {
    let pools = TierPools(vec![
        TierModifierPool {
            tier:      1,
            modifiers: vec![],
        },
        TierModifierPool {
            tier:      2,
            modifiers: vec![],
        },
    ]);
    assert_eq!(pools.0.len(), 2);
    assert_eq!(pools.0[0].tier, 1);
    assert_eq!(pools.0[1].tier, 2);

    // .0 is pub — accessible
    let _ = &pools.0;

    // empty TierPools is structurally valid
    let empty = TierPools(vec![]);
    assert_eq!(empty.0.len(), 0);
}
