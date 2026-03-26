//! Shared helper functions for bridge systems.
//!
//! These helpers are used by multiple trigger files under `effect/triggers/`.

use bevy::prelude::*;

use super::{
    armed::ArmedEffects,
    definition::{EffectChains, EffectNode, EffectTarget, Trigger},
    effect_nodes::until::{UntilTimers, UntilTriggers},
    evaluate::evaluate_node,
    typed_events::fire_typed_event,
};
use crate::breaker::messages::{BumpGrade, BumpPerformed};

/// Evaluates armed triggers on all bolt entities that have `ArmedEffects`.
pub(super) fn evaluate_armed_all(
    mut armed_query: Query<(Entity, &mut ArmedEffects)>,
    trigger_kind: Trigger,
    commands: &mut Commands,
) {
    for (bolt_entity, mut armed) in &mut armed_query {
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        resolve_armed(&mut armed, trigger_kind, targets, commands);
    }
}

/// Arms a bolt entity with a remaining trigger chain.
///
/// If the bolt already has `ArmedEffects`, pushes to the existing vec.
/// Otherwise, inserts a new `ArmedEffects` component.
pub(super) fn arm_bolt(
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    bolt_entity: Entity,
    chip_name: Option<String>,
    remaining: EffectNode,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        armed.0.push((chip_name, remaining));
    } else {
        commands
            .entity(bolt_entity)
            .insert(ArmedEffects(vec![(chip_name, remaining)]));
    }
}

/// Evaluates armed triggers on a specific bolt entity.
pub(super) fn evaluate_armed(
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    bolt_entity: Entity,
    trigger_kind: Trigger,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        resolve_armed(&mut armed, trigger_kind, targets, commands);
    }
}

/// Resolves armed trigger chains: fires leaves, re-arms non-leaves, retains non-matches.
pub(super) fn resolve_armed(
    armed: &mut ArmedEffects,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for (chip_name, chain) in armed.0.drain(..) {
        if let Some(children) = evaluate_node(trigger_kind, &chain) {
            for child in children {
                match child {
                    EffectNode::Do(effect) => {
                        fire_typed_event(
                            effect.clone(),
                            targets.clone(),
                            chip_name.clone(),
                            commands,
                        );
                    }
                    other => {
                        new_armed.push((chip_name.clone(), other.clone()));
                    }
                }
            }
        } else {
            new_armed.push((chip_name, chain));
        }
    }
    armed.0 = new_armed;
}

/// Evaluates entity-local `EffectChains` against a trigger kind.
///
/// Handles `Once` node unwrapping: if a `Once` wraps children that match the
/// trigger, the effects are fired and the `Once` is consumed (removed).
/// If no children match, the `Once` is preserved.
pub(super) fn evaluate_entity_chains(
    chains: &mut EffectChains,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut consumed_indices = Vec::new();

    for (i, (chip_name, node)) in chains.0.iter().enumerate() {
        match node {
            EffectNode::Once(children) => {
                // Unwrap the Once: evaluate inner children against the trigger
                let mut any_matched = false;
                for child in children {
                    if let Some(grandchildren) = evaluate_node(trigger_kind, child) {
                        any_matched = true;
                        for gc in grandchildren {
                            if let EffectNode::Do(effect) = gc {
                                fire_typed_event(
                                    effect.clone(),
                                    targets.clone(),
                                    chip_name.clone(),
                                    commands,
                                );
                            }
                            // Non-Do grandchildren (Arm equivalent) — treat as matched
                        }
                    }
                }
                if any_matched {
                    consumed_indices.push(i);
                }
            }
            _ => {
                // Regular nodes — evaluate directly
                if let Some(children) = evaluate_node(trigger_kind, node) {
                    for child in children {
                        if let EffectNode::Do(effect) = child {
                            fire_typed_event(
                                effect.clone(),
                                targets.clone(),
                                chip_name.clone(),
                                commands,
                            );
                        }
                        // Non-Do children (Arm equivalent) — discarded in entity chains
                    }
                }
            }
        }
    }

    // Remove consumed Once nodes in reverse order
    for &i in consumed_indices.iter().rev() {
        chains.0.remove(i);
    }
}

/// Sweep all entities for a bump trigger, filtering by grade.
///
/// Reads `BumpPerformed` messages, optionally filters by `BumpGrade`, sweeps ALL
/// entities with `EffectChains`, and evaluates `ArmedEffects` on the specific bolt.
pub(super) fn bridge_global_bump_inner(
    reader: &mut MessageReader<BumpPerformed>,
    chains_query: &mut Query<&mut EffectChains>,
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    grade: Option<BumpGrade>,
    trigger: Trigger,
) {
    for performed in reader.read() {
        if let Some(g) = grade
            && performed.grade != g
        {
            continue;
        }
        let targets = performed
            .bolt
            .map_or(vec![], |b| vec![EffectTarget::Entity(b)]);

        for mut chains in chains_query.iter_mut() {
            evaluate_entity_chains(&mut chains, trigger, targets.clone(), commands);
        }

        if let Some(bolt) = performed.bolt {
            evaluate_armed(armed_query, commands, bolt, trigger);
        }
    }
}

/// Evaluate a specific bolt for a bumped trigger, filtering by grade.
///
/// Reads `BumpPerformed` messages, optionally filters by `BumpGrade`, evaluates
/// ONLY the specific bolt entity's `EffectChains` and `ArmedEffects`.
pub(super) fn bridge_targeted_bumped_inner(
    reader: &mut MessageReader<BumpPerformed>,
    chains_query: &mut Query<&mut EffectChains>,
    armed_query: &mut Query<&mut ArmedEffects>,
    commands: &mut Commands,
    grade: Option<BumpGrade>,
    trigger: Trigger,
) {
    for performed in reader.read() {
        if let Some(g) = grade
            && performed.grade != g
        {
            continue;
        }
        let Some(bolt) = performed.bolt else {
            continue;
        };
        let targets = vec![EffectTarget::Entity(bolt)];

        if let Ok(mut chains) = chains_query.get_mut(bolt) {
            evaluate_entity_chains(&mut chains, trigger, targets.clone(), commands);
        }

        evaluate_armed(armed_query, commands, bolt, trigger);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{
        armed::ArmedEffects,
        definition::{Effect, EffectChains, EffectNode, EffectTarget, ImpactTarget, Trigger},
        triggers::test_helpers::{CapturedShockwaveFired, capture_shockwave_fired},
    };

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // =========================================================================
    // H1a: evaluate_entity_chains preserves non-matching chains
    // =========================================================================

    /// System wrapper: evaluates entity chains for `Trigger::Bump` on all entities
    /// with `EffectChains`.
    fn sys_evaluate_bump(
        mut query: Query<&mut EffectChains>,
        mut commands: Commands,
    ) {
        for mut chains in &mut query {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Bump,
                vec![],
                &mut commands,
            );
        }
    }

    #[test]
    fn evaluate_entity_chains_preserves_non_matching_chains() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, sys_evaluate_bump);

        // Entity with 2 chains: When(Bump) and When(Impact(Cell))
        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![
                (
                    None,
                    EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                    },
                ),
                (
                    None,
                    EffectNode::When {
                        trigger: Trigger::Impact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
                    },
                ),
            ]))
            .id();

        tick(&mut app);

        // ShockwaveFired(64.0) should fire for the Bump match
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the Bump chain should fire — got {}",
            captured.0.len()
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "should fire shockwave with base_range 64.0"
        );

        // EffectChains is the permanent source of truth — BOTH chains are preserved
        // (matching chains fire but are NOT consumed; only Once wrappers are consumed)
        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(
            chains.0.len(),
            2,
            "both chains should be retained (EffectChains is permanent) — got {}",
            chains.0.len()
        );
    }

    // =========================================================================
    // H1b: evaluate_entity_chains unwraps Once(When) on matching trigger
    // =========================================================================

    fn sys_evaluate_bolt_lost(
        mut query: Query<&mut EffectChains>,
        mut commands: Commands,
    ) {
        for mut chains in &mut query {
            evaluate_entity_chains(
                &mut chains,
                Trigger::BoltLost,
                vec![],
                &mut commands,
            );
        }
    }

    #[test]
    fn evaluate_entity_chains_unwraps_once_on_matching_trigger() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, sys_evaluate_bolt_lost);

        // Entity with Once([When(BoltLost, [Do(Shockwave(64.0))])])
        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }]),
            )]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "ShockwaveFired should fire when Once(When(BoltLost)) matches BoltLost trigger"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "should fire shockwave with base_range 64.0"
        );

        // EffectChains should be empty (Once consumed)
        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be consumed after matching — got {} entries",
            chains.0.len()
        );
    }

    // =========================================================================
    // H1c: evaluate_entity_chains preserves Once(When) on non-matching trigger
    // =========================================================================

    #[test]
    fn evaluate_entity_chains_preserves_once_on_non_matching_trigger() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, sys_evaluate_bump);

        // Entity with Once([When(BoltLost, [Do(Shockwave(64.0))])])
        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }]),
            )]))
            .id();

        // sys_evaluate_bump evaluates against Trigger::Bump — should NOT match BoltLost
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "no ShockwaveFired should fire when trigger does not match Once contents"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "Once node should be preserved when trigger does not match — got {}",
            chains.0.len()
        );
    }

    // =========================================================================
    // H1d: resolve_armed re-arms non-Do children
    // =========================================================================

    /// System wrapper: evaluates armed effects for `Trigger::Impact(Cell)` on all
    /// entities with `ArmedEffects`.
    fn sys_resolve_armed_impact_cell(
        mut query: Query<(Entity, &mut ArmedEffects)>,
        mut commands: Commands,
    ) {
        for (entity, mut armed) in &mut query {
            let targets = vec![EffectTarget::Entity(entity)];
            resolve_armed(&mut armed, Trigger::Impact(ImpactTarget::Cell), targets, &mut commands);
        }
    }

    #[test]
    fn resolve_armed_re_arms_non_do_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, sys_resolve_armed_impact_cell);

        // ArmedEffects with When(Impact(Cell), [When(CellDestroyed, [Do(Shockwave(64.0))])])
        let entity = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::When {
                        trigger: Trigger::CellDestroyed,
                        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                    }],
                },
            )]))
            .id();

        tick(&mut app);

        // No ShockwaveFired — inner child is When, not Do
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "no ShockwaveFired should fire when inner child is When, not Do"
        );

        // ArmedEffects should have 1 re-armed entry: When(CellDestroyed, [Do(Shockwave(64.0))])
        let armed = app.world().get::<ArmedEffects>(entity).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "should have 1 re-armed entry — got {}",
            armed.0.len()
        );
        assert!(
            matches!(
                &armed.0[0].1,
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    ..
                }
            ),
            "re-armed entry should be When(CellDestroyed, ...) — got {:?}",
            armed.0[0].1
        );
    }

    // =========================================================================
    // H1e: resolve_armed retains non-matching entries
    // =========================================================================

    #[test]
    fn resolve_armed_retains_non_matching_and_fires_matching() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, sys_resolve_armed_impact_cell);

        // ArmedEffects with 2 entries:
        // When(Impact(Cell), [Do(Shockwave(64.0))]) — matches
        // When(CellDestroyed, [Do(Shockwave(32.0))]) — does NOT match
        let entity = app
            .world_mut()
            .spawn(ArmedEffects(vec![
                (
                    None,
                    EffectNode::When {
                        trigger: Trigger::Impact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                    },
                ),
                (
                    None,
                    EffectNode::When {
                        trigger: Trigger::CellDestroyed,
                        then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
                    },
                ),
            ]))
            .id();

        tick(&mut app);

        // ShockwaveFired(64.0) should fire for the matching entry
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the Impact(Cell) match should fire — got {}",
            captured.0.len()
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "should fire shockwave with base_range 64.0"
        );

        // ArmedEffects should retain the non-matching CellDestroyed entry
        let armed = app.world().get::<ArmedEffects>(entity).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "non-matching CellDestroyed entry should be retained — got {}",
            armed.0.len()
        );
        assert!(
            matches!(
                &armed.0[0].1,
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    ..
                }
            ),
            "retained entry should be When(CellDestroyed, ...) — got {:?}",
            armed.0[0].1
        );
    }
}

/// Evaluates `When` children inside a bolt's `UntilTimers` and `UntilTriggers` entries.
///
/// Until entries contain children that may include `When` nodes. These `When` nodes
/// should be evaluated by bridge systems against the current trigger, firing effects
/// when matched. The Until entry itself is not consumed — only its children are
/// evaluated for the current trigger kind.
pub(super) fn evaluate_until_children(
    until_query: &Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    bolt_entity: Entity,
    trigger_kind: Trigger,
    targets: &[EffectTarget],
    commands: &mut Commands,
) {
    let Ok((until_timers, until_triggers)) = until_query.get(bolt_entity) else {
        return;
    };

    if let Some(timers) = until_timers {
        for entry in &timers.0 {
            for child in &entry.children {
                if let Some(children) = evaluate_node(trigger_kind, child) {
                    for c in children {
                        if let EffectNode::Do(effect) = c {
                            fire_typed_event(effect.clone(), targets.to_vec(), None, commands);
                        }
                    }
                }
            }
        }
    }

    if let Some(triggers) = until_triggers {
        for entry in &triggers.0 {
            for child in &entry.children {
                if let Some(children) = evaluate_node(trigger_kind, child) {
                    for c in children {
                        if let EffectNode::Do(effect) = c {
                            fire_typed_event(effect.clone(), targets.to_vec(), None, commands);
                        }
                    }
                }
            }
        }
    }
}
