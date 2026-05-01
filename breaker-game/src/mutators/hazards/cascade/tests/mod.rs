//! Tests for the cascade hazard retrofit to the unified heal pipeline.
//!
//! Most runtime tests observe emitted `HealDealt<Cell>` messages via
//! `MessageCollector<HealDealt<Cell>>`; Group G runs the full
//! `cascade_heal_on_death` → `apply_heal::<Cell>` pipeline and observes
//! post-tick `Hp.current`.

mod helpers;
mod ron_asset;

mod group_a_formula;
mod group_b_activate;
mod group_c_guards;
mod group_d_single;
mod group_e;
mod group_f;
mod group_g;
mod group_h;
