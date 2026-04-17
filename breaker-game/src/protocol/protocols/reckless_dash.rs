//! `RecklessDash` protocol — scaffold.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct RecklessDashConfig {
    pub(crate) risky_zone_start:  f32,
    pub(crate) damage_multiplier: f32,
    pub(crate) double_penalty:    bool,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::RecklessDash {
        risky_zone_start,
        damage_multiplier,
        double_penalty,
    } = *tuning
    else {
        warn!("reckless_dash::activate called with non-RecklessDash tuning");
        return;
    };
    commands.insert_resource(RecklessDashConfig {
        risky_zone_start,
        damage_multiplier,
        double_penalty,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::RecklessDash)),
    );
}

fn warn_stub(cfg: Option<Res<RecklessDashConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("RecklessDash activated — risky-zone plumbing pending.");
    *seen = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::RecklessDash {
                    risky_zone_start:  0.3,
                    damage_multiplier: 4.0,
                    double_penalty:    true,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<RecklessDashConfig>();
        assert!((cfg.damage_multiplier - 4.0).abs() < f32::EPSILON);
        assert!(cfg.double_penalty);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<RecklessDashConfig>().is_none());
    }
}
