//! Per-trigger bridge systems — translate messages into effect events.
//!
//! Each bridge reads one message type, evaluates all active chains,
//! and either fires `EffectFired` or arms the bolt with `ArmedEffects`.

use bevy::prelude::*;

use super::{
    active::ActiveEffects,
    armed::ArmedEffects,
    evaluate::{EvalResult, TriggerKind, evaluate},
    events::EffectFired,
};
use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall, BoltLost},
    breaker::messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    cells::messages::CellDestroyed,
    chips::definition::TriggerChain,
};

/// Bridge for `BoltLost` — evaluates chains when a bolt is lost.
///
/// Global trigger: evaluates active chains once per frame (not per message)
/// and evaluates armed triggers on ALL bolt entities.
pub(crate) fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = TriggerKind::BoltLost;
    evaluate_active_chains(&active, trigger_kind, None, &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Bridge for `BumpPerformed` — evaluates chains on bump.
///
/// For each bump message, evaluates two trigger kinds:
/// 1. Grade-specific: Perfect→`PerfectBump`, Early→`EarlyBump`, Late→`LateBump`
/// 2. `BumpSuccess`: all non-whiff bumps evaluate `OnBump` chains.
pub(crate) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut commands: Commands,
) {
    for performed in reader.read() {
        let bolt_entity = performed.bolt;
        let grade_trigger = match performed.grade {
            BumpGrade::Perfect => TriggerKind::PerfectBump,
            BumpGrade::Early => TriggerKind::EarlyBump,
            BumpGrade::Late => TriggerKind::LateBump,
        };

        for (chip_name, chain) in &active.0 {
            // Grade-specific evaluation
            for result in evaluate(grade_trigger, chain) {
                match result {
                    EvalResult::Fire(leaf) => {
                        commands.trigger(EffectFired {
                            effect: leaf,
                            bolt: Some(bolt_entity),
                            source_chip: chip_name.clone(),
                        });
                    }
                    EvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    EvalResult::NoMatch => {}
                }
            }
            // BumpSuccess evaluation (all grades)
            for result in evaluate(TriggerKind::BumpSuccess, chain) {
                match result {
                    EvalResult::Fire(leaf) => {
                        commands.trigger(EffectFired {
                            effect: leaf,
                            bolt: Some(bolt_entity),
                            source_chip: chip_name.clone(),
                        });
                    }
                    EvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    EvalResult::NoMatch => {}
                }
            }
        }

        // Evaluate armed triggers on the specific bolt
        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, grade_trigger);
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            TriggerKind::BumpSuccess,
        );
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
    let trigger_kind = TriggerKind::BumpWhiff;
    evaluate_active_chains(&active, trigger_kind, None, &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Bridge for `BoltHitCell` — evaluates chains and armed triggers on cell impact.
pub(crate) fn bridge_cell_impact(
    mut reader: MessageReader<BoltHitCell>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        for (chip_name, chain) in &active.0 {
            for result in evaluate(TriggerKind::CellImpact, chain) {
                match result {
                    EvalResult::Fire(leaf) => {
                        commands.trigger(EffectFired {
                            effect: leaf,
                            bolt: Some(bolt_entity),
                            source_chip: chip_name.clone(),
                        });
                    }
                    EvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    EvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            TriggerKind::CellImpact,
        );
    }
}

/// Bridge for `BoltHitBreaker` — evaluates chains and armed triggers on
/// breaker impact.
pub(crate) fn bridge_breaker_impact(
    mut reader: MessageReader<BoltHitBreaker>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        for (chip_name, chain) in &active.0 {
            for result in evaluate(TriggerKind::BreakerImpact, chain) {
                match result {
                    EvalResult::Fire(leaf) => {
                        commands.trigger(EffectFired {
                            effect: leaf,
                            bolt: Some(bolt_entity),
                            source_chip: chip_name.clone(),
                        });
                    }
                    EvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    EvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            TriggerKind::BreakerImpact,
        );
    }
}

/// Bridge for `BoltHitWall` — evaluates chains and armed triggers on
/// wall impact.
pub(crate) fn bridge_wall_impact(
    mut reader: MessageReader<BoltHitWall>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        for (chip_name, chain) in &active.0 {
            for result in evaluate(TriggerKind::WallImpact, chain) {
                match result {
                    EvalResult::Fire(leaf) => {
                        commands.trigger(EffectFired {
                            effect: leaf,
                            bolt: Some(bolt_entity),
                            source_chip: chip_name.clone(),
                        });
                    }
                    EvalResult::Arm(remaining) => {
                        arm_bolt(
                            &mut armed_query,
                            &mut commands,
                            bolt_entity,
                            chip_name.clone(),
                            remaining,
                        );
                    }
                    EvalResult::NoMatch => {}
                }
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            TriggerKind::WallImpact,
        );
    }
}

/// Bridge for `CellDestroyed` — evaluates chains when a cell is destroyed.
///
/// Global trigger: evaluates active chains once per frame and evaluates
/// armed triggers on ALL bolt entities.
pub(crate) fn bridge_cell_destroyed(
    mut reader: MessageReader<CellDestroyed>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = TriggerKind::CellDestroyed;
    evaluate_active_chains(&active, trigger_kind, None, &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Evaluates all active chains against a trigger kind.
///
/// `Arm` results are intentionally discarded for global triggers — only `Fire`
/// results are actioned. Arming requires a specific bolt entity, which global
/// triggers (cell destroyed, bolt lost, bump whiff) don't provide.
fn evaluate_active_chains(
    active: &ActiveEffects,
    trigger_kind: TriggerKind,
    bolt: Option<Entity>,
    commands: &mut Commands,
) {
    for (chip_name, chain) in &active.0 {
        for result in evaluate(trigger_kind, chain) {
            match result {
                EvalResult::Fire(leaf) => {
                    commands.trigger(EffectFired {
                        effect: leaf,
                        bolt,
                        source_chip: chip_name.clone(),
                    });
                }
                EvalResult::Arm(_) | EvalResult::NoMatch => {}
            }
        }
    }
}

/// Evaluates armed triggers on all bolt entities that have `ArmedEffects`.
fn evaluate_armed_all(
    mut armed_query: Query<(Entity, &mut ArmedEffects)>,
    trigger_kind: TriggerKind,
    commands: &mut Commands,
) {
    for (bolt_entity, mut armed) in &mut armed_query {
        resolve_armed(&mut armed, trigger_kind, Some(bolt_entity), commands);
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
    remaining: TriggerChain,
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
    trigger_kind: TriggerKind,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        resolve_armed(&mut armed, trigger_kind, Some(bolt_entity), commands);
    }
}

/// Resolves armed trigger chains: fires leaves, re-arms non-leaves, retains non-matches.
fn resolve_armed(
    armed: &mut ArmedEffects,
    trigger_kind: TriggerKind,
    bolt: Option<Entity>,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for (chip_name, chain) in armed.0.drain(..) {
        let results = evaluate(trigger_kind, &chain);
        let mut matched = false;
        for result in results {
            match result {
                EvalResult::Fire(leaf) => {
                    matched = true;
                    commands.trigger(EffectFired {
                        effect: leaf,
                        bolt,
                        source_chip: chip_name.clone(),
                    });
                }
                EvalResult::Arm(next) => {
                    matched = true;
                    new_armed.push((chip_name.clone(), next));
                }
                EvalResult::NoMatch => {}
            }
        }
        if !matched {
            new_armed.push((chip_name, chain));
        }
    }
    armed.0 = new_armed;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::messages::BumpGrade,
        chips::definition::{ImpactTarget, TriggerChain},
        effect::events::EffectFired,
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedEffects(Vec<(TriggerChain, Option<Entity>)>);

    fn capture_effects(trigger: On<EffectFired>, mut captured: ResMut<CapturedEffects>) {
        captured
            .0
            .push((trigger.event().effect.clone(), trigger.event().bolt));
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
    struct SendCellDestroyed(Option<CellDestroyed>);

    fn send_cell_destroyed(msg: Res<SendCellDestroyed>, mut writer: MessageWriter<CellDestroyed>) {
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

    /// Wraps a list of `TriggerChain`s as `(None, chain)` tuples for `ActiveEffects`.
    fn wrap_chains(chains: Vec<TriggerChain>) -> Vec<(Option<String>, TriggerChain)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    fn bolt_lost_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
        app
    }

    fn bump_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
        app
    }

    fn bump_whiff_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBumpWhiffFlag(false))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());
        app
    }

    fn cell_impact_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );
        app
    }

    fn breaker_impact_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );
        app
    }

    fn wall_impact_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );
        app
    }

    fn cell_destroyed_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_cell_destroyed, bridge_cell_destroyed).chain(),
            );
        app
    }

    // --- Bolt lost bridge tests ---

    #[test]
    fn bolt_lost_fires_active_chains() {
        let chain = TriggerChain::OnBoltLost(vec![TriggerChain::test_lose_life()]);
        let mut app = bolt_lost_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::LoseLife);
        assert_eq!(captured.0[0].1, None);
    }

    #[test]
    fn bolt_lost_no_message_no_fire() {
        let chain = TriggerChain::OnBoltLost(vec![TriggerChain::test_lose_life()]);
        let mut app = bolt_lost_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty());
    }

    // --- Bump bridge tests ---

    #[test]
    fn perfect_bump_fires_on_perfect_bump_chain() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]);
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(64.0));
        assert_eq!(captured.0[0].1, Some(bolt));
    }

    #[test]
    fn perfect_bump_fires_both_on_perfect_bump_and_on_bump_success() {
        let chains = vec![
            TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]),
            TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            2,
            "perfect bump should fire BOTH OnPerfectBump and OnBump chains"
        );
        let effects: Vec<&TriggerChain> = captured.0.iter().map(|(e, _)| e).collect();
        assert!(effects.contains(&&TriggerChain::test_shockwave(64.0)));
        assert!(effects.contains(&&TriggerChain::test_shield(3.0)));
    }

    #[test]
    fn early_bump_fires_on_early_bump_and_on_bump_success_but_not_on_perfect_bump() {
        let chains = vec![
            TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]),
            TriggerChain::OnEarlyBump(vec![TriggerChain::test_lose_life()]),
            TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            2,
            "early bump should fire OnEarlyBump and OnBump, not OnPerfectBump"
        );
        let effects: Vec<&TriggerChain> = captured.0.iter().map(|(e, _)| e).collect();
        assert!(effects.contains(&&TriggerChain::LoseLife));
        assert!(effects.contains(&&TriggerChain::test_shield(3.0)));
        assert!(!effects.contains(&&TriggerChain::test_shockwave(64.0)));
    }

    #[test]
    fn late_bump_fires_on_late_bump_and_on_bump_success() {
        let chains = vec![
            TriggerChain::OnLateBump(vec![TriggerChain::test_time_penalty(3.0)]),
            TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Late,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 2);
        let effects: Vec<&TriggerChain> = captured.0.iter().map(|(e, _)| e).collect();
        assert!(effects.contains(&&TriggerChain::test_time_penalty(3.0)));
        assert!(effects.contains(&&TriggerChain::test_shield(3.0)));
    }

    #[test]
    fn perfect_bump_with_non_leaf_arms_bolt() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::test_shockwave(64.0)],
        )]);
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty(), "non-leaf inner should arm, not fire");

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(armed.0.len(), 1);
        assert_eq!(
            armed.0[0].1,
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)])
        );
    }

    // --- BumpWhiff bridge tests ---

    #[test]
    fn bump_whiff_fires_on_bump_whiff_chain() {
        let chain = TriggerChain::OnBumpWhiff(vec![TriggerChain::test_lose_life()]);
        let mut app = bump_whiff_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::LoseLife);
        assert_eq!(captured.0[0].1, None);
    }

    #[test]
    fn bump_whiff_no_message_no_fire() {
        let chain = TriggerChain::OnBumpWhiff(vec![TriggerChain::test_lose_life()]);
        let mut app = bump_whiff_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty());
    }

    // --- Cell impact bridge tests ---

    #[test]
    fn cell_impact_fires_active_chain() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let mut app = cell_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(64.0));
    }

    #[test]
    fn cell_impact_fires_armed_trigger() {
        let mut app = cell_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                TriggerChain::OnImpact(
                    ImpactTarget::Cell,
                    vec![TriggerChain::test_shockwave(64.0)],
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(64.0));

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert!(armed.0.is_empty());
    }

    #[test]
    fn cell_impact_no_message_no_fire() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        let mut app = cell_impact_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty());
    }

    // --- Breaker impact bridge tests ---

    #[test]
    fn breaker_impact_fires_active_chain() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Breaker, vec![TriggerChain::test_shield(5.0)]);
        let mut app = breaker_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shield(5.0));
    }

    #[test]
    fn breaker_impact_fires_armed_trigger() {
        let mut app = breaker_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                TriggerChain::OnImpact(
                    ImpactTarget::Breaker,
                    vec![TriggerChain::test_multi_bolt(2)],
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_multi_bolt(2));
    }

    // --- Wall impact bridge tests ---

    #[test]
    fn wall_impact_fires_active_chain() {
        let chain =
            TriggerChain::OnImpact(ImpactTarget::Wall, vec![TriggerChain::test_shockwave(32.0)]);
        let mut app = wall_impact_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(32.0));
    }

    #[test]
    fn wall_impact_fires_armed_trigger() {
        let mut app = wall_impact_test_app(vec![]);
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                TriggerChain::OnImpact(ImpactTarget::Wall, vec![TriggerChain::test_shield(5.0)]),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shield(5.0));
    }

    // --- Cell destroyed bridge tests ---

    #[test]
    fn cell_destroyed_fires_active_chain() {
        let chain = TriggerChain::OnCellDestroyed(vec![TriggerChain::test_shockwave(32.0)]);
        let mut app = cell_destroyed_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(CellDestroyed {
            was_required_to_clear: true,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(32.0));
        assert_eq!(captured.0[0].1, None);
    }

    #[test]
    fn cell_destroyed_no_message_no_fire() {
        let chain = TriggerChain::OnCellDestroyed(vec![TriggerChain::test_shockwave(32.0)]);
        let mut app = cell_destroyed_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty());
    }

    // --- Full two-step chain tests ---

    #[test]
    fn full_two_step_chain_bump_arms_then_impact_fires() {
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::test_shockwave(64.0)],
        )]);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
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
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
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

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].0, TriggerChain::test_shockwave(64.0));
    }

    #[test]
    fn full_three_step_chain_bump_arms_impact_rearms_cell_destroyed_fires() {
        // 3-deep: OnPerfectBump(OnImpact(Cell, OnCellDestroyed(Shockwave(64.0))))
        let chain = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::OnCellDestroyed(vec![
                TriggerChain::test_shockwave(64.0),
            ])],
        )]);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .add_message::<CellDestroyed>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_bump,
                    send_bolt_hit_cell,
                    bridge_cell_impact,
                    send_cell_destroyed,
                    bridge_cell_destroyed,
                )
                    .chain(),
            );

        let bolt = app.world_mut().spawn_empty().id();

        // Step 1: Perfect bump — arms bolt with OnImpact(Cell, OnCellDestroyed(Shockwave))
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
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
        assert_eq!(
            armed.0[0].1,
            TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::OnCellDestroyed(vec![
                    TriggerChain::test_shockwave(64.0),
                ])],
            ),
            "step 1: armed trigger should be the 2-deep remaining chain"
        );

        // Clear bump, prepare cell impact
        app.world_mut().resource_mut::<SendBump>().0 = None;
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        // Step 2: Cell impact — re-arms bolt with OnCellDestroyed(Shockwave)
        let captured = app.world().resource::<CapturedEffects>();
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
        assert_eq!(
            armed.0[0].1,
            TriggerChain::OnCellDestroyed(vec![TriggerChain::test_shockwave(64.0)]),
            "step 2: armed trigger should be OnCellDestroyed(Shockwave)"
        );

        // Clear cell hit, prepare cell destroyed
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = None;
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(CellDestroyed {
            was_required_to_clear: true,
        });
        tick(&mut app);

        // Step 3: Cell destroyed — fires the shockwave
        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "step 3: shockwave should fire after cell destroyed"
        );
        assert_eq!(
            captured.0[0].0,
            TriggerChain::test_shockwave(64.0),
            "step 3: fired effect should be the shockwave leaf"
        );
    }

    // --- Integration: bridge + effect observer ---

    #[test]
    fn bridge_bolt_lost_plus_life_lost_observer_decrements_lives() {
        use crate::{
            effect::effects::life_lost::{LivesCount, handle_life_lost},
            run::messages::RunLost,
        };

        let chain = TriggerChain::OnBoltLost(vec![TriggerChain::LoseLife]);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .add_message::<RunLost>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBoltLostFlag(false))
            .add_observer(capture_effects)
            .add_observer(handle_life_lost)
            .init_resource::<CapturedEffects>()
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
    // These tests verify the EffectNode shapes that bridges will process
    // after migration. They exercise evaluate_node which fails with todo!().
    // =========================================================================

    #[test]
    fn evaluate_node_fires_effect_for_bolt_lost_bridge() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // Bridges will call evaluate_node(BoltLost, &node) and get Fire(Effect::Shockwave)
        let node = EffectNode::Trigger(
            Trigger::OnBoltLost,
            vec![EffectNode::Leaf(Effect::Shockwave {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        );
        let result = evaluate_node(TriggerKind::BoltLost, &node);
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
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let inner_node = EffectNode::Trigger(
            Trigger::OnImpact(super::super::definition::ImpactTarget::Cell),
            vec![EffectNode::Leaf(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        );
        let node = EffectNode::Trigger(Trigger::OnPerfectBump, vec![inner_node.clone()]);
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Arm(inner_node)],
            "bridge_bump should get Arm(inner_node) for non-leaf resolution"
        );
    }

    #[test]
    fn evaluate_node_no_match_for_wrong_trigger_in_bridge() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
        );
        // BoltLost should not match OnPerfectBump
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::NoMatch],
            "BoltLost trigger should not match OnPerfectBump node"
        );
    }
}
