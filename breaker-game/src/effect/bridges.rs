//! Per-trigger bridge systems — translate messages into effect events.
//!
//! Each bridge reads one message type, evaluates all active chains,
//! and either fires typed events or arms the bolt with `ArmedEffects`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    active::ActiveEffects,
    armed::ArmedEffects,
    definition::{EffectChains, EffectNode, EffectTarget, ImpactTarget, Trigger},
    effect_nodes::until::{UntilTimers, UntilTriggers},
    evaluate::{NodeEvalResult, evaluate_node},
    typed_events::fire_typed_event,
};
use crate::{
    bolt::messages::{
        BoltDestroyedAt, BoltHitBreaker, BoltHitCell, BoltHitWall, BoltLost, RequestBoltDestroyed,
    },
    breaker::{
        components::Breaker,
        messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    },
    cells::{
        components::RequiredToClear,
        messages::{CellDestroyedAt, RequestCellDestroyed},
    },
    run::node::resources::NodeTimer,
};

/// Bridge for `BoltLost` — evaluates chains when a bolt is lost.
///
/// Global trigger: evaluates active chains once per frame (not per message)
/// and evaluates armed triggers on ALL bolt entities.
/// Also evaluates breaker entity `EffectChains` (for `SecondWind` etc.).
pub(crate) fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = Trigger::BoltLost;
    evaluate_active_chains(&active, trigger_kind, vec![], &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);

    // Evaluate breaker entity EffectChains
    for mut chains in &mut breaker_query {
        evaluate_entity_chains(&mut chains, trigger_kind, vec![], &mut commands);
    }
}

/// Bridge for `BumpPerformed` — evaluates chains on bump.
///
/// For each bump message, evaluates two trigger kinds:
/// 1. Grade-specific: Perfect→`PerfectBump`, Early→`EarlyBump`, Late→`LateBump`
/// 2. `Bump`: all non-whiff bumps evaluate `Bump` chains.
///
/// Also evaluates breaker entity `EffectChains`.
pub(crate) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut bolt_chains_query: Query<&mut EffectChains, Without<Breaker>>,
    mut commands: Commands,
) {
    for performed in reader.read() {
        let grade_trigger = match performed.grade {
            BumpGrade::Perfect => Trigger::PerfectBump,
            BumpGrade::Early => Trigger::EarlyBump,
            BumpGrade::Late => Trigger::LateBump,
        };

        // Bolt-targeted evaluation requires a real bolt entity
        if let Some(bolt_entity) = performed.bolt {
            let targets = vec![EffectTarget::Entity(bolt_entity)];

            for (chip_name, chain) in &active.0 {
                // Grade-specific evaluation
                for result in evaluate_node(grade_trigger, chain) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(
                                effect,
                                targets.clone(),
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                        NodeEvalResult::Arm(remaining) => {
                            arm_bolt(
                                &mut armed_query,
                                &mut commands,
                                bolt_entity,
                                chip_name.clone(),
                                remaining,
                            );
                        }
                        NodeEvalResult::NoMatch => {}
                    }
                }
                // BumpSuccess evaluation (all grades)
                for result in evaluate_node(Trigger::Bump, chain) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(
                                effect,
                                targets.clone(),
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                        NodeEvalResult::Arm(remaining) => {
                            arm_bolt(
                                &mut armed_query,
                                &mut commands,
                                bolt_entity,
                                chip_name.clone(),
                                remaining,
                            );
                        }
                        NodeEvalResult::NoMatch => {}
                    }
                }
            }

            // Evaluate armed triggers on the specific bolt
            evaluate_armed(&mut armed_query, &mut commands, bolt_entity, grade_trigger);
            evaluate_armed(&mut armed_query, &mut commands, bolt_entity, Trigger::Bump);

            // Evaluate breaker entity EffectChains (bolt-targeted)
            for mut chains in &mut breaker_query {
                evaluate_entity_chains(&mut chains, grade_trigger, targets.clone(), &mut commands);
                evaluate_entity_chains(&mut chains, Trigger::Bump, targets.clone(), &mut commands);
            }

            // Evaluate bolt entity EffectChains with Bumped triggers
            if let Ok(mut bolt_chains) = bolt_chains_query.get_mut(bolt_entity) {
                let bumped_grade = match performed.grade {
                    BumpGrade::Perfect => Trigger::PerfectBumped,
                    BumpGrade::Early => Trigger::EarlyBumped,
                    BumpGrade::Late => Trigger::LateBumped,
                };
                evaluate_entity_chains(
                    &mut bolt_chains,
                    bumped_grade,
                    targets.clone(),
                    &mut commands,
                );
                evaluate_entity_chains(&mut bolt_chains, Trigger::Bumped, targets, &mut commands);
            }
        } else {
            // No bolt — still evaluate breaker entity EffectChains with empty targets
            for mut chains in &mut breaker_query {
                evaluate_entity_chains(&mut chains, grade_trigger, vec![], &mut commands);
                evaluate_entity_chains(&mut chains, Trigger::Bump, vec![], &mut commands);
            }
        }
    }
}

/// Bridge for `NoBump` — fires when `BoltHitBreaker` arrives without
/// any `BumpPerformed` in the same frame.
///
/// Evaluates active chains and breaker entity `EffectChains` with
/// `Trigger::NoBump`.
pub(crate) fn bridge_no_bump(
    mut hit_reader: MessageReader<BoltHitBreaker>,
    mut bump_reader: MessageReader<BumpPerformed>,
    active: Res<ActiveEffects>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut commands: Commands,
) {
    let hit_count = hit_reader.read().count();
    let bump_count = bump_reader.read().count();

    if hit_count == 0 || bump_count > 0 {
        return;
    }

    evaluate_active_chains(&active, Trigger::NoBump, vec![], &mut commands);

    for mut chains in &mut breaker_query {
        evaluate_entity_chains(&mut chains, Trigger::NoBump, vec![], &mut commands);
    }
}

/// Bridge for `BumpWhiffed` — evaluates chains when a bump whiffs.
///
/// Global trigger: evaluates active chains once per frame and evaluates
/// armed triggers on ALL bolt entities.
pub(crate) fn bridge_bump_whiff(
    mut reader: MessageReader<BumpWhiffed>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = Trigger::BumpWhiff;
    evaluate_active_chains(&active, trigger_kind, vec![], &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Bridge for `BoltHitCell` — evaluates chains and armed triggers on cell impact.
/// Also evaluates bolt entity `EffectChains` and `When` children inside
/// `UntilTimers`/`UntilTriggers`.
pub(crate) fn bridge_cell_impact(
    mut reader: MessageReader<BoltHitCell>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        for (chip_name, chain) in &active.0 {
            for result in evaluate_node(Trigger::Impact(ImpactTarget::Cell), chain) {
                match result {
                    NodeEvalResult::Fire(effect) => {
                        fire_typed_event(effect, targets.clone(), chip_name.clone(), &mut commands);
                    }
                    NodeEvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    NodeEvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Cell),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Cell),
                targets.clone(),
                &mut commands,
            );
        }

        // Evaluate When children inside UntilTimers/UntilTriggers
        evaluate_until_children(
            &until_query,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Cell),
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitBreaker` — evaluates chains and armed triggers on
/// breaker impact. Also evaluates bolt entity `EffectChains`.
pub(crate) fn bridge_breaker_impact(
    mut reader: MessageReader<BoltHitBreaker>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        for (chip_name, chain) in &active.0 {
            for result in evaluate_node(Trigger::Impact(ImpactTarget::Breaker), chain) {
                match result {
                    NodeEvalResult::Fire(effect) => {
                        fire_typed_event(effect, targets.clone(), chip_name.clone(), &mut commands);
                    }
                    NodeEvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    NodeEvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Breaker),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Breaker),
                targets,
                &mut commands,
            );
        }
    }
}

/// Bridge for `BoltHitWall` — evaluates chains and armed triggers on
/// wall impact. Also evaluates bolt entity `EffectChains` and `When`
/// children inside `UntilTimers`/`UntilTriggers`.
pub(crate) fn bridge_wall_impact(
    mut reader: MessageReader<BoltHitWall>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];
        for (chip_name, chain) in &active.0 {
            for result in evaluate_node(Trigger::Impact(ImpactTarget::Wall), chain) {
                match result {
                    NodeEvalResult::Fire(effect) => {
                        fire_typed_event(effect, targets.clone(), chip_name.clone(), &mut commands);
                    }
                    NodeEvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    NodeEvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Wall),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Wall),
                targets.clone(),
                &mut commands,
            );
        }

        // Evaluate When children inside UntilTimers/UntilTriggers
        evaluate_until_children(
            &until_query,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Wall),
            &targets,
            &mut commands,
        );
    }
}

/// Evaluates `Trigger::Death` on an entity's `EffectChains` and fires
/// any matching leaf effects. Shared by `bridge_cell_death` and `bridge_bolt_death`.
fn evaluate_ondeath_chains(entity: Entity, chains: Option<&EffectChains>, commands: &mut Commands) {
    if let Some(chains) = chains {
        let targets = vec![EffectTarget::Entity(entity)];
        for node in &chains.0 {
            for result in evaluate_node(Trigger::Death, node) {
                if let NodeEvalResult::Fire(effect) = result {
                    fire_typed_event(effect, targets.clone(), None, commands);
                }
            }
        }
    }
}

/// Bridge for `RequestCellDestroyed` — evaluates cell's `EffectChains` with
/// `Trigger::Death` while the entity is still alive, then writes
/// `CellDestroyedAt` with position and required-to-clear data.
///
/// Also evaluates `Trigger::CellDestroyed` active chains and armed triggers.
pub(crate) fn bridge_cell_death(
    mut reader: MessageReader<RequestCellDestroyed>,
    cell_query: Query<(Option<&EffectChains>, &Position2D, Has<RequiredToClear>)>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut destroyed_writer: MessageWriter<CellDestroyedAt>,
    mut commands: Commands,
) {
    let mut any_destroyed = false;
    for msg in reader.read() {
        let Ok((chains, position, is_required)) = cell_query.get(msg.cell) else {
            continue;
        };

        any_destroyed = true;
        evaluate_ondeath_chains(msg.cell, chains, &mut commands);

        destroyed_writer.write(CellDestroyedAt {
            position: position.0,
            was_required_to_clear: is_required,
        });
    }

    if any_destroyed {
        evaluate_active_chains(&active, Trigger::CellDestroyed, vec![], &mut commands);
        evaluate_armed_all(armed_query, Trigger::CellDestroyed, &mut commands);
    }
}

/// Bridge for `RequestBoltDestroyed` — evaluates bolt's `EffectChains` with
/// `Trigger::Death` while the entity is still alive, then writes
/// `BoltDestroyedAt` with position data.
pub(crate) fn bridge_bolt_death(
    mut reader: MessageReader<RequestBoltDestroyed>,
    bolt_query: Query<(Option<&EffectChains>, &Position2D)>,
    mut destroyed_writer: MessageWriter<BoltDestroyedAt>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok((chains, position)) = bolt_query.get(msg.bolt) else {
            continue;
        };

        evaluate_ondeath_chains(msg.bolt, chains, &mut commands);

        destroyed_writer.write(BoltDestroyedAt {
            position: position.0,
        });
    }
}

/// Bridge for `NodeTimer` — fires `When(NodeTimerThreshold(t))` chains
/// when the timer ratio crosses below the threshold. Fires once only.
pub(crate) fn bridge_timer_threshold(
    timer: Res<NodeTimer>,
    mut active: ResMut<ActiveEffects>,
    mut commands: Commands,
) {
    // Early return if no threshold chains exist
    let has_threshold = active.0.iter().any(|(_, chain)| {
        matches!(
            chain,
            EffectNode::When {
                trigger: Trigger::NodeTimerThreshold(_),
                ..
            }
        )
    });
    if !has_threshold {
        return;
    }

    let ratio = if timer.total == 0.0 {
        0.0
    } else {
        timer.remaining / timer.total
    };

    // Find and fire matching threshold chains, then remove them (fire-once).
    let mut indices_to_remove = Vec::new();
    for (i, (_chip_name, chain)) in active.0.iter().enumerate() {
        if let EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(threshold),
            then,
        } = chain
            && ratio < *threshold
        {
            // Fire children
            for child in then {
                if let EffectNode::Do(effect) = child {
                    fire_typed_event(effect.clone(), vec![], None, &mut commands);
                }
                // Non-leaf children from timer threshold — skip for now
            }
            indices_to_remove.push(i);
        }
    }

    // Remove fired chains in reverse order to preserve indices
    for &i in indices_to_remove.iter().rev() {
        active.0.remove(i);
    }
}

/// Cleanup system for cells — despawns cell entities from
/// `RequestCellDestroyed` messages. Runs after all bridges.
pub(crate) fn cleanup_destroyed_cells(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.cell).despawn();
    }
}

/// Cleanup system for bolts — despawns bolt entities from
/// `RequestBoltDestroyed` messages. Runs after all bridges.
pub(crate) fn cleanup_destroyed_bolts(
    mut reader: MessageReader<RequestBoltDestroyed>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.bolt).despawn();
    }
}

/// Processes `Once` nodes wrapping bare `Do` children at chip selection time.
/// Fires the effect and removes the `Once` wrapper from `EffectChains`.
/// Once nodes wrapping `When` nodes are left for bridge evaluation.
pub(crate) fn apply_once_nodes(mut query: Query<&mut EffectChains>, mut commands: Commands) {
    for mut chains in &mut query {
        chains.0.retain(|node| {
            if let EffectNode::Once(children) = node {
                // Check if all children are bare Do nodes
                let all_bare_do = children.iter().all(|c| matches!(c, EffectNode::Do(_)));
                if all_bare_do && !children.is_empty() {
                    // Fire all bare Do children
                    for child in children {
                        if let EffectNode::Do(effect) = child {
                            fire_typed_event(effect.clone(), vec![], None, &mut commands);
                        }
                    }
                    return false; // Remove the Once node
                }
            }
            true // Keep non-Once nodes and Once nodes wrapping When
        });
    }
}

/// Evaluates entity-local `EffectChains` against a trigger kind.
///
/// Handles `Once` node unwrapping: if a `Once` wraps children that match the
/// trigger, the effects are fired and the `Once` is consumed (removed).
/// If no children match, the `Once` is preserved.
fn evaluate_entity_chains(
    chains: &mut EffectChains,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut consumed_indices = Vec::new();

    for (i, node) in chains.0.iter().enumerate() {
        match node {
            EffectNode::Once(children) => {
                // Unwrap the Once: evaluate inner children against the trigger
                let mut any_matched = false;
                for child in children {
                    for result in evaluate_node(trigger_kind, child) {
                        match result {
                            NodeEvalResult::Fire(effect) => {
                                any_matched = true;
                                fire_typed_event(effect, targets.clone(), None, commands);
                            }
                            NodeEvalResult::Arm(_remaining) => {
                                // Arming from Once children — treat as matched
                                any_matched = true;
                            }
                            NodeEvalResult::NoMatch => {}
                        }
                    }
                }
                if any_matched {
                    consumed_indices.push(i);
                }
            }
            _ => {
                // Regular nodes — evaluate directly
                for result in evaluate_node(trigger_kind, node) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(effect, targets.clone(), None, commands);
                        }
                        NodeEvalResult::Arm(_) | NodeEvalResult::NoMatch => {}
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

/// Evaluates all active chains against a trigger kind.
///
/// `Arm` results are intentionally discarded for global triggers — only `Fire`
/// results are actioned. Arming requires a specific bolt entity, which global
/// triggers (cell destroyed, bolt lost, bump whiff) don't provide.
fn evaluate_active_chains(
    active: &ActiveEffects,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    for (chip_name, chain) in &active.0 {
        for result in evaluate_node(trigger_kind, chain) {
            match result {
                NodeEvalResult::Fire(effect) => {
                    fire_typed_event(effect, targets.clone(), chip_name.clone(), commands);
                }
                NodeEvalResult::Arm(_) | NodeEvalResult::NoMatch => {}
            }
        }
    }
}

/// Evaluates armed triggers on all bolt entities that have `ArmedEffects`.
fn evaluate_armed_all(
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
fn arm_bolt(
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
fn evaluate_armed(
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
fn resolve_armed(
    armed: &mut ArmedEffects,
    trigger_kind: Trigger,
    targets: Vec<EffectTarget>,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for (chip_name, chain) in armed.0.drain(..) {
        let results = evaluate_node(trigger_kind, &chain);
        let mut matched = false;
        for result in results {
            match result {
                NodeEvalResult::Fire(effect) => {
                    matched = true;
                    fire_typed_event(effect, targets.clone(), chip_name.clone(), commands);
                }
                NodeEvalResult::Arm(next) => {
                    matched = true;
                    new_armed.push((chip_name.clone(), next));
                }
                NodeEvalResult::NoMatch => {}
            }
        }
        if !matched {
            new_armed.push((chip_name, chain));
        }
    }
    armed.0 = new_armed;
}

/// Evaluates `When` children inside a bolt's `UntilTimers` and `UntilTriggers` entries.
///
/// Until entries contain children that may include `When` nodes. These `When` nodes
/// should be evaluated by bridge systems against the current trigger, firing effects
/// when matched. The Until entry itself is not consumed — only its children are
/// evaluated for the current trigger kind.
fn evaluate_until_children(
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
                for result in evaluate_node(trigger_kind, child) {
                    if let NodeEvalResult::Fire(effect) = result {
                        fire_typed_event(effect, targets.to_vec(), None, commands);
                    }
                }
            }
        }
    }

    if let Some(triggers) = until_triggers {
        for entry in &triggers.0 {
            for child in &entry.children {
                for result in evaluate_node(trigger_kind, child) {
                    if let NodeEvalResult::Fire(effect) = result {
                        fire_typed_event(effect, targets.to_vec(), None, commands);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::messages::BumpGrade,
        effect::{
            definition::{Effect, EffectNode, ImpactTarget, Trigger},
            typed_events::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedShockwaveFired(Vec<ShockwaveFired>);

    fn capture_shockwave_fired(
        trigger: On<ShockwaveFired>,
        mut captured: ResMut<CapturedShockwaveFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedLoseLifeFired(Vec<LoseLifeFired>);

    fn capture_lose_life_fired(
        trigger: On<LoseLifeFired>,
        mut captured: ResMut<CapturedLoseLifeFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpeedBoostFired(Vec<SpeedBoostFired>);

    fn capture_speed_boost_fired(
        trigger: On<SpeedBoostFired>,
        mut captured: ResMut<CapturedSpeedBoostFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedShieldFired(Vec<ShieldFired>);

    fn capture_shield_fired(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShieldFired>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedTimePenaltyFired(Vec<TimePenaltyFired>);

    fn capture_time_penalty_fired(
        trigger: On<TimePenaltyFired>,
        mut captured: ResMut<CapturedTimePenaltyFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedMultiBoltFired(Vec<MultiBoltFired>);

    fn capture_multi_bolt_fired(
        trigger: On<MultiBoltFired>,
        mut captured: ResMut<CapturedMultiBoltFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource)]
    struct SendBoltLostFlag(bool);

    fn send_bolt_lost(flag: Res<SendBoltLostFlag>, mut writer: MessageWriter<BoltLost>) {
        if flag.0 {
            writer.write(BoltLost);
        }
    }

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBumpWhiffFlag(bool);

    fn send_bump_whiff(flag: Res<SendBumpWhiffFlag>, mut writer: MessageWriter<BumpWhiffed>) {
        if flag.0 {
            writer.write(BumpWhiffed);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitCell(Option<BoltHitCell>);

    fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
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

    #[derive(Resource)]
    struct SendBoltHitWall(Option<BoltHitWall>);

    fn send_bolt_hit_wall(msg: Res<SendBoltHitWall>, mut writer: MessageWriter<BoltHitWall>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendCellDestroyed(Option<RequestCellDestroyed>);

    fn send_cell_destroyed(
        msg: Res<SendCellDestroyed>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Per-bridge test app builders ---

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `ActiveEffects`.
    fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    fn bolt_lost_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedLoseLifeFired>()
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_lose_life_fired)
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
        app
    }

    fn bump_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedShieldFired>()
            .init_resource::<CapturedLoseLifeFired>()
            .init_resource::<CapturedTimePenaltyFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_shield_fired)
            .add_observer(capture_lose_life_fired)
            .add_observer(capture_time_penalty_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
        app
    }

    fn bump_whiff_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBumpWhiffFlag(false))
            .init_resource::<CapturedLoseLifeFired>()
            .add_observer(capture_lose_life_fired)
            .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());
        app
    }

    fn cell_impact_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );
        app
    }

    fn breaker_impact_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedShieldFired>()
            .init_resource::<CapturedMultiBoltFired>()
            .add_observer(capture_shield_fired)
            .add_observer(capture_multi_bolt_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );
        app
    }

    fn wall_impact_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedShieldFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_shield_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );
        app
    }

    fn cell_destroyed_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_cell_destroyed, bridge_cell_death).chain(),
            );
        app
    }

    // --- Bolt lost bridge tests ---

    #[test]
    fn bolt_lost_fires_active_chains() {
        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = bolt_lost_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert_eq!(captured.0.len(), 1);
        assert!(captured.0[0].targets.is_empty());
    }

    #[test]
    fn bolt_lost_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = bolt_lost_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert!(captured.0.is_empty());
    }

    // --- Bump bridge tests ---

    #[test]
    fn perfect_bump_fires_on_perfect_bump_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0));
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
        assert_eq!(captured.0[0].targets, vec![EffectTarget::Entity(bolt)]);
    }

    #[test]
    fn perfect_bump_fires_both_on_perfect_bump_and_on_bump_success() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0)),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "perfect bump should fire ShockwaveFired from PerfectBump chain"
        );
        assert_eq!(
            shields.0.len(),
            1,
            "perfect bump should fire ShieldFired from Bump chain"
        );
    }

    #[test]
    fn early_bump_fires_on_early_bump_and_on_bump_success_but_not_on_perfect_bump() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0)),
            EffectNode::trigger_leaf(Trigger::EarlyBump, Effect::LoseLife),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let lose_life = app.world().resource::<CapturedLoseLifeFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            lose_life.0.len(),
            1,
            "early bump should fire LoseLifeFired from EarlyBump chain"
        );
        assert_eq!(
            shields.0.len(),
            1,
            "early bump should fire ShieldFired from Bump chain"
        );
        assert!(
            shockwaves.0.is_empty(),
            "early bump should NOT fire ShockwaveFired from PerfectBump chain"
        );
    }

    #[test]
    fn late_bump_fires_on_late_bump_and_on_bump_success() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::LateBump, Effect::test_time_penalty(3.0)),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Late,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let time_penalty = app.world().resource::<CapturedTimePenaltyFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        assert_eq!(time_penalty.0.len(), 1);
        assert!((time_penalty.0[0].seconds - 3.0).abs() < f32::EPSILON);
        assert_eq!(shields.0.len(), 1);
    }

    #[test]
    fn perfect_bump_with_non_leaf_arms_bolt() {
        let chain = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        };
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty(), "non-leaf inner should arm, not fire");

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(armed.0.len(), 1);
        assert_eq!(
            armed.0[0].1,
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))]
            }
        );
    }

    // --- BumpWhiff bridge tests ---

    #[test]
    fn bump_whiff_fires_on_bump_whiff_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::LoseLife);
        let mut app = bump_whiff_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert_eq!(captured.0.len(), 1);
        assert!(captured.0[0].targets.is_empty());
    }

    #[test]
    fn bump_whiff_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::LoseLife);
        let mut app = bump_whiff_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert!(captured.0.is_empty());
    }

    // --- Cell impact bridge tests ---

    #[test]
    fn cell_impact_fires_active_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Cell),
            Effect::test_shockwave(64.0),
        );
        let mut app = cell_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_impact_fires_armed_trigger() {
        let mut app = cell_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Cell),
                    Effect::test_shockwave(64.0),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert!(armed.0.is_empty());
    }

    #[test]
    fn cell_impact_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Cell),
            Effect::test_shockwave(64.0),
        );
        let mut app = cell_impact_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty());
    }

    // --- Breaker impact bridge tests ---

    #[test]
    fn breaker_impact_fires_active_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Breaker),
            Effect::test_shield(5.0),
        );
        let mut app = breaker_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShieldFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_duration - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_impact_fires_armed_trigger() {
        let mut app = breaker_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Breaker),
                    Effect::test_multi_bolt(2),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedMultiBoltFired>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].base_count, 2);
    }

    // --- Wall impact bridge tests ---

    #[test]
    fn wall_impact_fires_active_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Wall),
            Effect::test_shockwave(32.0),
        );
        let mut app = wall_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wall_impact_fires_armed_trigger() {
        let mut app = wall_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Wall),
                    Effect::test_shield(5.0),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShieldFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_duration - 5.0).abs() < f32::EPSILON);
    }

    // --- Cell destroyed bridge tests ---

    #[test]
    fn cell_destroyed_fires_active_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(32.0));
        let mut app = cell_destroyed_test_app(vec![chain]);
        // Spawn a cell entity with Position2D for bridge_cell_death to query
        let cell = app
            .world_mut()
            .spawn((Position2D(Vec2::new(10.0, 20.0)), RequiredToClear))
            .id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 32.0).abs() < f32::EPSILON);
        assert!(captured.0[0].targets.is_empty());
    }

    #[test]
    fn cell_destroyed_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::CellDestroyed, Effect::test_shockwave(32.0));
        let mut app = cell_destroyed_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty());
    }

    // --- Full two-step chain tests ---

    #[test]
    fn full_two_step_chain_bump_arms_then_impact_fires() {
        let chain = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_bump,
                    send_bolt_hit_cell,
                    bridge_cell_impact,
                )
                    .chain(),
            );

        let bolt = app.world_mut().spawn_empty().id();

        // Step 1: Perfect bump -- arms
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty(), "step 1: should arm, not fire");
        assert!(
            app.world().get::<ArmedEffects>(bolt).is_some(),
            "step 1: bolt should be armed"
        );

        app.world_mut().resource_mut::<SendBump>().0 = None;

        // Step 2: Cell impact -- fires
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn full_three_step_chain_bump_arms_impact_rearms_cell_destroyed_fires() {
        let chain = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_bump,
                    send_bolt_hit_cell,
                    bridge_cell_impact,
                    send_cell_destroyed,
                    bridge_cell_death,
                )
                    .chain(),
            );

        let bolt = app.world_mut().spawn_empty().id();

        // Step 1: Perfect bump — arms
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "step 1: should arm, not fire any effect"
        );
        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "step 1: bolt should have exactly one armed trigger"
        );

        // Clear bump, prepare cell impact
        app.world_mut().resource_mut::<SendBump>().0 = None;
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        // Step 2: Cell impact — re-arms
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "step 2: should re-arm, not fire any effect"
        );
        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "step 2: bolt should have exactly one armed trigger"
        );

        // Clear cell hit, prepare cell destroyed — spawn entity for bridge_cell_death to query
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = None;
        let cell = app
            .world_mut()
            .spawn((Position2D(Vec2::new(10.0, 20.0)), RequiredToClear))
            .id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        // Step 3: Cell destroyed — fires the shockwave
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "step 3: shockwave should fire after cell destroyed"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "step 3: fired effect should be a shockwave with base_range 64.0"
        );
    }

    // --- Integration: bridge + effect observer ---

    #[test]
    fn bridge_bolt_lost_plus_life_lost_observer_decrements_lives() {
        use crate::{
            effect::effects::life_lost::{LivesCount, handle_life_lost},
            run::messages::RunLost,
        };

        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .add_message::<RunLost>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBoltLostFlag(false))
            .add_observer(handle_life_lost)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        let entity = app.world_mut().spawn(LivesCount(3)).id();
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "bolt lost should decrement LivesCount via unified bridge"
        );
    }

    // =========================================================================
    // B12b: Bridge evaluation with EffectNode types (behaviors 23-24)
    // =========================================================================

    #[test]
    fn evaluate_node_fires_effect_for_bolt_lost_bridge() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, evaluate_node},
        };

        let node = EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let result = evaluate_node(Trigger::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::Shockwave {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
            "bridge_bolt_lost should get Fire(Shockwave) from evaluate_node"
        );
    }

    #[test]
    fn evaluate_node_arms_effect_node_for_bump_bridge_non_leaf() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, evaluate_node},
        };

        let inner_node = EffectNode::When {
            trigger: Trigger::Impact(super::super::definition::ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![inner_node.clone()],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Arm(inner_node)],
            "PerfectBump with non-leaf child should Arm the inner node"
        );
    }

    #[test]
    fn evaluate_node_no_match_for_wrong_trigger() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, evaluate_node},
        };

        let node = EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature A2: NodeTimerThreshold bridge (behaviors 13-16)
    // =========================================================================

    #[test]
    fn bridge_timer_threshold_fires_when_ratio_crosses_below_threshold() {
        use crate::run::node::resources::NodeTimer;

        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.25),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                target: super::super::definition::Target::Bolt,
                multiplier: 2.0,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 14.9,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "timer ratio 14.9/60.0 = 0.248 < 0.25 should fire"
        );
        assert!(
            (captured.0[0].multiplier - 2.0).abs() < f32::EPSILON,
            "fired effect should have multiplier 2.0"
        );
    }

    #[test]
    fn bridge_timer_threshold_no_fire_when_ratio_above_threshold() {
        use crate::run::node::resources::NodeTimer;

        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.25),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                target: super::super::definition::Target::Bolt,
                multiplier: 2.0,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 30.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert!(
            captured.0.is_empty(),
            "timer ratio 30/60 = 0.5 > 0.25 should NOT fire"
        );
    }

    #[test]
    fn bridge_timer_threshold_fires_once_even_if_ratio_stays_below() {
        use crate::run::node::resources::NodeTimer;

        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                target: super::super::definition::Target::Bolt,
                multiplier: 1.5,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 12.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        // First tick: fires
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(captured.0.len(), 1, "first tick should fire");

        // Second tick: should NOT fire again (chain consumed)
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "second tick should NOT fire again — chain should be consumed"
        );
    }

    #[test]
    fn bridge_timer_threshold_zero_total_treats_ratio_as_zero() {
        use crate::run::node::resources::NodeTimer;

        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                target: super::super::definition::Target::Bolt,
                multiplier: 1.5,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 10.0,
                total: 0.0, // Edge case: total is zero
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "total == 0.0 should treat ratio as 0.0, which is below 0.5"
        );
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature B: Once node bridge evaluation (behaviors 18-20)
    // =========================================================================

    #[test]
    fn apply_once_nodes_fires_bare_do_and_removes_once_wrapper() {
        use crate::effect::definition::EffectChains;

        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired(Vec<super::super::typed_events::SpawnBoltsFired>);

        fn capture_spawn_bolts(
            trigger: On<super::super::typed_events::SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired>()
            .add_observer(capture_spawn_bolts)
            .add_systems(FixedUpdate, apply_once_nodes);

        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::Once(vec![EffectNode::Do(
                Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                },
            )])]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bare Do inside Once should fire at chip selection time"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be removed from EffectChains after firing"
        );
    }

    #[test]
    fn once_already_consumed_does_not_fire_again() {
        use crate::effect::definition::EffectChains;

        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired2(Vec<super::super::typed_events::SpawnBoltsFired>);

        fn capture_spawn_bolts2(
            trigger: On<super::super::typed_events::SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired2>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired2>()
            .add_observer(capture_spawn_bolts2)
            .add_systems(FixedUpdate, apply_once_nodes);

        // Empty EffectChains — Once was already consumed
        app.world_mut().spawn(EffectChains(vec![]));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired2>();
        assert!(
            captured.0.is_empty(),
            "empty EffectChains should not fire anything"
        );
    }

    // --- Behavior 19a: Bridge Once unwrapping — bridge penetrates Once wrapper ---

    #[test]
    fn bridge_cell_impact_unwraps_once_when_inner_when_matches() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::Once(vec![
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                },
            ])]))
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
            "bridge should unwrap Once, evaluate inner When(Impact(Cell)), and fire Shockwave"
        );

        let chains = app.world().get::<EffectChains>(bolt).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be consumed (removed from EffectChains)"
        );
    }

    // --- Behavior 19b: Bridge Once unwrapping — non-matching inner When preserves Once ---

    #[test]
    fn bridge_cell_impact_preserves_once_when_inner_when_does_not_match() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::Once(vec![
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Wall), // Does NOT match CellImpact
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: super::super::definition::Target::Bolt,
                        multiplier: 2.0,
                    })],
                },
            ])]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert!(
            captured.0.is_empty(),
            "inner When(Impact(Wall)) should NOT match CellImpact trigger"
        );

        let chains = app.world().get::<EffectChains>(bolt).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "Once node should be preserved when inner When does not match"
        );
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature C: EffectChains bridge integration (behaviors 23-28)
    // =========================================================================

    // --- Behavior 23: bridge_cell_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_cell_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })],
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
            "bolt entity EffectChains should be evaluated by bridge_cell_impact"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    // --- Behavior 26: bridge_breaker_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_breaker_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                then: vec![EffectNode::Do(Effect::SpeedBoost {
                    target: super::super::definition::Target::Bolt,
                    multiplier: 1.5,
                })],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt entity EffectChains should be evaluated by bridge_breaker_impact"
        );
        assert!((captured.0[0].multiplier - 1.5).abs() < f32::EPSILON);
    }

    // --- Behavior 27: bridge_wall_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_wall_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Wall),
                then: vec![EffectNode::Do(Effect::SpeedBoost {
                    target: super::super::definition::Target::Bolt,
                    multiplier: 1.3,
                })],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt entity EffectChains should be evaluated by bridge_wall_impact"
        );
        assert!((captured.0[0].multiplier - 1.3).abs() < f32::EPSILON);
    }

    // --- Behavior 25: bridge_bolt_lost evaluates breaker entity EffectChains ---

    #[test]
    fn bridge_bolt_lost_evaluates_breaker_effect_chains_once_consumed() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        #[derive(Resource, Default)]
        struct CapturedSecondWindFired(Vec<super::super::typed_events::SecondWindFired>);

        fn capture_second_wind(
            trigger: On<super::super::typed_events::SecondWindFired>,
            mut captured: ResMut<CapturedSecondWindFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedSecondWindFired>()
            .add_observer(capture_second_wind)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        // Breaker with Once([When(BoltLost, [Do(SecondWind)])])
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::SecondWind { invuln_secs: 3.0 })],
                }])]),
            ))
            .id();

        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "breaker EffectChains should be evaluated by bridge_bolt_lost"
        );
        assert!((captured.0[0].invuln_secs - 3.0).abs() < f32::EPSILON);
    }

    // --- Behavior 28: bridge_bump evaluates breaker entity EffectChains ---

    #[test]
    fn bridge_bump_evaluates_breaker_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: super::super::definition::Target::Bolt,
                        multiplier: 1.2,
                    })],
                }]),
            ))
            .id();

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "breaker EffectChains should be evaluated by bridge_bump"
        );
        assert!((captured.0[0].multiplier - 1.2).abs() < f32::EPSILON);
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature D: Two-Phase Destruction — bridge_cell_death (behaviors 24, 30)
    // =========================================================================

    #[test]
    fn bridge_cell_death_evaluates_ondeath_effect_chains_and_writes_cell_destroyed_at() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::{
            cells::{
                components::{Cell, RequiredToClear},
                messages::{CellDestroyedAt, RequestCellDestroyed},
            },
            effect::definition::EffectChains,
        };

        #[derive(Resource)]
        struct SendRequestCellDestroyed(Option<RequestCellDestroyed>);

        fn send_request(
            msg: Res<SendRequestCellDestroyed>,
            mut writer: MessageWriter<RequestCellDestroyed>,
        ) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedCellDestroyedAt(Vec<CellDestroyedAt>);

        fn capture_cell_destroyed_at(
            mut reader: MessageReader<CellDestroyedAt>,
            mut captured: ResMut<CapturedCellDestroyedAt>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .init_resource::<ActiveEffects>()
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedCellDestroyedAt>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_request, bridge_cell_death, capture_cell_destroyed_at).chain(),
            );

        let cell = app
            .world_mut()
            .spawn((
                Cell,
                RequiredToClear,
                Position2D(Vec2::new(100.0, 200.0)),
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::Death,
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 48.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }]),
            ))
            .id();

        app.insert_resource(SendRequestCellDestroyed(Some(RequestCellDestroyed {
            cell,
        })));

        tick(&mut app);

        // Cell entity should still be alive (bridge doesn't despawn)
        assert!(
            app.world().get_entity(cell).is_ok(),
            "cell entity should still be alive after bridge_cell_death"
        );

        // Death EffectChains should have fired
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "bridge_cell_death should evaluate cell's Death EffectChains"
        );
        assert!((shockwaves.0[0].base_range - 48.0).abs() < f32::EPSILON);

        // CellDestroyedAt message should be written
        let destroyed = app.world().resource::<CapturedCellDestroyedAt>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "bridge_cell_death should write CellDestroyedAt"
        );
        assert_eq!(destroyed.0[0].position, Vec2::new(100.0, 200.0));
        assert!(destroyed.0[0].was_required_to_clear);
    }

    #[test]
    fn bridge_cell_death_writes_cell_destroyed_at_even_without_effect_chains() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::cells::{
            components::{Cell, RequiredToClear},
            messages::{CellDestroyedAt, RequestCellDestroyed},
        };

        #[derive(Resource)]
        struct SendReqCellDest(Option<RequestCellDestroyed>);

        fn send_req(msg: Res<SendReqCellDest>, mut writer: MessageWriter<RequestCellDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedCDA(Vec<CellDestroyedAt>);

        fn capture_cda(
            mut reader: MessageReader<CellDestroyedAt>,
            mut captured: ResMut<CapturedCDA>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .init_resource::<ActiveEffects>()
            .init_resource::<CapturedCDA>()
            .add_systems(
                FixedUpdate,
                (send_req, bridge_cell_death, capture_cda).chain(),
            );

        // Cell without EffectChains
        let cell = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Position2D(Vec2::new(50.0, 75.0))))
            .id();

        app.insert_resource(SendReqCellDest(Some(RequestCellDestroyed { cell })));

        tick(&mut app);

        let captured = app.world().resource::<CapturedCDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "CellDestroyedAt should be written even without EffectChains"
        );
        assert_eq!(captured.0[0].position, Vec2::new(50.0, 75.0));
        assert!(captured.0[0].was_required_to_clear);
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature D: Two-Phase — bridge_bolt_death (behavior 34)
    // =========================================================================

    #[test]
    fn bridge_bolt_death_evaluates_ondeath_and_writes_bolt_destroyed_at() {
        use rantzsoft_spatial2d::components::Position2D;

        use crate::{
            bolt::messages::{BoltDestroyedAt, RequestBoltDestroyed},
            effect::definition::EffectChains,
        };

        #[derive(Resource)]
        struct SendReqBoltDest(Option<RequestBoltDestroyed>);

        fn send_req(msg: Res<SendReqBoltDest>, mut writer: MessageWriter<RequestBoltDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        #[derive(Resource, Default)]
        struct CapturedBDA(Vec<BoltDestroyedAt>);

        fn capture_bda(
            mut reader: MessageReader<BoltDestroyedAt>,
            mut captured: ResMut<CapturedBDA>,
        ) {
            for msg in reader.read() {
                captured.0.push(msg.clone());
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .add_message::<BoltDestroyedAt>()
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedBDA>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_req, bridge_bolt_death, capture_bda).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(50.0, -100.0)),
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::Death,
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 32.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }]),
            ))
            .id();

        app.insert_resource(SendReqBoltDest(Some(RequestBoltDestroyed { bolt })));

        tick(&mut app);

        // Bolt entity should still be alive
        assert!(
            app.world().get_entity(bolt).is_ok(),
            "bolt entity should still be alive after bridge_bolt_death"
        );

        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "bridge_bolt_death should evaluate bolt's Death EffectChains"
        );

        let captured = app.world().resource::<CapturedBDA>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_bolt_death should write BoltDestroyedAt"
        );
        assert_eq!(captured.0[0].position, Vec2::new(50.0, -100.0));
    }

    // =========================================================================
    // C7 Wave 2a Sub-Feature D: Two-Phase — Breaker stub types (behavior 36)
    // =========================================================================

    #[test]
    fn request_breaker_destroyed_and_breaker_destroyed_at_types_exist() {
        use crate::breaker::messages::{BreakerDestroyedAt, RequestBreakerDestroyed};

        let req = RequestBreakerDestroyed {
            breaker: Entity::PLACEHOLDER,
        };
        let dest = BreakerDestroyedAt {
            position: Vec2::new(0.0, -300.0),
        };
        // Types construct without error — Message derive is valid
        assert!(format!("{req:?}").contains("RequestBreakerDestroyed"));
        assert!(format!("{dest:?}").contains("BreakerDestroyedAt"));
    }

    // =========================================================================
    // BLOCKING: Behavior 32 — cleanup_destroyed_cells despawns entities
    // =========================================================================

    #[test]
    fn cleanup_destroyed_cells_despawns_cell_entity() {
        use crate::cells::messages::RequestCellDestroyed;

        #[derive(Resource)]
        struct SendReq(Option<RequestCellDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestCellDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_systems(FixedUpdate, (send_req, cleanup_destroyed_cells).chain());

        let cell = app.world_mut().spawn_empty().id();
        app.insert_resource(SendReq(Some(RequestCellDestroyed { cell })));

        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned after cleanup_destroyed_cells"
        );
    }

    #[test]
    fn cleanup_destroyed_cells_no_panic_if_entity_already_despawned() {
        use crate::cells::messages::RequestCellDestroyed;

        #[derive(Resource)]
        struct SendReqTwice(Vec<RequestCellDestroyed>);

        fn send_req_twice(
            mut msgs: ResMut<SendReqTwice>,
            mut writer: MessageWriter<RequestCellDestroyed>,
        ) {
            for m in msgs.0.drain(..) {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .add_systems(
                FixedUpdate,
                (send_req_twice, cleanup_destroyed_cells).chain(),
            );

        let cell = app.world_mut().spawn_empty().id();
        // Send the same cell entity in two separate messages — second should not panic
        app.insert_resource(SendReqTwice(vec![
            RequestCellDestroyed { cell },
            RequestCellDestroyed { cell },
        ]));

        // Should not panic even if the entity is despawned by the first message
        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned"
        );
    }

    // =========================================================================
    // BLOCKING: Behavior 35 — cleanup_destroyed_bolts despawns entities
    // =========================================================================

    #[test]
    fn cleanup_destroyed_bolts_despawns_bolt_entity() {
        use crate::bolt::messages::RequestBoltDestroyed;

        #[derive(Resource)]
        struct SendReq(Option<RequestBoltDestroyed>);

        fn send_req(msg: Res<SendReq>, mut writer: MessageWriter<RequestBoltDestroyed>) {
            if let Some(m) = msg.0.clone() {
                writer.write(m);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .add_systems(FixedUpdate, (send_req, cleanup_destroyed_bolts).chain());

        let bolt = app.world_mut().spawn_empty().id();
        app.insert_resource(SendReq(Some(RequestBoltDestroyed { bolt })));

        tick(&mut app);

        assert!(
            app.world().get_entity(bolt).is_err(),
            "bolt entity should be despawned after cleanup_destroyed_bolts"
        );
    }

    // =========================================================================
    // IMPORTANT: Behavior 19/25 — Once consumed prevents second firing
    // =========================================================================

    #[test]
    fn bridge_bolt_lost_once_consumed_second_bolt_lost_does_not_fire() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        #[derive(Resource, Default)]
        struct CapturedSecondWindFired2(Vec<super::super::typed_events::SecondWindFired>);

        fn capture_second_wind2(
            trigger: On<super::super::typed_events::SecondWindFired>,
            mut captured: ResMut<CapturedSecondWindFired2>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedSecondWindFired2>()
            .add_observer(capture_second_wind2)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        // Breaker with Once([When(BoltLost, [Do(SecondWind { invuln_secs: 3.0 })])])
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::SecondWind { invuln_secs: 3.0 })],
                }])]),
            ))
            .id();

        // First bolt lost — should fire SecondWind
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired2>();
        assert_eq!(
            captured.0.len(),
            1,
            "first BoltLost should fire SecondWind from Once wrapper"
        );

        // Second bolt lost — should NOT fire again (Once consumed)
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired2>();
        assert_eq!(
            captured.0.len(),
            1,
            "second BoltLost should NOT fire SecondWind — Once was consumed"
        );
    }

    // =========================================================================
    // IMPORTANT: Behavior 23 edge — ActiveEffects + EffectChains both fire
    // =========================================================================

    #[test]
    fn bridge_cell_impact_fires_both_active_effects_and_entity_effect_chains() {
        use crate::effect::definition::EffectChains;

        // ActiveEffects has a Shockwave chain on Impact(Cell)
        let active_chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Cell),
            Effect::test_shockwave(64.0),
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(vec![(
                Some("chip1".to_string()),
                active_chain,
            )]))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with entity-level EffectChains containing SpeedBoost on cell impact
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::SpeedBoost {
                    target: super::super::definition::Target::Bolt,
                    multiplier: 1.5,
                })],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        // ActiveEffects should fire Shockwave
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "ActiveEffects Shockwave chain should fire on cell impact"
        );
        assert!((shockwaves.0[0].base_range - 64.0).abs() < f32::EPSILON);

        // Entity EffectChains should fire SpeedBoost
        let speed_boosts = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            speed_boosts.0.len(),
            1,
            "entity EffectChains SpeedBoost should also fire on cell impact"
        );
        assert!((speed_boosts.0[0].multiplier - 1.5).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Nested When children inside UntilTimers/UntilTriggers — bridge evaluation
    // =========================================================================

    // --- bridge_cell_impact evaluates UntilTimers children with When nodes ---

    #[test]
    fn bridge_cell_impact_evaluates_until_timer_children() {
        use crate::effect::effect_nodes::until::{UntilTimerEntry, UntilTimers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with UntilTimers containing a When(Impact(Cell)) child
        let bolt = app
            .world_mut()
            .spawn(UntilTimers(vec![UntilTimerEntry {
                remaining: 3.0, // Still active (not expired)
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
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
            "bridge_cell_impact should evaluate When children inside UntilTimers entries"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "fired shockwave should have base_range 64.0, got {}",
            captured.0[0].base_range
        );
    }

    // --- bridge_cell_impact does not fire expired Until children ---

    #[test]
    fn bridge_cell_impact_does_not_fire_expired_until_children() {
        // Bolt entity with NO UntilTimers component (all timers expired).
        // BoltHitCell message written.
        // No ShockwaveFired event expected.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt without UntilTimers — all timers have already expired
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "no ShockwaveFired should be emitted when Until has expired (no UntilTimers component)"
        );
    }

    // --- bridge_cell_impact evaluates UntilTriggers children with When nodes ---

    #[test]
    fn bridge_cell_impact_evaluates_until_trigger_children() {
        use crate::effect::effect_nodes::until::{UntilTriggerEntry, UntilTriggers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with UntilTriggers entry (trigger: Impact(Breaker)) containing
        // When(Impact(Cell), [Do(Shockwave)]). The Until itself has NOT expired
        // (trigger is Impact(Breaker), and we're hitting a Cell), so nested
        // children should still be active and evaluated.
        let bolt = app
            .world_mut()
            .spawn(UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 48.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
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
            "bridge_cell_impact should evaluate When children inside UntilTriggers entries"
        );
        assert!(
            (captured.0[0].base_range - 48.0).abs() < f32::EPSILON,
            "fired shockwave should have base_range 48.0, got {}",
            captured.0[0].base_range
        );
    }

    // --- bridge_wall_impact evaluates UntilTimers children with When nodes ---

    #[test]
    fn bridge_wall_impact_evaluates_until_timer_children() {
        use crate::effect::effect_nodes::until::{UntilTimerEntry, UntilTimers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );

        // Bolt with UntilTimers containing a When(Impact(Wall)) child
        let bolt = app
            .world_mut()
            .spawn(UntilTimers(vec![UntilTimerEntry {
                remaining: 3.0, // Still active
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Wall),
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: super::super::definition::Target::Bolt,
                        multiplier: 1.3,
                    })],
                }],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_wall_impact should evaluate When children inside UntilTimers entries"
        );
        assert!(
            (captured.0[0].multiplier - 1.3).abs() < f32::EPSILON,
            "fired SpeedBoost should have multiplier 1.3, got {}",
            captured.0[0].multiplier
        );
    }

    // =========================================================================
    // NoBump bridge tests (behaviors 1-4)
    // =========================================================================

    /// Test app for `bridge_no_bump` — registers both `BoltHitBreaker` and
    /// `BumpPerformed` messages so the bridge can detect `NoBump` conditions.
    fn no_bump_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitBreaker(None))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, send_bump, bridge_no_bump).chain(),
            );
        app
    }

    #[test]
    fn bridge_no_bump_fires_when_bolt_hits_breaker_without_bump() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();

        // BoltHitBreaker present, NO BumpPerformed
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "NoBump chain should fire when BoltHitBreaker arrives without BumpPerformed"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_no_bump_does_not_fire_when_bump_performed_also_arrives() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();

        // Both BoltHitBreaker AND BumpPerformed present
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "NoBump chain should NOT fire when BumpPerformed is also present"
        );
    }

    #[test]
    fn bridge_no_bump_does_not_fire_when_no_bolt_hit_breaker() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);

        // Neither BoltHitBreaker nor BumpPerformed
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "NoBump chain should NOT fire when no BoltHitBreaker arrives"
        );
    }

    #[test]
    fn bridge_no_bump_evaluates_breaker_entity_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitBreaker(None))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, send_bump, bridge_no_bump).chain(),
            );

        // Breaker entity with NoBump EffectChain
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::NoBump,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }]),
            ))
            .id();

        let bolt = app.world_mut().spawn_empty().id();

        // BoltHitBreaker present, NO BumpPerformed
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_no_bump should evaluate breaker entity EffectChains with Trigger::NoBump"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Bumped split in bridge_bump tests (behaviors 5-8)
    // =========================================================================

    #[test]
    fn bridge_bump_fires_perfect_bumped_on_bolt_entity_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Bolt entity with PerfectBumped EffectChain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]))
            .id();

        // Spawn breaker entity so breaker_query has something
        app.world_mut().spawn((Breaker, EffectChains::default()));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "PerfectBumped chain on bolt entity should fire on Perfect bump"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_bump_fires_generic_bumped_on_bolt_entity_for_any_grade() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Bolt entity with generic Bumped EffectChain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]))
            .id();

        // Spawn breaker entity so breaker_query has something
        app.world_mut().spawn((Breaker, EffectChains::default()));

        // Send Early bump — Bumped should match any non-whiff grade
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "Bumped chain on bolt entity should fire for Early grade (matches any non-whiff)"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_bump_perfect_bumped_does_not_fire_on_breaker_entity() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Breaker entity with PerfectBumped EffectChain — should NOT fire
        // (PerfectBumped is bolt-perspective only)
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        ));

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "PerfectBumped on breaker entity should NOT fire — Bumped is bolt-perspective only"
        );
    }

    #[test]
    fn bridge_bump_perfect_bump_still_fires_on_breaker_entity() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Breaker entity with PerfectBump (breaker-perspective) EffectChain
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        ));

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "PerfectBump (breaker-perspective) on breaker entity should still fire"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }
}
