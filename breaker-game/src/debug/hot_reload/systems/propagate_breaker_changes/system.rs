//! System to propagate `BreakerDefinition` asset changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    breaker::{
        SelectedBreaker,
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{
        definition::{BreakerDefinition, EffectChains, RootEffect, Target},
        effects::life_lost::LivesCount,
        init::apply_stat_overrides,
        registry::BreakerRegistry,
    },
    screen::loading::resources::DefaultsCollection,
};

/// Bundled system parameters for the breaker change propagation system.
#[derive(SystemParam)]
pub(crate) struct BreakerChangeContext<'w, 's> {
    /// Asset collection handles.
    collection: Res<'w, DefaultsCollection>,
    /// Loaded breaker definition assets.
    assets: Res<'w, Assets<BreakerDefinition>>,
    /// Loaded breaker defaults assets.
    defaults_assets: Res<'w, Assets<BreakerDefaults>>,
    /// Currently selected breaker name.
    selected: Res<'w, SelectedBreaker>,
    /// Mutable breaker registry.
    registry: ResMut<'w, BreakerRegistry>,
    /// Mutable breaker configuration.
    config: ResMut<'w, BreakerConfig>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Breaker `EffectChains` for populating from definition.
    breaker_chains_query: Query<'w, 's, &'static mut EffectChains, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects `AssetEvent::Modified` on any `BreakerDefinition`, rebuilds
/// `BreakerRegistry`, and if the selected breaker was modified:
/// 1. Resets `BreakerConfig` from defaults + re-applies stat overrides
/// 2. Resets `LivesCount` if breaker has `life_pool`
/// 3. Rebuilds breaker entity `EffectChains`
pub(crate) fn propagate_breaker_changes(
    mut events: MessageReader<AssetEvent<BreakerDefinition>>,
    mut ctx: BreakerChangeContext,
) {
    let any_modified = events.read().any(|event| {
        ctx.collection
            .breakers
            .iter()
            .any(|h| event.is_modified(h.id()))
    });

    if !any_modified {
        return;
    }

    // Rebuild registry
    ctx.registry.clear();
    for handle in &ctx.collection.breakers {
        if let Some(def) = ctx.assets.get(handle.id()) {
            ctx.registry.insert(def.name.clone(), def.clone());
        }
    }

    // Check if the selected breaker was modified
    let Some(def) = ctx.registry.get(&ctx.selected.0) else {
        return;
    };
    let def = def.clone();

    // Reset BreakerConfig from defaults + re-apply stat overrides
    if let Some(loaded) = ctx.defaults_assets.iter().next().map(|(_, d)| d) {
        *ctx.config = BreakerConfig::from(loaded.clone());
    }
    apply_stat_overrides(&mut ctx.config, &def.stat_overrides);

    // Re-stamp consequence components and lives on breaker entities
    for entity in &ctx.breaker_query {
        if let Some(life_pool) = def.life_pool {
            ctx.commands.entity(entity).insert(LivesCount(life_pool));
        }
    }

    // Resolve On targets to entity EffectChains
    for mut chains in &mut ctx.breaker_chains_query {
        chains.0.clear();
    }
    for root in &def.effects {
        let RootEffect::On { target, then } = root;
        match target {
            Target::Breaker => {
                for mut chains in &mut ctx.breaker_chains_query {
                    for child in then {
                        chains.0.push((None, child.clone()));
                    }
                }
            }
            // At hot-reload time, bolt/cell/wall targets are not resolved here
            Target::Bolt | Target::AllBolts | Target::Cell | Target::Wall | Target::AllCells => {}
        }
    }
}
