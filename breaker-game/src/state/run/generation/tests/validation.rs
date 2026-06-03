//! Group E — validation helpers (#22–35)

use crate::state::run::{generation::*, node::definition::NodePool};

// ── Behavior #22: BlockDef::validate() accepts a well-formed block ────────────

#[test]
fn block_def_validate_accepts_well_formed_block() {
    let block = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![
            ((0, 0), CellConstraint::Any),
            ((1, 0), CellConstraint::Any),
            ((2, 0), CellConstraint::Any),
            ((3, 0), CellConstraint::Any),
        ],
        sequences: vec![],
    };
    assert!(
        block.validate().is_ok(),
        "well-formed block must validate Ok"
    );

    // With a valid sequence (subset of cells)
    let with_seq = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![
            ((0, 0), CellConstraint::Any),
            ((1, 0), CellConstraint::Any),
            ((2, 0), CellConstraint::Any),
            ((3, 0), CellConstraint::Any),
        ],
        sequences: vec![Sequence {
            id:    1,
            cells: vec![(0, 0), (1, 0)],
        }],
    };
    assert!(
        with_seq.validate().is_ok(),
        "block with valid sequence must validate Ok"
    );

    // Empty cells and sequences is structurally valid
    let empty = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![],
        sequences: vec![],
    };
    assert!(
        empty.validate().is_ok(),
        "empty cells/sequences must not be rejected by validator",
    );

    // Zero-zero tier range (Default-constructible boundary) is accepted
    let zero_tier = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  0,
        max_tier:  0,
        cells:     vec![],
        sequences: vec![],
    };
    assert!(
        zero_tier.validate().is_ok(),
        "min_tier=0, max_tier=0 must be accepted",
    );
}

// ── Behavior #23: BlockDef::validate() rejects cell position outside bounds ───

#[test]
fn block_def_validate_rejects_cell_outside_grid_bounds() {
    // col=4 exceeds max col=3 in S4x3
    let col_oob = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![((4, 0), CellConstraint::Any)],
        sequences: vec![],
    };
    let err = col_oob
        .validate()
        .expect_err("col=4 in S4x3 (max 3) must be rejected");
    assert!(
        err.contains("bounds"),
        "error must mention 'bounds', got: {err}",
    );

    // row=3 exceeds max row=2 in S4x3
    let row_oob = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![((0, 3), CellConstraint::Any)],
        sequences: vec![],
    };
    let err2 = row_oob
        .validate()
        .expect_err("row=3 in S4x3 (max 2) must be rejected");
    assert!(
        err2.contains("bounds"),
        "error must mention 'bounds', got: {err2}",
    );

    // Maximum legal corner (3, 2) in S4x3 is accepted
    let corner = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![((3, 2), CellConstraint::Any)],
        sequences: vec![],
    };
    assert!(
        corner.validate().is_ok(),
        "corner (3, 2) must be accepted in S4x3"
    );
}

// ── Behavior #24: BlockDef::validate() rejects duplicate cell positions ────────

#[test]
fn block_def_validate_rejects_duplicate_cell_positions() {
    let dup = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![
            ((0, 0), CellConstraint::Any),
            (
                (0, 0),
                CellConstraint::MustInclude(vec![BehaviorKind::Volatile]),
            ),
        ],
        sequences: vec![],
    };
    let err = dup
        .validate()
        .expect_err("duplicate position must be rejected");
    assert!(
        err.contains("duplicate"),
        "error must mention 'duplicate', got: {err}",
    );

    // Distinct positions do NOT trigger duplicate detection
    let distinct = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![((0, 0), CellConstraint::Any), ((1, 0), CellConstraint::Any)],
        sequences: vec![],
    };
    assert!(
        distinct.validate().is_ok(),
        "distinct positions must not trigger duplicate check"
    );
}

// ── Behavior #25: BlockDef::validate() rejects sequence cells not in cells list

#[test]
fn block_def_validate_rejects_sequence_cells_missing_from_cells_list() {
    // (2, 0) is in the sequence but not in cells
    let missing_seq_cell = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  1,
        max_tier:  3,
        cells:     vec![((0, 0), CellConstraint::Any), ((1, 0), CellConstraint::Any)],
        sequences: vec![Sequence {
            id:    1,
            cells: vec![(0, 0), (2, 0)],
        }],
    };
    let err = missing_seq_cell
        .validate()
        .expect_err("sequence referencing absent position must be rejected");
    assert!(
        err.contains("sequence"),
        "error must mention 'sequence', got: {err}",
    );
    // (2, 0) mentioned in error message
    assert!(
        err.contains("2, 0") || err.contains("(2, 0)"),
        "error must mention the missing position, got: {err}",
    );
}

// ── Behavior #26: BlockDef::validate() rejects min_tier > max_tier ───────────

#[test]
fn block_def_validate_rejects_inverted_tier_range() {
    let inverted = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  5,
        max_tier:  3,
        cells:     vec![((0, 0), CellConstraint::Any)],
        sequences: vec![],
    };
    let err = inverted
        .validate()
        .expect_err("min_tier > max_tier must be rejected");
    assert!(
        err.contains("tier"),
        "error must mention 'tier', got: {err}",
    );
    assert!(
        err.contains("min") || err.contains("range"),
        "error must mention 'min' or 'range', got: {err}",
    );

    // min_tier == max_tier is valid (single-tier block)
    let equal = BlockDef {
        size:      BlockSize::S4x3,
        min_tier:  5,
        max_tier:  5,
        cells:     vec![((0, 0), CellConstraint::Any)],
        sequences: vec![],
    };
    assert!(
        equal.validate().is_ok(),
        "min_tier == max_tier must be accepted"
    );
}

// ── Behavior #27: FrameDef::validate() accepts a well-formed frame ────────────

#[test]
fn frame_def_validate_accepts_well_formed_frame() {
    let frame = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![
            ((0, 0), CellConstraint::Any),
            ((7, 0), CellConstraint::Any),
            ((0, 5), CellConstraint::Any),
            ((7, 5), CellConstraint::Any),
        ],
        slots:     vec![Slot {
            id:     "main".to_owned(),
            origin: (1, 1),
            size:   BlockSize::S6x4,
        }],
        portal:    false,
    };
    assert!(
        frame.validate().is_ok(),
        "well-formed frame must validate Ok"
    );

    // Empty slots and fixed is Ok
    let empty = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![],
        slots:     vec![],
        portal:    false,
    };
    assert!(empty.validate().is_ok(), "empty frame must validate Ok");
}

// ── Behavior #28: FrameDef::validate() rejects fixed cell outside grid ────────

#[test]
fn frame_def_validate_rejects_fixed_cell_outside_grid() {
    // col 8 exceeds max 7 in (8, 6) grid
    let col_oob = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((8, 0), CellConstraint::Any)],
        slots:     vec![],
        portal:    false,
    };
    let err = col_oob
        .validate()
        .expect_err("col=8 in (8, 6) grid must be rejected");
    assert!(
        err.contains("bounds"),
        "error must mention 'bounds', got: {err}"
    );

    // row 6 exceeds max 5
    let row_oob = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((0, 6), CellConstraint::Any)],
        slots:     vec![],
        portal:    false,
    };
    let err2 = row_oob
        .validate()
        .expect_err("row=6 in (8, 6) grid must be rejected");
    assert!(
        err2.contains("bounds"),
        "error must mention 'bounds', got: {err2}"
    );

    // Maximum legal corner (7, 5) is accepted
    let corner = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((7, 5), CellConstraint::Any)],
        slots:     vec![],
        portal:    false,
    };
    assert!(
        corner.validate().is_ok(),
        "corner (7, 5) must be accepted in (8, 6) grid"
    );
}

// ── Behavior #29: FrameDef::validate() rejects slot extending outside grid ────

#[test]
fn frame_def_validate_rejects_slot_outside_grid() {
    // S6x4 at origin (3, 3): covers cols 3..=8, rows 3..=6 — both exceed (8, 6)
    let oob = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![],
        slots:     vec![Slot {
            id:     "oversize".to_owned(),
            origin: (3, 3),
            size:   BlockSize::S6x4,
        }],
        portal:    false,
    };
    let err = oob
        .validate()
        .expect_err("slot extending outside grid must be rejected");
    assert!(
        err.contains("bounds") || err.contains("slot"),
        "error must mention 'bounds' or 'slot', got: {err}",
    );

    // S6x4 at origin (2, 2) in (8, 6): covers cols 2..=7, rows 2..=5 — fits exactly
    let fits = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![],
        slots:     vec![Slot {
            id:     "edge".to_owned(),
            origin: (2, 2),
            size:   BlockSize::S6x4,
        }],
        portal:    false,
    };
    assert!(
        fits.validate().is_ok(),
        "slot fitting exactly must be accepted"
    );
}

// ── Behavior #30: FrameDef::validate() rejects overlap between fixed and slot ─

#[test]
fn frame_def_validate_rejects_fixed_cell_overlapping_slot() {
    // Fixed at (3, 3); S6x4 at origin (1, 1) covers cols 1..=6, rows 1..=4
    let overlap = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((3, 3), CellConstraint::Any)],
        slots:     vec![Slot {
            id:     "overlap".to_owned(),
            origin: (1, 1),
            size:   BlockSize::S6x4,
        }],
        portal:    false,
    };
    let err = overlap
        .validate()
        .expect_err("fixed cell inside slot must be rejected");
    assert!(
        err.contains("overlap") || (err.contains("3, 3") && err.contains("slot")),
        "error must mention 'overlap' or '(3, 3)' + 'slot', got: {err}",
    );

    // Fixed cell at (0, 0) — outside the same slot rectangle — is ok
    let no_overlap = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((0, 0), CellConstraint::Any)],
        slots:     vec![Slot {
            id:     "overlap".to_owned(),
            origin: (1, 1),
            size:   BlockSize::S6x4,
        }],
        portal:    false,
    };
    assert!(
        no_overlap.validate().is_ok(),
        "fixed cell outside slot must not trigger overlap",
    );
}

// ── Behavior #31: FrameDef::validate() rejects overlap between two slots ──────

#[test]
fn frame_def_validate_rejects_overlapping_slots() {
    // slot a: S8x4 at (0, 0) → cols 0..=7, rows 0..=3
    // slot b: S8x4 at (4, 0) → cols 4..=11, rows 0..=3 — overlaps on cols 4..=7
    let overlap = FrameDef {
        node_type: NodePool::Passive,
        grid:      (16, 8),
        fixed:     vec![],
        slots:     vec![
            Slot {
                id:     "a".to_owned(),
                origin: (0, 0),
                size:   BlockSize::S8x4,
            },
            Slot {
                id:     "b".to_owned(),
                origin: (4, 0),
                size:   BlockSize::S8x4,
            },
        ],
        portal:    false,
    };
    let err = overlap
        .validate()
        .expect_err("overlapping slots must be rejected");
    assert!(
        err.contains("overlap") || err.contains("slot"),
        "error must mention 'overlap' or 'slot', got: {err}",
    );

    // Edge-adjacent slots (touching but not overlapping) are ok
    // slot a: S8x4 at (0, 0) → cols 0..=7
    // slot b: S8x4 at (8, 0) → cols 8..=15
    let adjacent = FrameDef {
        node_type: NodePool::Passive,
        grid:      (16, 8),
        fixed:     vec![],
        slots:     vec![
            Slot {
                id:     "a".to_owned(),
                origin: (0, 0),
                size:   BlockSize::S8x4,
            },
            Slot {
                id:     "b".to_owned(),
                origin: (8, 0),
                size:   BlockSize::S8x4,
            },
        ],
        portal:    false,
    };
    assert!(
        adjacent.validate().is_ok(),
        "edge-adjacent (non-overlapping) slots must be accepted"
    );
}

// ── Behavior #32: FrameDef::validate() rejects portal frame with MustInclude([Portal]) ──

#[test]
fn frame_def_validate_rejects_portal_frame_with_must_include_portal() {
    // Portal frame + MustInclude([Portal]) in fixed → rejected
    let violating = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustInclude(vec![BehaviorKind::Portal]),
        )],
        slots:     vec![],
        portal:    true,
    };
    let err = violating
        .validate()
        .expect_err("portal frame with MustInclude([Portal]) must be rejected");
    assert!(
        err.to_lowercase().contains("portal"),
        "error must mention 'portal', got: {err}",
    );

    // Same constraint but portal: false is accepted
    let non_portal = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustInclude(vec![BehaviorKind::Portal]),
        )],
        slots:     vec![],
        portal:    false,
    };
    assert!(
        non_portal.validate().is_ok(),
        "non-portal frame with MustInclude([Portal]) must be accepted",
    );

    // Portal frame with MustInclude([Armored]) (no Portal) is accepted
    let armored_ok = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustInclude(vec![BehaviorKind::Armored]),
        )],
        slots:     vec![],
        portal:    true,
    };
    assert!(
        armored_ok.validate().is_ok(),
        "portal frame with non-Portal MustInclude must be accepted",
    );

    // Portal frame with MustNotInclude([Portal]) is accepted
    let must_not = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustNotInclude(vec![BehaviorKind::Portal]),
        )],
        slots:     vec![],
        portal:    true,
    };
    assert!(
        must_not.validate().is_ok(),
        "portal frame with MustNotInclude([Portal]) must be accepted",
    );

    // Portal in any position of MustInclude vec triggers rejection
    let portal_with_armored = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustInclude(vec![BehaviorKind::Armored, BehaviorKind::Portal]),
        )],
        slots:     vec![],
        portal:    true,
    };
    let err2 = portal_with_armored
        .validate()
        .expect_err("Portal anywhere in MustInclude vec must trigger rejection");
    assert!(
        err2.to_lowercase().contains("portal"),
        "error must mention 'portal', got: {err2}",
    );

    let portal_first = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![(
            (1, 1),
            CellConstraint::MustInclude(vec![BehaviorKind::Portal, BehaviorKind::Armored]),
        )],
        slots:     vec![],
        portal:    true,
    };
    let err3 = portal_first
        .validate()
        .expect_err("Portal at any position in MustInclude must trigger rejection");
    assert!(
        err3.to_lowercase().contains("portal"),
        "error must mention 'portal', got: {err3}",
    );
}

#[test]
fn frame_def_validate_accepts_portal_frame_with_plain_cell_in_fixed_cells() {
    // PlainCell in fixed does not trigger the portal-exclusivity check
    let plain_ok = FrameDef {
        node_type: NodePool::Passive,
        grid:      (8, 6),
        fixed:     vec![((1, 1), CellConstraint::PlainCell)],
        slots:     vec![],
        portal:    true,
    };
    assert!(
        plain_ok.validate().is_ok(),
        "portal frame with PlainCell in fixed must be accepted",
    );
}

// ── Behavior #33: TierPools::validate() accepts a well-formed pool list ────────

#[test]
fn tier_pools_validate_accepts_well_formed_pool_list() {
    let pools = TierPools(vec![
        TierModifierPool {
            tier:      1,
            modifiers: vec![(BehaviorKind::Volatile, 0.3)],
        },
        TierModifierPool {
            tier:      2,
            modifiers: vec![(BehaviorKind::Volatile, 0.25)],
        },
        TierModifierPool {
            tier:      3,
            modifiers: vec![
                (BehaviorKind::Volatile, 0.2),
                (BehaviorKind::Sequence, 0.15),
            ],
        },
    ]);
    assert!(
        pools.validate().is_ok(),
        "well-formed pool list must validate Ok"
    );

    // Empty pool list validates Ok
    let empty = TierPools(vec![]);
    assert!(empty.validate().is_ok(), "empty TierPools must validate Ok");
}

// ── Behavior #34: TierPools::validate() rejects duplicate tier indices ─────────

#[test]
fn tier_pools_validate_rejects_duplicate_tier_indices() {
    let dup = TierPools(vec![
        TierModifierPool {
            tier:      1,
            modifiers: vec![],
        },
        TierModifierPool {
            tier:      1,
            modifiers: vec![],
        },
    ]);
    let err = dup
        .validate()
        .expect_err("duplicate tier index must be rejected");
    assert!(
        err.contains("duplicate"),
        "error must mention 'duplicate', got: {err}",
    );
    assert!(
        err.contains("tier") || err.contains('1'),
        "error must mention 'tier' or '1', got: {err}",
    );

    // Distinct tiers are accepted
    let distinct = TierPools(vec![
        TierModifierPool {
            tier:      1,
            modifiers: vec![],
        },
        TierModifierPool {
            tier:      2,
            modifiers: vec![],
        },
        TierModifierPool {
            tier:      3,
            modifiers: vec![],
        },
    ]);
    assert!(
        distinct.validate().is_ok(),
        "distinct tier indices must be accepted"
    );
}

// ── Behavior #35: TierPools::validate() rejects weights summing above 1.0 ──────

#[test]
fn tier_pools_validate_rejects_weight_sum_above_one() {
    // sum = 1.2 (0.6 + 0.6)
    let over_limit = TierPools(vec![TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, 0.6), (BehaviorKind::Sequence, 0.6)],
    }]);
    let err = over_limit
        .validate()
        .expect_err("weight sum > 1.0 must be rejected");
    assert!(
        err.contains("weight")
            || err.contains("sum")
            || (err.contains("1.0") && err.contains("tier")),
        "error must mention 'weight' or 'sum' or '1.0' + 'tier', got: {err}",
    );

    // sum exactly 1.0 validates Ok (boundary inclusive)
    let exact_one = TierPools(vec![TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, 0.5), (BehaviorKind::Sequence, 0.5)],
    }]);
    assert!(
        exact_one.validate().is_ok(),
        "sum exactly 1.0 must validate Ok (boundary inclusive)",
    );

    // sum well below 1.0 validates Ok
    let low = TierPools(vec![TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, 0.3)],
    }]);
    assert!(low.validate().is_ok(), "sum below 1.0 must validate Ok");
}

// ── Regression: TierPools::validate() rejects negative weights ───────────────

#[test]
fn tier_pools_validate_rejects_negative_weight() {
    let negative = TierPools(vec![TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, -0.1)],
    }]);
    let err = negative
        .validate()
        .expect_err("negative weight -0.1 must be rejected");
    assert!(
        err.contains("weight") || err.contains("negative") || err.contains('-'),
        "error must mention 'weight', 'negative', or '-', got: {err}",
    );

    // Zero weight is a boundary — accepted (no modifier assigned, effectively plain)
    let zero_weight = TierPools(vec![TierModifierPool {
        tier:      1,
        modifiers: vec![(BehaviorKind::Volatile, 0.0)],
    }]);
    assert!(
        zero_weight.validate().is_ok(),
        "weight of exactly 0.0 must be accepted",
    );
}
