use super::super::core::*;

// ── Behavior 43: SideData::compute_position for Left ──

#[test]
fn left_compute_position_default_ht() {
    let left = Left {
        playfield_left: -400.0,
        half_height: 300.0,
    };
    let pos = left.compute_position(90.0);
    assert!(
        (pos.x - (-490.0)).abs() < f32::EPSILON,
        "Left position x should be -490.0, got {}",
        pos.x
    );
    assert!(
        pos.y.abs() < f32::EPSILON,
        "Left position y should be 0.0, got {}",
        pos.y
    );
}

#[test]
fn left_compute_position_zero_ht() {
    let left = Left {
        playfield_left: -400.0,
        half_height: 300.0,
    };
    let pos = left.compute_position(0.0);
    assert!(
        (pos.x - (-400.0)).abs() < f32::EPSILON,
        "Left position x should be -400.0 with ht=0.0, got {}",
        pos.x
    );
    assert!(pos.y.abs() < f32::EPSILON, "Left position y should be 0.0");
}

// ── Behavior 44: SideData::compute_half_extents for Left ──

#[test]
fn left_compute_half_extents_default_ht() {
    let left = Left {
        playfield_left: -400.0,
        half_height: 300.0,
    };
    let he = left.compute_half_extents(90.0);
    assert!(
        (he.x - 90.0).abs() < f32::EPSILON,
        "Left half_extents.x should be 90.0, got {}",
        he.x
    );
    assert!(
        (he.y - 300.0).abs() < f32::EPSILON,
        "Left half_extents.y should be 300.0, got {}",
        he.y
    );
}

#[test]
fn left_compute_half_extents_zero_ht() {
    let left = Left {
        playfield_left: -400.0,
        half_height: 300.0,
    };
    let he = left.compute_half_extents(0.0);
    assert!(
        he.x.abs() < f32::EPSILON,
        "Left half_extents.x should be 0.0 with ht=0.0, got {}",
        he.x
    );
    assert!(
        (he.y - 300.0).abs() < f32::EPSILON,
        "Left half_extents.y should be 300.0"
    );
}

// ── Behavior 45: SideData::compute_position for Right ──

#[test]
fn right_compute_position_default_ht() {
    let right = Right {
        playfield_right: 400.0,
        half_height: 300.0,
    };
    let pos = right.compute_position(90.0);
    assert!(
        (pos.x - 490.0).abs() < f32::EPSILON,
        "Right position x should be 490.0, got {}",
        pos.x
    );
    assert!(pos.y.abs() < f32::EPSILON, "Right position y should be 0.0");
}

#[test]
fn right_compute_position_zero_ht() {
    let right = Right {
        playfield_right: 400.0,
        half_height: 300.0,
    };
    let pos = right.compute_position(0.0);
    assert!(
        (pos.x - 400.0).abs() < f32::EPSILON,
        "Right position x should be 400.0 with ht=0.0, got {}",
        pos.x
    );
}

// ── Behavior 46: SideData::compute_half_extents for Right ──

#[test]
fn right_compute_half_extents_default_ht() {
    let right = Right {
        playfield_right: 400.0,
        half_height: 300.0,
    };
    let he = right.compute_half_extents(90.0);
    assert!(
        (he.x - 90.0).abs() < f32::EPSILON,
        "Right half_extents.x should be 90.0, got {}",
        he.x
    );
    assert!(
        (he.y - 300.0).abs() < f32::EPSILON,
        "Right half_extents.y should be 300.0"
    );
}

#[test]
fn right_compute_half_extents_zero_ht() {
    let right = Right {
        playfield_right: 400.0,
        half_height: 300.0,
    };
    let he = right.compute_half_extents(0.0);
    assert!(
        he.x.abs() < f32::EPSILON,
        "Right half_extents.x should be 0.0 with ht=0.0"
    );
    assert!(
        (he.y - 300.0).abs() < f32::EPSILON,
        "Right half_extents.y should be 300.0"
    );
}

// ── Behavior 47: SideData::compute_position for Ceiling ──

#[test]
fn ceiling_compute_position_default_ht() {
    let ceiling = Ceiling {
        playfield_top: 300.0,
        half_width: 400.0,
    };
    let pos = ceiling.compute_position(90.0);
    assert!(
        pos.x.abs() < f32::EPSILON,
        "Ceiling position x should be 0.0, got {}",
        pos.x
    );
    assert!(
        (pos.y - 390.0).abs() < f32::EPSILON,
        "Ceiling position y should be 390.0, got {}",
        pos.y
    );
}

#[test]
fn ceiling_compute_position_zero_ht() {
    let ceiling = Ceiling {
        playfield_top: 300.0,
        half_width: 400.0,
    };
    let pos = ceiling.compute_position(0.0);
    assert!(
        (pos.y - 300.0).abs() < f32::EPSILON,
        "Ceiling position y should be 300.0 with ht=0.0, got {}",
        pos.y
    );
}

// ── Behavior 48: SideData::compute_half_extents for Ceiling ──

#[test]
fn ceiling_compute_half_extents_default_ht() {
    let ceiling = Ceiling {
        playfield_top: 300.0,
        half_width: 400.0,
    };
    let he = ceiling.compute_half_extents(90.0);
    assert!(
        (he.x - 400.0).abs() < f32::EPSILON,
        "Ceiling half_extents.x should be 400.0, got {}",
        he.x
    );
    assert!(
        (he.y - 90.0).abs() < f32::EPSILON,
        "Ceiling half_extents.y should be 90.0, got {}",
        he.y
    );
}

#[test]
fn ceiling_compute_half_extents_zero_ht() {
    let ceiling = Ceiling {
        playfield_top: 300.0,
        half_width: 400.0,
    };
    let he = ceiling.compute_half_extents(0.0);
    assert!(
        (he.x - 400.0).abs() < f32::EPSILON,
        "Ceiling half_extents.x should be 400.0"
    );
    assert!(
        he.y.abs() < f32::EPSILON,
        "Ceiling half_extents.y should be 0.0 with ht=0.0"
    );
}

// ── Behavior 49: SideData::compute_position for Floor ──

#[test]
fn floor_compute_position_default_ht() {
    let floor = Floor {
        playfield_bottom: -300.0,
        half_width: 400.0,
    };
    let pos = floor.compute_position(90.0);
    assert!(
        pos.x.abs() < f32::EPSILON,
        "Floor position x should be 0.0, got {}",
        pos.x
    );
    assert!(
        (pos.y - (-300.0)).abs() < f32::EPSILON,
        "Floor position y should be -300.0, got {}",
        pos.y
    );
}

#[test]
fn floor_compute_position_zero_ht() {
    let floor = Floor {
        playfield_bottom: -300.0,
        half_width: 400.0,
    };
    let pos = floor.compute_position(0.0);
    assert!(
        (pos.y - (-300.0)).abs() < f32::EPSILON,
        "Floor position y should be -300.0 with ht=0.0 (same y regardless of ht), got {}",
        pos.y
    );
}

// ── Behavior 50: SideData::compute_half_extents for Floor ──

#[test]
fn floor_compute_half_extents_default_ht() {
    let floor = Floor {
        playfield_bottom: -300.0,
        half_width: 400.0,
    };
    let he = floor.compute_half_extents(90.0);
    assert!(
        (he.x - 400.0).abs() < f32::EPSILON,
        "Floor half_extents.x should be 400.0, got {}",
        he.x
    );
    assert!(
        (he.y - 90.0).abs() < f32::EPSILON,
        "Floor half_extents.y should be 90.0, got {}",
        he.y
    );
}

#[test]
fn floor_compute_half_extents_zero_ht() {
    let floor = Floor {
        playfield_bottom: -300.0,
        half_width: 400.0,
    };
    let he = floor.compute_half_extents(0.0);
    assert!(
        (he.x - 400.0).abs() < f32::EPSILON,
        "Floor half_extents.x should be 400.0"
    );
    assert!(
        he.y.abs() < f32::EPSILON,
        "Floor half_extents.y should be 0.0 with ht=0.0"
    );
}
