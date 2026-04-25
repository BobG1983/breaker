//! W7 Behavior 6 (Explode): `apply_explode_damage` is registered in
//! `DmgSystems::EmitDamage` in `FixedUpdate` — and ONLY there. Asserts
//! single-set membership; guards against accidental dual-set membership in
//! `MutateDamage`, `ApplyDamage`, or `PostApplyDamage`.

use bevy::prelude::*;

use super::{super::apply_explode_damage, helpers::*};
use crate::prelude::*;

#[test]
fn apply_explode_damage_is_in_emit_damage_set() {
    let mut app = explode_pipeline_app();
    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            apply_explode_damage,
            DmgSystems::EmitDamage,
        ),
        "apply_explode_damage MUST live in DmgSystems::EmitDamage",
    );
}

#[test]
fn apply_explode_damage_is_not_in_mutate_damage_set() {
    let mut app = explode_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_explode_damage,
            DmgSystems::MutateDamage,
        ),
        "apply_explode_damage MUST NOT live in DmgSystems::MutateDamage",
    );
}

#[test]
fn apply_explode_damage_is_not_in_apply_damage_set() {
    let mut app = explode_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_explode_damage,
            DmgSystems::ApplyDamage,
        ),
        "apply_explode_damage MUST NOT live in DmgSystems::ApplyDamage",
    );
}

#[test]
fn apply_explode_damage_is_not_in_post_apply_damage_set() {
    let mut app = explode_pipeline_app();
    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            apply_explode_damage,
            DmgSystems::PostApplyDamage,
        ),
        "apply_explode_damage MUST NOT live in DmgSystems::PostApplyDamage",
    );
}
