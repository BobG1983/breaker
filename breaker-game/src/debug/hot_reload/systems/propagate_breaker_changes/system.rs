//! System to propagate `BreakerDefinition` registry changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_defaults::prelude::DefaultsHandle;

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
};

/// Bundled system parameters for the breaker change propagation system.
#[derive(SystemParam)]
pub(crate) struct BreakerChangeContext<'w, 's> {
    /// Handle to the breaker defaults asset (for looking up the correct asset).
    defaults_handle: Option<Res<'w, DefaultsHandle<BreakerDefaults>>>,
    /// Loaded breaker defaults assets.
    defaults_assets: Res<'w, Assets<BreakerDefaults>>,
    /// Currently selected breaker name.
    selected: Res<'w, SelectedBreaker>,
    /// Breaker registry (rebuilt by `propagate_registry`).
    registry: Res<'w, BreakerRegistry>,
    /// Mutable breaker configuration.
    config: ResMut<'w, BreakerConfig>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Breaker `EffectChains` for populating from definition.
    breaker_chains_query: Query<'w, 's, &'static mut EffectChains, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects when `propagate_registry` has rebuilt the `BreakerRegistry`
/// and if the selected breaker was modified:
/// 1. Resets `BreakerConfig` from defaults + re-applies stat overrides
/// 2. Resets `LivesCount` if breaker has `life_pool`
/// 3. Rebuilds breaker entity `EffectChains`
pub(crate) fn propagate_breaker_changes(mut ctx: BreakerChangeContext) {
    if !ctx.registry.is_changed() || ctx.registry.is_added() {
        return;
    }

    // Check if the selected breaker exists in the registry
    let Some(def) = ctx.registry.get(&ctx.selected.0) else {
        return;
    };
    let def = def.clone();

    // Reset BreakerConfig from defaults + re-apply stat overrides
    if let Some(loaded) = ctx
        .defaults_handle
        .as_ref()
        .and_then(|h| ctx.defaults_assets.get(h.0.id()))
    {
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
