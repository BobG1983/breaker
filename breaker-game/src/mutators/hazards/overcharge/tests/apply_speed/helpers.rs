use crate::{chips::definition::Rarity, mutators::hazards::definition::HazardKind, prelude::*};

pub(super) fn hazard_overcharge() -> SourceId {
    SourceId::hazard(HazardKind::Overcharge).build()
}

pub(super) fn hazard_haste() -> SourceId {
    SourceId::hazard(HazardKind::Haste).build()
}

pub(super) fn chip_overclock() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

pub(super) fn chip_feedback_loop() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}
