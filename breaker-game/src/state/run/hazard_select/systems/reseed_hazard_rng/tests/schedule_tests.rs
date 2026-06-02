use bevy::prelude::*;
use rantzsoft_stateflow::RoutingTable;

use super::super::system::reseed_hazard_rng;
use crate::{
    prelude::*,
    state::run::hazard_select::{HazardSelectPlugin, sets::HazardSelectSystems},
};

// ── Group F, Behavior 30 — schedule set membership ──────────────────────

#[test]
fn reseed_hazard_rng_is_in_hazard_select_reseed_set() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.init_resource::<RoutingTable<HazardSelectState>>();
    app.add_plugins(HazardSelectPlugin);

    assert!(
        system_in_set(
            &mut app,
            OnEnter(HazardSelectState::Selecting),
            reseed_hazard_rng,
            HazardSelectSystems::Reseed,
        ),
        "reseed_hazard_rng must be in HazardSelectSystems::Reseed"
    );
}

#[test]
fn reseed_hazard_rng_is_not_in_generate_offerings_set() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.init_resource::<RoutingTable<HazardSelectState>>();
    app.add_plugins(HazardSelectPlugin);

    assert!(
        !system_in_set(
            &mut app,
            OnEnter(HazardSelectState::Selecting),
            reseed_hazard_rng,
            HazardSelectSystems::GenerateOfferings,
        ),
        "reseed_hazard_rng must NOT be in HazardSelectSystems::GenerateOfferings"
    );
}
