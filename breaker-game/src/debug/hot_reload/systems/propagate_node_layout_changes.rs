//! System to propagate `NodeLayout` asset changes — despawn and respawn cells.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    cells::{
        components::Cell,
        resources::{CellConfig, CellTypeRegistry},
    },
    run::node::{
        ActiveNodeLayout, ClearRemainingCount, NodeLayout, NodeLayoutRegistry,
        systems::spawn_cells_from_grid,
    },
    screen::loading::resources::DefaultsCollection,
    shared::PlayfieldConfig,
};

/// Bundled system parameters for the layout change propagation system.
#[derive(SystemParam)]
pub(crate) struct LayoutChangeContext<'w, 's> {
    /// Asset event source for node layouts.
    collection: Res<'w, DefaultsCollection>,
    /// Loaded node layout assets.
    layout_assets: Res<'w, Assets<NodeLayout>>,
    /// Cell dimensions and padding configuration.
    cell_config: Res<'w, CellConfig>,
    /// Playfield boundaries.
    playfield: Res<'w, PlayfieldConfig>,
    /// Currently active layout (if any).
    active_layout: Option<Res<'w, ActiveNodeLayout>>,
    /// Mutable registry of available node layouts.
    registry: ResMut<'w, NodeLayoutRegistry>,
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

/// Detects `AssetEvent::Modified` on any `NodeLayout`, rebuilds
/// `NodeLayoutRegistry`, and if the active layout was modified,
/// despawns all cells and respawns from the updated layout.
///
/// Also triggers on `CellConfig` changes (grid positioning depends on
/// cell dimensions/padding).
pub(crate) fn propagate_node_layout_changes(
    mut events: MessageReader<AssetEvent<NodeLayout>>,
    mut ctx: LayoutChangeContext,
) {
    // Check for any modified layout
    let any_layout_modified = events.read().any(|event| {
        ctx.collection
            .layouts
            .iter()
            .any(|h| event.is_modified(h.id()))
    });

    let cell_config_changed = ctx.cell_config.is_changed() && !ctx.cell_config.is_added();

    if !any_layout_modified && !cell_config_changed {
        return;
    }

    // Rebuild layout registry
    if any_layout_modified {
        ctx.registry.layouts.clear();
        for handle in &ctx.collection.layouts {
            if let Some(layout) = ctx.layout_assets.get(handle.id()) {
                ctx.registry.layouts.push(layout.clone());
            }
        }
    }

    // If we have an active layout, check if it was modified (by name match)
    let Some(active) = &ctx.active_layout else {
        return;
    };

    let updated_layout = if any_layout_modified {
        ctx.registry
            .layouts
            .iter()
            .find(|l| l.name == active.0.name)
            .cloned()
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

    // Respawn cells from updated layout
    let required_count = spawn_cells_from_grid(
        &mut ctx.commands,
        &ctx.cell_config,
        &ctx.playfield,
        &layout,
        &ctx.cell_type_registry,
        &mut ctx.meshes,
        &mut ctx.materials,
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
    use crate::cells::{components::CellTypeAlias, resources::CellTypeDefinition};

    fn test_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.types.insert(
            'S',
            CellTypeDefinition {
                id: "standard".to_owned(),
                alias: 'S',
                hp: 1,
                color_rgb: [4.0, 0.2, 0.5],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
            },
        );
        registry.types.insert(
            'T',
            CellTypeDefinition {
                id: "tough".to_owned(),
                alias: 'T',
                hp: 3,
                color_rgb: [2.5, 0.2, 4.0],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
            },
        );
        registry
    }

    fn make_layout(name: &str, grid: Vec<Vec<char>>) -> NodeLayout {
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
        }
    }

    fn make_collection(layouts: Vec<Handle<NodeLayout>>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types: vec![],
            layouts,
            archetypes: vec![],
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<NodeLayout>()
            .init_asset::<ColorMaterial>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .init_resource::<NodeLayoutRegistry>()
            .insert_resource(test_registry())
            .add_systems(Update, propagate_node_layout_changes);
        app
    }

    #[test]
    fn respawns_cells_when_layout_modified() {
        let mut app = test_app();

        // Create initial layout with 2 cells
        let initial_layout = make_layout("test", vec![vec!['S', 'S']]);
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            assets.add(initial_layout.clone())
        };

        app.world_mut()
            .insert_resource(ActiveNodeLayout(initial_layout));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        // Flush Added event
        app.update();
        app.update();

        // Count cells after initial state (no cells spawned — no Modified yet)
        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(cell_count, 0, "no cells until layout is modified");

        // Modify layout: now 3 cells
        {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.grid = vec![vec!['S', 'T', 'S']];
            asset.cols = 3;
            asset.rows = 1;
        }

        // Flush Modified
        app.update();
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(cell_count, 3, "should have 3 cells after layout change");
    }

    #[test]
    fn clear_remaining_count_updated_after_respawn() {
        let mut app = test_app();

        let layout = make_layout("test", vec![vec!['S', 'T']]);
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            assets.add(layout.clone())
        };

        app.world_mut().insert_resource(ActiveNodeLayout(layout));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));
        app.insert_resource(ClearRemainingCount { remaining: 99 });

        app.update();
        app.update();

        // Modify to trigger respawn
        {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.grid = vec![vec!['S', 'S', 'S']];
            asset.cols = 3;
            asset.rows = 1;
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

        let layout = make_layout("test", vec![vec!['S']]);
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            assets.add(layout.clone())
        };

        app.world_mut().insert_resource(ActiveNodeLayout(layout));
        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        // Manually spawn some "old" cell entities
        app.world_mut().spawn((Cell, CellTypeAlias('S')));
        app.world_mut().spawn((Cell, CellTypeAlias('S')));
        app.world_mut().spawn((Cell, CellTypeAlias('S')));

        app.update();
        app.update();

        // Modify layout to 1 cell
        {
            let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.grid = vec![vec!['T']];
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
