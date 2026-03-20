//! Per-trigger bridge systems for the overclock evaluation engine.
//!
//! Each bridge reads one message type, evaluates all active overclock chains,
//! and either fires `OverclockEffectFired` or arms the bolt with `ArmedTriggers`.

use bevy::prelude::*;

use super::{
    active::ActiveOverclocks,
    armed::ArmedTriggers,
    evaluate::{EvalResult, OverclockTriggerKind, evaluate},
    events::OverclockEffectFired,
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    cells::messages::CellDestroyed,
    chips::definition::TriggerChain,
    physics::messages::{BoltHitCell, BoltLost},
};

/// Bridge for `BumpPerformed` — evaluates overclock chains on perfect bump.
///
/// Only processes `BumpGrade::Perfect` messages. Evaluates all active chains
/// and arms the specific bolt entity on `Arm`, or fires on `Fire`.
pub(crate) fn bridge_overclock_bump(
    mut reader: MessageReader<BumpPerformed>,
    active: Res<ActiveOverclocks>,
    mut armed_query: Query<&mut ArmedTriggers>,
    mut commands: Commands,
) {
    for performed in reader.read() {
        if performed.grade != BumpGrade::Perfect {
            continue;
        }
        let bolt_entity = performed.bolt;
        for chain in &active.0 {
            match evaluate(OverclockTriggerKind::PerfectBump, chain) {
                EvalResult::Fire(leaf) => {
                    commands.trigger(OverclockEffectFired {
                        effect: leaf,
                        bolt: bolt_entity,
                    });
                }
                EvalResult::Arm(remaining) => {
                    arm_bolt(&mut armed_query, &mut commands, bolt_entity, remaining);
                }
                EvalResult::NoMatch => {}
            }
        }
    }
}

/// Bridge for `BoltHitCell` — evaluates overclock chains and armed triggers on impact.
///
/// For each impact message, evaluates active chains against `Impact` trigger kind
/// and also evaluates armed triggers on the specific bolt that hit the cell.
pub(crate) fn bridge_overclock_impact(
    mut reader: MessageReader<BoltHitCell>,
    active: Res<ActiveOverclocks>,
    mut armed_query: Query<&mut ArmedTriggers>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        for chain in &active.0 {
            match evaluate(OverclockTriggerKind::Impact, chain) {
                EvalResult::Fire(leaf) => {
                    commands.trigger(OverclockEffectFired {
                        effect: leaf,
                        bolt: bolt_entity,
                    });
                }
                EvalResult::Arm(remaining) => {
                    arm_bolt(&mut armed_query, &mut commands, bolt_entity, remaining);
                }
                EvalResult::NoMatch => {}
            }
        }
        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            OverclockTriggerKind::Impact,
        );
    }
}

/// Bridge for `CellDestroyed` — evaluates overclock chains when a cell is destroyed.
///
/// Global trigger — evaluates active chains once per message and evaluates
/// armed triggers on ALL bolt entities.
pub(crate) fn bridge_overclock_cell_destroyed(
    mut reader: MessageReader<CellDestroyed>,
    active: Res<ActiveOverclocks>,
    armed_query: Query<(Entity, &mut ArmedTriggers)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = OverclockTriggerKind::CellDestroyed;
    evaluate_active_chains(&active, trigger_kind, Entity::PLACEHOLDER, &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Bridge for `BoltLost` — evaluates overclock chains when a bolt is lost.
///
/// Global trigger — evaluates active chains once per message and evaluates
/// armed triggers on ALL bolt entities.
pub(crate) fn bridge_overclock_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    active: Res<ActiveOverclocks>,
    armed_query: Query<(Entity, &mut ArmedTriggers)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = OverclockTriggerKind::BoltLost;
    evaluate_active_chains(&active, trigger_kind, Entity::PLACEHOLDER, &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Evaluates all active overclock chains against a trigger kind.
///
/// `Arm` results are intentionally discarded for global triggers — only `Fire`
/// results are actioned. Arming requires a specific bolt entity, which global
/// triggers (cell destroyed, bolt lost) don't provide.
fn evaluate_active_chains(
    active: &ActiveOverclocks,
    trigger_kind: OverclockTriggerKind,
    bolt: Entity,
    commands: &mut Commands,
) {
    for chain in &active.0 {
        match evaluate(trigger_kind, chain) {
            EvalResult::Fire(leaf) => {
                commands.trigger(OverclockEffectFired { effect: leaf, bolt });
            }
            EvalResult::Arm(_) | EvalResult::NoMatch => {}
        }
    }
}

/// Evaluates armed triggers on all bolt entities that have `ArmedTriggers`.
fn evaluate_armed_all(
    mut armed_query: Query<(Entity, &mut ArmedTriggers)>,
    trigger_kind: OverclockTriggerKind,
    commands: &mut Commands,
) {
    for (bolt_entity, mut armed) in &mut armed_query {
        resolve_armed(&mut armed, trigger_kind, bolt_entity, commands);
    }
}

/// Arms a bolt entity with a remaining trigger chain.
///
/// If the bolt already has `ArmedTriggers`, pushes to the existing vec.
/// Otherwise, inserts a new `ArmedTriggers` component.
fn arm_bolt(
    armed_query: &mut Query<&mut ArmedTriggers>,
    commands: &mut Commands,
    bolt_entity: Entity,
    remaining: TriggerChain,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        armed.0.push(remaining);
    } else {
        commands
            .entity(bolt_entity)
            .insert(ArmedTriggers(vec![remaining]));
    }
}

/// Evaluates armed triggers on a specific bolt entity.
///
/// Fires matching leaf chains, re-arms non-leaf matches, and retains
/// non-matching chains.
fn evaluate_armed(
    armed_query: &mut Query<&mut ArmedTriggers>,
    commands: &mut Commands,
    bolt_entity: Entity,
    trigger_kind: OverclockTriggerKind,
) {
    if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
        resolve_armed(&mut armed, trigger_kind, bolt_entity, commands);
    }
}

/// Resolves armed trigger chains: fires leaves, re-arms non-leaves, retains non-matches.
fn resolve_armed(
    armed: &mut ArmedTriggers,
    trigger_kind: OverclockTriggerKind,
    bolt: Entity,
    commands: &mut Commands,
) {
    let mut new_armed = Vec::new();
    for chain in armed.0.drain(..) {
        match evaluate(trigger_kind, &chain) {
            EvalResult::Fire(leaf) => commands.trigger(OverclockEffectFired { effect: leaf, bolt }),
            EvalResult::Arm(next) => new_armed.push(next),
            EvalResult::NoMatch => new_armed.push(chain),
        }
    }
    armed.0 = new_armed;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::behaviors::events::OverclockEffectFired, breaker::messages::BumpGrade,
        chips::definition::TriggerChain,
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedEffects(Vec<(TriggerChain, Entity)>);

    fn capture_effects(trigger: On<OverclockEffectFired>, mut captured: ResMut<CapturedEffects>) {
        captured
            .0
            .push((trigger.event().effect.clone(), trigger.event().bolt));
    }

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
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
    struct SendCellDestroyed(Option<CellDestroyed>);

    fn send_cell_destroyed(msg: Res<SendCellDestroyed>, mut writer: MessageWriter<CellDestroyed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltLostFlag(bool);

    fn send_bolt_lost(flag: Res<SendBoltLostFlag>, mut writer: MessageWriter<BoltLost>) {
        if flag.0 {
            writer.write(BoltLost);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Builds a test app with the bump bridge wired.
    fn bump_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveOverclocks(active_chains))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(FixedUpdate, (send_bump, bridge_overclock_bump).chain());
        app
    }

    /// Builds a test app with the impact bridge wired.
    fn impact_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveOverclocks(active_chains))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_overclock_impact).chain(),
            );
        app
    }

    /// Builds a test app with the cell destroyed bridge wired.
    fn cell_destroyed_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .insert_resource(ActiveOverclocks(active_chains))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_cell_destroyed, bridge_overclock_cell_destroyed).chain(),
            );
        app
    }

    /// Builds a test app with the bolt lost bridge wired.
    fn bolt_lost_test_app(active_chains: Vec<TriggerChain>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveOverclocks(active_chains))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (send_bolt_lost, bridge_overclock_bolt_lost).chain(),
            );
        app
    }

    // --- Bump bridge tests ---

    #[test]
    fn perfect_bump_with_active_leaf_fires_effect() {
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let mut app = bump_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.5,
            bolt: Entity::PLACEHOLDER,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "perfect bump with active OnPerfectBump(leaf) should fire exactly one effect"
        );
        assert_eq!(captured.0[0].0, TriggerChain::Shockwave { range: 64.0 });
    }

    #[test]
    fn perfect_bump_with_active_non_leaf_arms_bolt() {
        // Full surge chain: OnPerfectBump(OnImpact(Shockwave{64}))
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
            TriggerChain::Shockwave { range: 64.0 },
        ))));
        let mut app = bump_test_app(vec![chain]);
        let bolt_entity = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.5,
            bolt: bolt_entity,
        });
        tick(&mut app);

        // Should NOT fire -- inner is not a leaf
        let captured = app.world().resource::<CapturedEffects>();
        assert!(
            captured.0.is_empty(),
            "non-leaf inner should not fire, should arm instead"
        );

        // Should arm the bolt entity with the inner chain
        let armed = app.world().get::<ArmedTriggers>(bolt_entity);
        assert!(armed.is_some(), "bolt entity should have ArmedTriggers");
        let armed = armed.unwrap();
        assert_eq!(armed.0.len(), 1);
        assert_eq!(
            armed.0[0],
            TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 }))
        );
    }

    #[test]
    fn early_bump_does_not_fire() {
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let mut app = bump_test_app(vec![chain]);
        let bolt_entity = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            multiplier: 1.1,
            bolt: bolt_entity,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(
            captured.0.is_empty(),
            "early bump should not trigger OnPerfectBump chain"
        );
        assert!(
            app.world().get::<ArmedTriggers>(bolt_entity).is_none(),
            "early bump should not arm bolt"
        );
    }

    #[test]
    fn bolt_specific_arming() {
        // Two bolts, perfect bump only references bolt A
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
            TriggerChain::Shockwave { range: 64.0 },
        ))));
        let mut app = bump_test_app(vec![chain]);
        let bolt_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.5,
            bolt: bolt_a,
        });
        tick(&mut app);

        assert!(
            app.world().get::<ArmedTriggers>(bolt_a).is_some(),
            "bolt A should be armed"
        );
        assert!(
            app.world().get::<ArmedTriggers>(bolt_b).is_none(),
            "bolt B should NOT be armed"
        );
    }

    // --- Impact bridge tests ---

    #[test]
    fn impact_with_armed_trigger_fires() {
        let active_chains: Vec<TriggerChain> = vec![];
        let mut app = impact_test_app(active_chains);

        // Bolt with an armed OnImpact(Shockwave) chain
        let bolt_entity = app
            .world_mut()
            .spawn(ArmedTriggers(vec![TriggerChain::OnImpact(Box::new(
                TriggerChain::Shockwave { range: 64.0 },
            ))]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt: bolt_entity,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "impact on armed bolt should fire the leaf effect"
        );
        assert_eq!(captured.0[0].0, TriggerChain::Shockwave { range: 64.0 });

        // ArmedTriggers entry should be removed after firing
        let armed = app.world().get::<ArmedTriggers>(bolt_entity).unwrap();
        assert!(armed.0.is_empty(), "fired armed trigger should be removed");
    }

    #[test]
    fn impact_without_armed_triggers_does_nothing() {
        let active_chains: Vec<TriggerChain> = vec![];
        let mut app = impact_test_app(active_chains);

        // Bolt without ArmedTriggers
        let bolt_entity = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt: bolt_entity,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(
            captured.0.is_empty(),
            "impact without armed triggers should not fire any effect"
        );
    }

    #[test]
    fn full_surge_chain_two_step() {
        // Step 1: PerfectBump arms the bolt with OnImpact(Shockwave{64})
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::OnImpact(Box::new(
            TriggerChain::Shockwave { range: 64.0 },
        ))));

        // Build an app with BOTH bridges
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .insert_resource(ActiveOverclocks(vec![chain]))
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedEffects>()
            .add_observer(capture_effects)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_overclock_bump,
                    send_bolt_hit_cell,
                    bridge_overclock_impact,
                )
                    .chain(),
            );

        let bolt_entity = app.world_mut().spawn_empty().id();

        // Step 1: Perfect bump -- should arm the bolt
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.5,
            bolt: bolt_entity,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert!(captured.0.is_empty(), "step 1: should arm, not fire");
        let armed = app.world().get::<ArmedTriggers>(bolt_entity);
        assert!(armed.is_some(), "step 1: bolt should be armed");

        // Clear the bump message for step 2
        app.world_mut().resource_mut::<SendBump>().0 = None;

        // Step 2: Impact -- should fire the shockwave
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt: bolt_entity,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "step 2: impact on armed bolt should fire shockwave"
        );
        assert_eq!(captured.0[0].0, TriggerChain::Shockwave { range: 64.0 });
    }

    // --- Cell destroyed bridge tests ---

    #[test]
    fn cell_destroyed_fires_active_chains() {
        let chain =
            TriggerChain::OnCellDestroyed(Box::new(TriggerChain::Shockwave { range: 32.0 }));
        let mut app = cell_destroyed_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(CellDestroyed {
            was_required_to_clear: true,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell destroyed with active OnCellDestroyed chain should fire"
        );
        assert_eq!(captured.0[0].0, TriggerChain::Shockwave { range: 32.0 });
    }

    // --- Bolt lost bridge tests ---

    #[test]
    fn bolt_lost_fires_active_chains() {
        let chain = TriggerChain::OnBoltLost(Box::new(TriggerChain::Shield { duration: 5.0 }));
        let mut app = bolt_lost_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt lost with active OnBoltLost chain should fire"
        );
        assert_eq!(captured.0[0].0, TriggerChain::Shield { duration: 5.0 });
    }

    // --- Bolt entity propagation tests ---

    #[test]
    fn effect_fired_carries_bolt_entity() {
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let mut app = bump_test_app(vec![chain]);
        let bolt_a = app.world_mut().spawn_empty().id();
        let _bolt_b = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.5,
            bolt: bolt_a,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "perfect bump with active OnPerfectBump(leaf) should fire exactly one effect"
        );
        assert_eq!(
            captured.0[0].0,
            TriggerChain::Shockwave { range: 64.0 },
            "effect should be the leaf shockwave"
        );
        assert_eq!(
            captured.0[0].1, bolt_a,
            "effect should carry the bolt entity that triggered it, not PLACEHOLDER"
        );
    }

    #[test]
    fn global_trigger_uses_placeholder_bolt() {
        let chain =
            TriggerChain::OnCellDestroyed(Box::new(TriggerChain::Shockwave { range: 32.0 }));
        let mut app = cell_destroyed_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(CellDestroyed {
            was_required_to_clear: true,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedEffects>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell destroyed with active OnCellDestroyed chain should fire"
        );
        assert_eq!(
            captured.0[0].0,
            TriggerChain::Shockwave { range: 32.0 },
            "effect should be the leaf shockwave"
        );
        assert_eq!(
            captured.0[0].1,
            Entity::PLACEHOLDER,
            "global triggers should use Entity::PLACEHOLDER since no specific bolt triggered them"
        );
    }
}
