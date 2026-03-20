//! Seeds `DifficultyCurve` from loaded `DifficultyCurveDefaults`.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    run::difficulty::{DifficultyCurve, DifficultyCurveDefaults},
    screen::loading::resources::DefaultsCollection,
};

/// Reads the loaded `DifficultyCurveDefaults` asset and inserts `DifficultyCurve`.
pub(crate) fn seed_difficulty_curve(
    collection: Option<Res<DefaultsCollection>>,
    assets: Res<Assets<DifficultyCurveDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let Some(defaults) = assets.get(&collection.difficulty) else {
        return Progress { done: 0, total: 1 };
    };

    commands.insert_resource::<DifficultyCurve>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}
