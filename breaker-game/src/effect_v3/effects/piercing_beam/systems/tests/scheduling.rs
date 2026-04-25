//! W7 Behavior 6 (PiercingBeam): `apply_piercing_beam_damage` lives in
//! `DmgSystems::EmitDamage` in `FixedUpdate` — and ONLY there.

use bevy::prelude::*;

use super::{super::apply_piercing_beam_damage, helpers::*};
use crate::prelude::*;

#[test]
fn apply_piercing_beam_damage_is_in_emit_damage_set() {
    let mut app = piercing_pipeline_app();
    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            apply_piercing_beam_damage,
            DmgSystems::EmitDamage,
        ),
        "apply_piercing_beam_damage MUST live in DmgSystems::EmitDamage",
    );
}

#[test]
fn apply_piercing_beam_damage_is_not_in_mutate_damage_set() {
    let mut app = piercing_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_piercing_beam_damage,
            DmgSystems::MutateDamage,
        ),
        "apply_piercing_beam_damage MUST NOT live in DmgSystems::MutateDamage",
    );
}

#[test]
fn apply_piercing_beam_damage_is_not_in_apply_damage_set() {
    let mut app = piercing_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_piercing_beam_damage,
            DmgSystems::ApplyDamage,
        ),
        "apply_piercing_beam_damage MUST NOT live in DmgSystems::ApplyDamage",
    );
}

#[test]
fn apply_piercing_beam_damage_is_not_in_post_apply_damage_set() {
    let mut app = piercing_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_piercing_beam_damage,
            DmgSystems::PostApplyDamage,
        ),
        "apply_piercing_beam_damage MUST NOT live in DmgSystems::PostApplyDamage",
    );
}
