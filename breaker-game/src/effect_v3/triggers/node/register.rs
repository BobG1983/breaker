//! Registration for node trigger bridges, threshold checker, and resources.

use bevy::prelude::*;

use super::{
    bridges, check_thresholds, messages::NodeTimerThresholdCrossed,
    resources::NodeTimerThresholdRegistry,
};
use crate::{effect_v3::EffectV3Systems, state::types::NodeState};

/// Registers node trigger bridge systems, the threshold checker, and resources.
pub fn register(app: &mut App) {
    app.init_resource::<NodeTimerThresholdRegistry>();
    app.add_message::<NodeTimerThresholdCrossed>();

    // NodeStart/NodeEnd are state-transition-based, not FixedUpdate
    app.add_systems(OnEnter(NodeState::Playing), bridges::on_node_start_occurred);
    app.add_systems(OnExit(NodeState::Playing), bridges::on_node_end_occurred);

    // Threshold checker and its bridge run in FixedUpdate
    app.add_systems(
        FixedUpdate,
        (
            check_thresholds::check_node_timer_thresholds,
            bridges::on_node_timer_threshold_occurred,
        )
            .chain()
            .in_set(EffectV3Systems::Bridge),
    );
}
