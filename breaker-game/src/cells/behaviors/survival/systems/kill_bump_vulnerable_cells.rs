//! Writes lethal `DamageDealt<Cell>` for `BumpVulnerable` cells hit by
//! the breaker.
//!
//! Reads `BreakerImpactCell` messages and writes `DamageDealt<Cell>`
//! with `amount: f32::MAX` for each hit cell that has `BumpVulnerable`
//! and is not `Dead`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{cells::behaviors::survival::components::BumpVulnerable, prelude::*};

/// Writes `DamageDealt<Cell>` with `f32::MAX` for living `BumpVulnerable`
/// cells on breaker contact. Does not bypass `Invulnerable`.
pub(crate) fn kill_bump_vulnerable_cells(
    mut impacts: MessageReader<BreakerImpactCell>,
    vulnerable_query: Query<(), (With<BumpVulnerable>, Without<Dead>)>,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for impact in impacts.read() {
        if vulnerable_query.get(impact.cell).is_ok() {
            damage_writer.write(DamageDealt {
                dealer:      Some(impact.breaker),
                target:      impact.cell,
                amount:      f32::MAX,
                source_chip: None,
                _marker:     PhantomData,
            });
        }
    }
}
