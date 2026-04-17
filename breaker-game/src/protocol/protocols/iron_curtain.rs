//! `IronCurtain` protocol — scaffold.

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct IronCurtainConfig {
    pub(crate) damage_fraction: f32,
    pub(crate) falloff_start:   f32,
}

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::IronCurtain {
        damage_fraction,
        falloff_start,
    } = *tuning
    else {
        warn!("iron_curtain::activate called with non-IronCurtain tuning");
        return;
    };
    commands.insert_resource(IronCurtainConfig {
        damage_fraction,
        falloff_start,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        Update,
        warn_stub.run_if(protocol_active(ProtocolKind::IronCurtain)),
    );
}

fn warn_stub(cfg: Option<Res<IronCurtainConfig>>, mut seen: Local<bool>) {
    if *seen || cfg.is_none() {
        return;
    }
    warn!("IronCurtain activated — runtime behaviour pending damage-pipeline plumbing.");
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
                &ProtocolTuning::IronCurtain {
                    damage_fraction: 0.25,
                    falloff_start:   0.5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<IronCurtainConfig>();
        assert!((cfg.damage_fraction - 0.25).abs() < f32::EPSILON);
        assert!((cfg.falloff_start - 0.5).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<IronCurtainConfig>().is_none());
    }
}
