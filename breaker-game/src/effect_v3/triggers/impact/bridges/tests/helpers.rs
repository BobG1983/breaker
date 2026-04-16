use bevy::{ecs::system::SystemParam, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::{on_impact_occurred, on_impacted};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, EntityKind, Tree, Trigger},
    },
    prelude::*,
};

// -- Test message resource ------------------------------------------------

/// Resource to inject all seven collision message types into the test app.
#[derive(Resource, Default)]
pub(super) struct TestImpactMessages {
    pub(super) bolt_cell:     Vec<BoltImpactCell>,
    pub(super) bolt_wall:     Vec<BoltImpactWall>,
    pub(super) bolt_breaker:  Vec<BoltImpactBreaker>,
    pub(super) breaker_cell:  Vec<BreakerImpactCell>,
    pub(super) breaker_wall:  Vec<BreakerImpactWall>,
    pub(super) cell_wall:     Vec<CellImpactWall>,
    pub(super) salvo_breaker: Vec<SalvoImpactBreaker>,
}

#[derive(SystemParam)]
struct ImpactWriters<'w> {
    bolt_cell:     MessageWriter<'w, BoltImpactCell>,
    bolt_wall:     MessageWriter<'w, BoltImpactWall>,
    bolt_breaker:  MessageWriter<'w, BoltImpactBreaker>,
    breaker_cell:  MessageWriter<'w, BreakerImpactCell>,
    breaker_wall:  MessageWriter<'w, BreakerImpactWall>,
    cell_wall:     MessageWriter<'w, CellImpactWall>,
    salvo_breaker: MessageWriter<'w, SalvoImpactBreaker>,
}

fn inject_impacts(messages: Res<TestImpactMessages>, mut w: ImpactWriters) {
    for msg in &messages.bolt_cell {
        w.bolt_cell.write(msg.clone());
    }
    for msg in &messages.bolt_wall {
        w.bolt_wall.write(msg.clone());
    }
    for msg in &messages.bolt_breaker {
        w.bolt_breaker.write(msg.clone());
    }
    for msg in &messages.breaker_cell {
        w.breaker_cell.write(msg.clone());
    }
    for msg in &messages.breaker_wall {
        w.breaker_wall.write(msg.clone());
    }
    for msg in &messages.cell_wall {
        w.cell_wall.write(msg.clone());
    }
    for msg in &messages.salvo_breaker {
        w.salvo_breaker.write(msg.clone());
    }
}

pub(super) fn bridge_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltImpactWall>()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BreakerImpactCell>()
        .with_message::<BreakerImpactWall>()
        .with_message::<CellImpactWall>()
        .with_message::<SalvoImpactBreaker>()
        .with_resource::<TestImpactMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_impacts.before(on_impact_occurred),
                on_impact_occurred,
            ),
        )
        .build()
}

/// Helper to build a When(ImpactOccurred(kind), Fire(SpeedBoost)) tree.
pub(super) fn impact_occurred_speed_tree(
    name: &str,
    kind: EntityKind,
    multiplier: f32,
) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::ImpactOccurred(kind),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

/// Helper to build a When(Impacted(kind), Fire(SpeedBoost)) tree.
pub(super) fn impacted_speed_tree(name: &str, kind: EntityKind, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::Impacted(kind),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

/// Builds a test app with `on_impacted` wired (local dispatch).
pub(super) fn impacted_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltImpactWall>()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BreakerImpactCell>()
        .with_message::<BreakerImpactWall>()
        .with_message::<CellImpactWall>()
        .with_message::<SalvoImpactBreaker>()
        .with_resource::<TestImpactMessages>()
        .with_system(
            FixedUpdate,
            (inject_impacts.before(on_impacted), on_impacted),
        )
        .build()
}
