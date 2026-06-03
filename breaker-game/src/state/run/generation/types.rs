#[cfg(test)]
use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::constraint::{BehaviorKind, CellConstraint};
use crate::state::run::node::definition::NodePool;

/// Grid size of a block template — encodes cols × rows.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BlockSize {
    S4x3,
    S4x4,
    S6x4,
    S6x6,
    S8x4,
    S8x6,
    S10x5,
    S10x8,
}

impl BlockSize {
    pub(crate) const ALL: &'static [Self] = &[
        Self::S4x3,
        Self::S4x4,
        Self::S6x4,
        Self::S6x6,
        Self::S8x4,
        Self::S8x6,
        Self::S10x5,
        Self::S10x8,
    ];

    #[must_use]
    pub(crate) const fn cols(self) -> u32 {
        match self {
            Self::S4x3 | Self::S4x4 => 4,
            Self::S6x4 | Self::S6x6 => 6,
            Self::S8x4 | Self::S8x6 => 8,
            Self::S10x5 | Self::S10x8 => 10,
        }
    }

    #[must_use]
    pub(crate) const fn rows(self) -> u32 {
        match self {
            Self::S4x3 => 3,
            Self::S4x4 | Self::S6x4 | Self::S8x4 => 4,
            Self::S10x5 => 5,
            Self::S6x6 | Self::S8x6 => 6,
            Self::S10x8 => 8,
        }
    }
}

/// A named placement slot within a `FrameDef` — defines a rectangular region
/// into which a `BlockDef` of matching size is placed.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Slot {
    pub id:     String,
    pub origin: (u32, u32),
    pub size:   BlockSize,
}

/// An ordered sequence of cell positions within a `BlockDef` — sequence
/// cells activate in index order (position 0 first, then 1, etc.).
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Sequence {
    pub id:    u32,
    pub cells: Vec<(u32, u32)>,
}

/// Frame (top-level layout template) loaded from a RON asset.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct FrameDef {
    pub node_type: NodePool,
    pub grid:      (u32, u32),
    pub fixed:     Vec<((u32, u32), CellConstraint)>,
    pub slots:     Vec<Slot>,
    pub portal:    bool,
}

impl FrameDef {
    /// Validates frame structure: grid bounds for fixed cells, slot rectangles fit
    /// within grid, no fixed/slot or slot/slot overlap, and portal frames may not
    /// require `Portal` via `MustInclude` in fixed cells.
    ///
    /// Currently gated `#[cfg(test)]` — Wave 2 will lift this gate when registry
    /// `seed()` callers wire production validation.
    #[cfg(test)]
    pub(crate) fn validate(&self) -> Result<(), String> {
        let (grid_cols, grid_rows) = self.grid;

        for ((col, row), _) in &self.fixed {
            if *col >= grid_cols || *row >= grid_rows {
                return Err(format!(
                    "fixed cell ({col}, {row}) out of bounds for grid ({grid_cols}, {grid_rows})",
                ));
            }
        }

        for slot in &self.slots {
            let (ox, oy) = slot.origin;
            if ox + slot.size.cols() > grid_cols || oy + slot.size.rows() > grid_rows {
                return Err(format!(
                    "slot {:?} out of bounds: origin ({ox}, {oy}) + size {:?} exceeds grid ({grid_cols}, {grid_rows})",
                    slot.id, slot.size,
                ));
            }
        }

        for ((col, row), _) in &self.fixed {
            for slot in &self.slots {
                let (ox, oy) = slot.origin;
                let sc = slot.size.cols();
                let sr = slot.size.rows();
                if *col >= ox && *col < ox + sc && *row >= oy && *row < oy + sr {
                    return Err(format!(
                        "fixed cell ({col}, {row}) overlaps slot {:?}",
                        slot.id,
                    ));
                }
            }
        }

        for (i, a) in self.slots.iter().enumerate() {
            for b in self.slots.iter().skip(i + 1) {
                let (ax, ay) = a.origin;
                let (bx, by) = b.origin;
                let ac = a.size.cols();
                let ar = a.size.rows();
                let bc = b.size.cols();
                let br = b.size.rows();
                if ax < bx + bc && bx < ax + ac && ay < by + br && by < ay + ar {
                    return Err(format!("slots {:?} and {:?} overlap", a.id, b.id));
                }
            }
        }

        if self.portal {
            for ((..), constraint) in &self.fixed {
                if let CellConstraint::MustInclude(v) = constraint
                    && v.contains(&BehaviorKind::Portal)
                {
                    return Err(
                        "portal frame may not require Portal via MustInclude in fixed cells"
                            .to_owned(),
                    );
                }
            }
        }

        Ok(())
    }
}

/// Block template (a rectangular region of cells with constraints) loaded
/// from a RON asset.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct BlockDef {
    pub size:      BlockSize,
    pub min_tier:  u32,
    pub max_tier:  u32,
    pub cells:     Vec<((u32, u32), CellConstraint)>,
    pub sequences: Vec<Sequence>,
}

impl BlockDef {
    /// Validates block structure: tier range (`min_tier <= max_tier`), cell positions
    /// within size grid, no duplicate cell positions, and every sequence cell
    /// references a position present in `cells`.
    ///
    /// Currently gated `#[cfg(test)]` — Wave 2 will lift this gate when registry
    /// `seed()` callers wire production validation.
    #[cfg(test)]
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.min_tier > self.max_tier {
            return Err(format!(
                "block tier range invalid: min_tier {} > max_tier {}",
                self.min_tier, self.max_tier,
            ));
        }

        let cols = self.size.cols();
        let rows = self.size.rows();
        for ((col, row), _) in &self.cells {
            if *col >= cols || *row >= rows {
                return Err(format!(
                    "block cell ({col}, {row}) out of bounds for {:?}",
                    self.size,
                ));
            }
        }

        let positions: HashSet<(u32, u32)> = self.cells.iter().map(|(pos, _)| *pos).collect();
        if positions.len() != self.cells.len() {
            return Err("block has duplicate cell position(s)".to_owned());
        }

        for seq in &self.sequences {
            for pos in &seq.cells {
                if !positions.contains(pos) {
                    return Err(format!(
                        "sequence references missing position ({}, {})",
                        pos.0, pos.1,
                    ));
                }
            }
        }

        Ok(())
    }
}

/// A single tier entry in a modifier pool — the tier index and a list of
/// `(BehaviorKind, weight)` pairs.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct TierModifierPool {
    pub tier:      u32,
    pub modifiers: Vec<(BehaviorKind, f32)>,
}

/// The complete modifier pool asset — an ordered list of `TierModifierPool`
/// entries indexed by tier.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct TierPools(pub Vec<TierModifierPool>);

impl TierPools {
    /// Validates the tier pool list: no duplicate `tier` indices across entries,
    /// every modifier weight is non-negative, and within each entry the sum of
    /// modifier weights is `<= 1.0` (the remainder is plain-cell probability).
    ///
    /// Currently gated `#[cfg(test)]` — Wave 2 will lift this gate when registry
    /// `seed()` callers wire production validation.
    #[cfg(test)]
    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut seen: HashSet<u32> = HashSet::new();
        for pool in &self.0 {
            if !seen.insert(pool.tier) {
                return Err(format!(
                    "duplicate tier index in TierPools: tier {} appears more than once",
                    pool.tier,
                ));
            }
        }

        for pool in &self.0 {
            for (kind, weight) in &pool.modifiers {
                if *weight < 0.0 {
                    return Err(format!(
                        "tier {} modifier {kind:?} has negative weight {weight}",
                        pool.tier,
                    ));
                }
            }
        }

        for pool in &self.0 {
            let sum: f32 = pool.modifiers.iter().map(|(_, w)| *w).sum();
            if sum > 1.0 {
                return Err(format!("tier {} weight sum {} exceeds 1.0", pool.tier, sum));
            }
        }

        Ok(())
    }
}
