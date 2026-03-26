//! Bridge systems for impact triggers — sweeps ALL entities with `EffectChains`
//! for `Trigger::Impact(Cell/Wall/Breaker)`, evaluates `ArmedEffects` on bolt,
//! and evaluates `Until` children.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, EffectTarget, ImpactTarget, Trigger},
        effect_nodes::until::{UntilTimers, UntilTriggers},
        helpers::*,
    },
};

/// Bridge for `BoltHitCell` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Cell)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_cell_impact(
    mut reader: MessageReader<BoltHitCell>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Cell);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);

        evaluate_until_children(
            &until_query,
            bolt_entity,
            trigger_kind,
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitWall` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Wall)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_wall_impact(
    mut reader: MessageReader<BoltHitWall>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Wall);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);

        evaluate_until_children(
            &until_query,
            bolt_entity,
            trigger_kind,
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitBreaker` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Breaker)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_breaker_impact(
    mut reader: MessageReader<BoltHitBreaker>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Breaker);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);
    }
}

/// Registers bridge systems for impact triggers.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        (
            bridge_cell_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_wall_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_breaker_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::effect::{
        definition::{Effect, EffectNode, ImpactTarget, Trigger},
        typed_events::*,
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedShieldFired(Vec<ShieldFired>);

    fn capture_shield_fired(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShieldFired>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource)]
    struct SendBoltHitCell(Option<BoltHitCell>);

    fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitWall(Option<BoltHitWall>);

    fn send_bolt_hit_wall(msg: Res<SendBoltHitWall>, mut writer: MessageWriter<BoltHitWall>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitBreaker(Option<BoltHitBreaker>);

    fn send_bolt_hit_breaker(
        msg: Res<SendBoltHitBreaker>,
        mut writer: MessageWriter<BoltHitBreaker>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn cell_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );
        app
    }

    fn wall_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );
        app
    }

    fn breaker_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedShieldFired>()
            .add_observer(capture_shield_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );
        app
    }

    // --- Cell impact bridge tests ---

    /// Both bolt and breaker entities have `When(Impact(Cell))` chains.
    /// On `BoltHitCell`, BOTH should fire (sweep all entities).
    #[test]
    fn bridge_cell_impact_sweeps_all_entities() {
        let mut app = cell_impact_test_app();

        // Bolt entity with Impact(Cell) chain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Cell),
                    Effect::test_shockwave(64.0),
                ),
            )]))
            .id();

        // Unrelated entity (simulating breaker) also with Impact(Cell) chain
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impact(ImpactTarget::Cell),
                Effect::test_shockwave(32.0),
            ),
        )]));

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "both bolt and second entity should fire on cell impact — got {}",
            captured.0.len()
        );
    }

    // --- Wall impact bridge tests ---

    /// Both bolt and breaker entities have `When(Impact(Wall))` chains.
    /// On `BoltHitWall`, BOTH should fire (sweep all entities).
    #[test]
    fn bridge_wall_impact_sweeps_all_entities() {
        let mut app = wall_impact_test_app();

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Wall),
                    Effect::test_shockwave(64.0),
                ),
            )]))
            .id();

        // Second entity also with Impact(Wall) chain
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impact(ImpactTarget::Wall),
                Effect::test_shockwave(32.0),
            ),
        )]));

        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall {
            bolt,
            wall: Entity::PLACEHOLDER,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "both bolt and second entity should fire on wall impact — got {}",
            captured.0.len()
        );
    }

    // --- Breaker impact bridge tests ---

    /// Both bolt and breaker entities have `When(Impact(Breaker))` chains.
    /// On `BoltHitBreaker`, BOTH should fire (sweep all entities).
    #[test]
    fn bridge_breaker_impact_sweeps_all_entities() {
        let mut app = breaker_impact_test_app();

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Breaker),
                    Effect::test_shield(5.0),
                ),
            )]))
            .id();

        // Second entity also with Impact(Breaker) chain
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impact(ImpactTarget::Breaker),
                Effect::test_shield(3.0),
            ),
        )]));

        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShieldFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "both bolt and second entity should fire on breaker impact — got {}",
            captured.0.len()
        );
    }

    // --- M3: evaluate_until_children with matching When child ---

    /// M3: Until entry with When(Impact(Cell)) child fires ShockwaveFired when
    /// bridge_cell_impact calls evaluate_until_children. The UntilTriggers entry
    /// itself is NOT consumed (still active — Until removal is handled separately).
    #[test]
    fn bridge_cell_impact_evaluates_until_children_when_node() {
        use crate::effect::effect_nodes::until::{UntilTriggerEntry, UntilTriggers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt entity with UntilTriggers containing a When(Impact(Cell)) child
        let bolt = app
            .world_mut()
            .spawn(UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::BoltLost,
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "Until's When(Impact(Cell)) child should fire via evaluate_until_children"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "ShockwaveFired base_range should be 64.0"
        );

        // UntilTriggers entry should NOT be consumed — it's still active
        let triggers = app.world().get::<UntilTriggers>(bolt).unwrap();
        assert_eq!(
            triggers.0.len(),
            1,
            "UntilTriggers entry should NOT be consumed by evaluate_until_children"
        );
    }

    // --- Negative case: Impact(Cell) does NOT match Impact(Wall) ---

    /// Entity with `When(Impact(Wall))` should NOT fire on `BoltHitCell`.
    #[test]
    fn impact_cell_does_not_match_wall() {
        let mut app = cell_impact_test_app();

        let bolt = app.world_mut().spawn_empty().id();

        // Entity with Impact(Wall) — should NOT match cell impact
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impact(ImpactTarget::Wall),
                Effect::test_shockwave(64.0),
            ),
        )]));

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "Impact(Wall) chain should NOT fire on BoltHitCell"
        );
    }
}
