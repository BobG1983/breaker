//! Group A — `BehaviorKind` enum (#1–3)

use std::collections::HashSet;

use crate::{
    cells::{
        behaviors::armored::components::ArmorDirection,
        definition::{CellBehavior, GuardedBehavior, data::AttackPattern},
    },
    state::run::generation::*,
};

// ── Behavior #1: BehaviorKind unit variants, derives ─────────────────────────

#[test]
fn behavior_kind_has_ten_unit_variants_with_correct_derives() {
    use BehaviorKind::*;
    let all = [
        Regen,
        Guarded,
        Volatile,
        Sequence,
        Armored,
        Phantom,
        Magnetic,
        Survival,
        SurvivalPermanent,
        Portal,
    ];
    assert_eq!(all.len(), 10);
    // Eq + Clone + Copy
    assert_eq!(BehaviorKind::Armored, BehaviorKind::Armored);
    assert_ne!(BehaviorKind::Armored, BehaviorKind::Volatile);

    // Hash + Eq: all 10 variants are distinct
    let set: HashSet<BehaviorKind> = all.iter().copied().collect();
    assert_eq!(
        set.len(),
        10,
        "all BehaviorKind variants must be distinct under Hash+Eq"
    );
}

// ── Behavior #2: BehaviorKind mirrors every CellBehavior discriminator ────────

fn kind_of(b: &CellBehavior) -> BehaviorKind {
    match b {
        CellBehavior::Regen { .. } => BehaviorKind::Regen,
        CellBehavior::Guarded(_) => BehaviorKind::Guarded,
        CellBehavior::Volatile { .. } => BehaviorKind::Volatile,
        CellBehavior::Sequence { .. } => BehaviorKind::Sequence,
        CellBehavior::Armored { .. } => BehaviorKind::Armored,
        CellBehavior::Phantom { .. } => BehaviorKind::Phantom,
        CellBehavior::Magnetic { .. } => BehaviorKind::Magnetic,
        CellBehavior::Survival { .. } => BehaviorKind::Survival,
        CellBehavior::SurvivalPermanent { .. } => BehaviorKind::SurvivalPermanent,
        CellBehavior::Portal { .. } => BehaviorKind::Portal,
    }
}

#[test]
fn behavior_kind_mirrors_every_cell_behavior_discriminator() {
    // Simple variants
    assert_eq!(
        kind_of(&CellBehavior::Regen { rate: 1.0 }),
        BehaviorKind::Regen
    );
    assert_eq!(
        kind_of(&CellBehavior::Guarded(GuardedBehavior {
            guardian_hp_fraction: 0.5,
            guardian_color_rgb:   [1.0, 1.0, 1.0],
            slide_speed:          60.0,
        })),
        BehaviorKind::Guarded,
    );
    assert_eq!(
        kind_of(&CellBehavior::Volatile {
            damage: 1.0,
            radius: 1.0,
        }),
        BehaviorKind::Volatile,
    );
    assert_eq!(
        kind_of(&CellBehavior::Sequence {
            group:    0,
            position: 0,
        }),
        BehaviorKind::Sequence,
    );
    assert_eq!(
        kind_of(&CellBehavior::Armored {
            value:  1,
            facing: ArmorDirection::Bottom,
        }),
        BehaviorKind::Armored,
    );
    assert_eq!(
        kind_of(&CellBehavior::Phantom {
            cycle_secs:     1.0,
            telegraph_secs: 0.2,
            starting_phase: crate::cells::behaviors::phantom::components::PhantomPhase::Solid,
        }),
        BehaviorKind::Phantom,
    );
    assert_eq!(
        kind_of(&CellBehavior::Magnetic {
            radius:   1.0,
            strength: 1.0,
        }),
        BehaviorKind::Magnetic,
    );
    assert_eq!(
        kind_of(&CellBehavior::Survival {
            pattern:    AttackPattern::StraightDown,
            timer_secs: 5.0,
        }),
        BehaviorKind::Survival,
    );
    assert_eq!(
        kind_of(&CellBehavior::SurvivalPermanent {
            pattern: AttackPattern::StraightDown,
        }),
        BehaviorKind::SurvivalPermanent,
    );
    assert_eq!(
        kind_of(&CellBehavior::Portal {
            sub_node_tier_offset: 0,
        }),
        BehaviorKind::Portal,
    );
}

// ── Behavior #3: BehaviorKind deserializes from bare-identifier RON ───────────

#[test]
fn behavior_kind_deserializes_from_bare_ron_identifiers() {
    let armored: BehaviorKind = ron::de::from_str("Armored").expect("Armored should parse");
    assert_eq!(armored, BehaviorKind::Armored);

    let portal: BehaviorKind = ron::de::from_str("Portal").expect("Portal should parse");
    assert_eq!(portal, BehaviorKind::Portal);

    let volatile: BehaviorKind = ron::de::from_str("Volatile").expect("Volatile should parse");
    assert_eq!(volatile, BehaviorKind::Volatile);

    let survival_perm: BehaviorKind =
        ron::de::from_str("SurvivalPermanent").expect("SurvivalPermanent should parse");
    assert_eq!(survival_perm, BehaviorKind::SurvivalPermanent);

    // Unknown variant rejected
    let err = ron::de::from_str::<BehaviorKind>("Unknown");
    assert!(err.is_err(), "unknown variant 'Unknown' must be rejected");
}
