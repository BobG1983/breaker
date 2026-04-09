//! System to propagate `NodeLayout` registry changes — despawn and respawn cells.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    cells::{
        components::Cell,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::PlayfieldConfig,
    state::run::node::{
        ActiveNodeLayout, ClearRemainingCount, NodeLayoutRegistry,
        systems::{HpContext, RenderAssets, ToughnessHpData, spawn_cells_from_grid},
    },
};

/// Bundled system parameters for the layout change propagation system.
#[derive(SystemParam)]
pub(crate) struct LayoutChangeContext<'w, 's> {
    /// Cell dimensions and padding configuration.
    cell_config: Res<'w, CellConfig>,
    /// Playfield boundaries.
    playfield: Res<'w, PlayfieldConfig>,
    /// Currently active layout (if any).
    active_layout: Option<Res<'w, ActiveNodeLayout>>,
    /// Node layout registry (rebuilt by `propagate_registry`).
    registry: Res<'w, NodeLayoutRegistry>,
    /// Cell type definitions for spawning.
    cell_type_registry: Res<'w, CellTypeRegistry>,
    /// Existing cell entities to despawn on layout change.
    cell_query: Query<'w, 's, Entity, With<Cell>>,
    /// Command buffer for entity spawn/despawn.
    commands: Commands<'w, 's>,
    /// Mesh asset storage.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// Material asset storage.
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

/// Detects when `propagate_registry` has rebuilt the `NodeLayoutRegistry`
/// or when `CellConfig` has changed, and if the active layout was affected,
/// despawns all cells and respawns from the updated layout.
pub(crate) fn propagate_node_layout_changes(mut ctx: LayoutChangeContext) {
    let registry_changed = ctx.registry.is_changed() && !ctx.registry.is_added();
    let cell_config_changed = ctx.cell_config.is_changed() && !ctx.cell_config.is_added();

    if !registry_changed && !cell_config_changed {
        return;
    }

    // If we have an active layout, check if it was modified (by name match)
    let Some(active) = &ctx.active_layout else {
        return;
    };

    let updated_layout = if registry_changed {
        ctx.registry.get_by_name(&active.0.name).cloned()
    } else {
        // Cell config changed — respawn with same layout
        Some(active.0.clone())
    };

    let Some(layout) = updated_layout else {
        return;
    };

    // Despawn all existing cells directly (avoid destruction pipeline)
    for entity in &ctx.cell_query {
        ctx.commands.entity(entity).despawn();
    }

    // Respawn cells from updated layout (hot-reload uses default toughness fallback)
    let required_count = spawn_cells_from_grid(
        &mut ctx.commands,
        &ctx.cell_config,
        &ctx.playfield,
        &layout,
        &ctx.cell_type_registry,
        RenderAssets {
            meshes: &mut ctx.meshes,
            materials: &mut ctx.materials,
        },
        ToughnessHpData {
            toughness_config: None,
            hp_context: HpContext {
                tier: 0,
                position_in_tier: 0,
                is_boss: false,
            },
        },
    );

    // Update active layout and clear remaining count
    ctx.commands.insert_resource(ActiveNodeLayout(layout));
    ctx.commands.insert_resource(ClearRemainingCount {
        remaining: required_count,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{CellTypeDefinition, components::CellTypeAlias, definition::Toughness},
        state::run::node::{NodeLayout, definition::NodePool},
    };

    fn test_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            "S".to_owned(),
            CellTypeDefinition {
                id: "standard".to_owned(),
                alias: "S".to_owned(),
                toughness: Toughness::default(),
                color_rgb: [4.0, 0.2, 0.5],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behaviors: None,

                effects: None,
            },
        );
        registry.insert(
            "T".to_owned(),
            CellTypeDefinition {
                id: "tough".to_owned(),
                alias: "T".to_owned(),
                toughness: Toughness::default(),
                color_rgb: [2.5, 0.2, 4.0],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behaviors: None,

                effects: None,
            },
        );
        registry
    }

    fn make_layout(name: &str, grid: Vec<Vec<String>>) -> NodeLayout {
        let rows = u32::try_from(grid.len()).unwrap();
        let cols = if grid.is_empty() {
            0
        } else {
            u32::try_from(grid[0].len()).unwrap()
        };
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols,
            rows,
            grid_top_offset: 50.0,
            grid,
            pool: NodePool::default(),
            entity_scale: 1.0,
            locks: None,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ColorMaterial>()
            .init_asset::<Mesh>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<NodeLayoutRegistry>()
            .insert_resource(test_registry())
            .add_systems(Update, propagate_node_layout_changes);
        app
    }

    #[test]
    fn respawns_cells_when_registry_changed() {
        let mut app = test_app();

        // Create initial layout
        let initial_layout = make_layout("test", vec![vec!["S".to_owned(), "S".to_owned()]]);
        app.world_mut()
            .insert_resource(ActiveNodeLayout(initial_layout.clone()));

        // Seed registry with initial layout
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.insert(initial_layout);
        }

        // Flush Added change detection (system should skip Added)
        app.update();
        app.update();

        // Count cells after initial state (no cells spawned — system skipped Added)
        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(cell_count, 0, "no cells until registry is mutated");

        // Mutate registry — simulates propagate_registry rebuild with updated layout
        let updated_layout = make_layout(
            "test",
            vec![vec!["S".to_owned(), "T".to_owned(), "S".to_owned()]],
        );
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.clear();
            registry.insert(updated_layout);
        }

        app.update();
        // Need another update for despawn commands to flush
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(cell_count, 3, "should have 3 cells after registry change");
    }

    #[test]
    fn clear_remaining_count_updated_after_respawn() {
        let mut app = test_app();

        let layout = make_layout("test", vec![vec!["S".to_owned(), "T".to_owned()]]);
        app.world_mut()
            .insert_resource(ActiveNodeLayout(layout.clone()));
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.insert(layout);
        }
        app.insert_resource(ClearRemainingCount { remaining: 99 });

        // Flush Added
        app.update();
        app.update();

        // Modify registry to trigger respawn
        let updated_layout = make_layout(
            "test",
            vec![vec!["S".to_owned(), "S".to_owned(), "S".to_owned()]],
        );
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.clear();
            registry.insert(updated_layout);
        }

        app.update();
        app.update();

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 3,
            "ClearRemainingCount should reflect new layout"
        );
    }

    #[test]
    fn old_cells_despawned_on_layout_change() {
        let mut app = test_app();

        let layout = make_layout("test", vec![vec!["S".to_owned()]]);
        app.world_mut()
            .insert_resource(ActiveNodeLayout(layout.clone()));
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.insert(layout);
        }

        // Manually spawn some "old" cell entities
        app.world_mut().spawn((Cell, CellTypeAlias("S".to_owned())));
        app.world_mut().spawn((Cell, CellTypeAlias("S".to_owned())));
        app.world_mut().spawn((Cell, CellTypeAlias("S".to_owned())));

        // Flush Added
        app.update();
        app.update();

        // Modify registry to 1 cell
        let updated_layout = make_layout("test", vec![vec!["T".to_owned()]]);
        {
            let mut registry = app.world_mut().resource_mut::<NodeLayoutRegistry>();
            registry.clear();
            registry.insert(updated_layout);
        }

        app.update();
        // Need another update for despawn commands to flush
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(
            cell_count, 1,
            "old cells should be despawned, only new cells present"
        );
    }
}
