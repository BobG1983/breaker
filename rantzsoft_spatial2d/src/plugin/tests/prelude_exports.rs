use super::super::definition::*;

// -- Behavior: prelude re-exports all public API types --

#[test]
fn prelude_re_exports_spatial_systems() {
    // Verify `SpatialSystems` is importable from the prelude.
    use crate::prelude::SpatialSystems as PreludeSpatialSystems;

    let set = PreludeSpatialSystems::SavePrevious;
    // Prove it's the same type by comparing with the direct import.
    assert_eq!(
        set,
        SpatialSystems::SavePrevious,
        "SpatialSystems from prelude should be the same type as the direct import"
    );
}
