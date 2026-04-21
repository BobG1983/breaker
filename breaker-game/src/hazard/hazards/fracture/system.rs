use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellHeight, CellWidth},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::{
        collision_layers::{BOLT_LAYER, CELL_LAYER},
        death_pipeline::{Destroyed, Hp, KilledBy},
    },
};

/// Default debris cell dimensions. Matches the pristine cell footprint
/// used by `bolt_cell_collision` tests; keeps debris playable without
/// needing to consult per-node grid metrics here.
const DEBRIS_WIDTH: f32 = 70.0;
const DEBRIS_HEIGHT: f32 = 24.0;
/// Fixed 1 HP per debris cell regardless of stack — stacking increases
/// count only, per the design.
const DEBRIS_HP: f32 = 1.0;

/// World-space offsets for the four orthogonal debris positions, in the
/// order debris are placed (right, left, up, down). Over-stacks that
/// exceed four simply run out of offsets — the effective cap at stack 3+
/// is four debris per death.
const DEBRIS_OFFSETS: [Vec2; 4] = [
    Vec2::new(DEBRIS_WIDTH, 0.0),
    Vec2::new(-DEBRIS_WIDTH, 0.0),
    Vec2::new(0.0, DEBRIS_HEIGHT),
    Vec2::new(0.0, -DEBRIS_HEIGHT),
];

/// Per-run tuning extracted from [`HazardTuning::Fracture`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct FractureConfig {
    /// Debris spawned per death at stack 1.
    pub(crate) base_splits:      u32,
    /// Additional debris spawned per stack beyond the first.
    pub(crate) per_level_splits: u32,
}

impl FractureConfig {
    /// Debris count for the given stack count. Returns `0` when
    /// `stacks == 0` (hazard inactive). For `stacks >= 1`, returns
    /// `min(base_splits + per_level_splits * (stacks - 1),
    /// DEBRIS_OFFSETS.len() as u32)` — linear scaling in stacks, saturated
    /// at the number of orthogonal offsets (4) so indexing into
    /// `DEBRIS_OFFSETS` is always in-bounds. `saturating_sub`,
    /// `saturating_mul`, and `saturating_add` prevent overflow on
    /// pathological `u32::MAX` inputs. `const fn` so config consumers can
    /// evaluate at compile time.
    #[must_use]
    pub(crate) const fn splits_for(self, stacks: u32) -> u32 {
        if stacks == 0 {
            return 0;
        }
        let extra = stacks.saturating_sub(1);
        let raw = self
            .base_splits
            .saturating_add(self.per_level_splits.saturating_mul(extra));
        if raw > DEBRIS_OFFSETS.len() as u32 {
            DEBRIS_OFFSETS.len() as u32
        } else {
            raw
        }
    }
}

/// Marks a cell as debris spawned by the Fracture hazard. Lets other
/// systems identify and (optionally) treat debris differently from
/// regular cells. Currently inert — reserved for future
/// "don't-recurse-fracture-on-debris" logic.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct FractureDebris;

/// Inserts `FractureConfig` from `HazardTuning::Fracture { base_splits,
/// per_level_splits }`. Called each time the player picks Fracture from
/// `hazards::activate`; last write wins (overwrites any prior
/// `FractureConfig`; stack count is owned by `ActiveHazards`, not the
/// config; debris already spawned retain their existing `Hp` — the
/// config only governs future spawn count). Does not mutate
/// `ActiveHazards`. Warns and no-ops on a non-Fracture tuning variant,
/// leaving any existing `FractureConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Fracture {
        base_splits,
        per_level_splits,
    } = *tuning
    else {
        warn!("fracture::activate called with non-Fracture tuning");
        return;
    };
    commands.insert_resource(FractureConfig {
        base_splits,
        per_level_splits,
    });
}

/// Registers the Fracture `FixedUpdate` system `fracture_on_death`,
/// gated on `hazard_active(HazardKind::Fracture)` AND
/// `in_state(NodeState::Playing)`. `fracture_on_death` reads
/// `Destroyed<Cell>` and spawns `count` debris cells per destroyed
/// cell at orthogonal offsets, where `count = FractureConfig::splits_for(stacks)`.
/// No `DeathPipelineSystems` ordering — debris are spawned directly via
/// `commands.spawn(...)`, not via a `SpawnDebrisCell` message
/// (pending the message pipeline in Commit 5 / Wave 7).
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        fracture_on_death
            .run_if(hazard_active(HazardKind::Fracture))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Spawns `splits_for(stacks)` debris cells in orthogonal positions
/// around each destroyed cell. Excludes the victim itself — debris
/// cells are `FractureDebris`-marked with `CellWidth`/`CellHeight` and
/// the same collision layers as regular cells. Early-returns (draining
/// the message reader via `reader.clear()`) when `FractureConfig` is
/// absent, or when `splits_for(stacks) == 0` (zero-stack inactive
/// path) — the drain prevents stale `Destroyed<Cell>` messages from
/// spawning debris on a later tick after the hazard activates.
pub(crate) fn fracture_on_death(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<FractureConfig>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let stacks = active.stacks(HazardKind::Fracture);
    let count = config.splits_for(stacks) as usize;
    if count == 0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        for offset in &DEBRIS_OFFSETS[..count] {
            let pos = destroyed.victim_pos + *offset;
            commands.spawn((
                Cell,
                FractureDebris,
                Position2D(pos),
                Scale2D {
                    x: DEBRIS_WIDTH,
                    y: DEBRIS_HEIGHT,
                },
                Aabb2D::new(
                    Vec2::ZERO,
                    Vec2::new(DEBRIS_WIDTH / 2.0, DEBRIS_HEIGHT / 2.0),
                ),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                CellWidth::new(DEBRIS_WIDTH),
                CellHeight::new(DEBRIS_HEIGHT),
                Hp::new(DEBRIS_HP),
                KilledBy::default(),
            ));
        }
    }
}
