//! Burnout protocol — scaffold.

use bevy::prelude::*;

use crate::protocol::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::protocol_active,
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct BurnoutConfig {
    pub(crate) fill_duration:               f32,
    pub(crate) drain_duration:              f32,
    pub(crate) still_threshold:             f32,
    pub(crate) full_heat_damage_multiplier: f32,
    pub(crate) speed_boost_duration:        f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Burnout {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    } = *tuning
    else {
        warn!("burnout::activate called with non-Burnout tuning");
        return;
    };
    commands.insert_resource(BurnoutConfig {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::Burnout)),
    );
}

fn warn_stub(cfg: Option<Res<BurnoutConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("Burnout activated — heat-gauge plumbing pending.");
    *seen = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestAppBuilder;

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &ProtocolTuning::Burnout {
                    fill_duration:               3.0,
                    drain_duration:              5.0,
                    still_threshold:             0.25,
                    full_heat_damage_multiplier: 2.0,
                    speed_boost_duration:        1.0,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<BurnoutConfig>();
        assert!((cfg.fill_duration - 3.0).abs() < f32::EPSILON);
        assert!((cfg.full_heat_damage_multiplier - 2.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<BurnoutConfig>().is_none());
    }
}
