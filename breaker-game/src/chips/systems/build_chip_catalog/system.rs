//! Builds the `ChipCatalog` from `ChipTemplateRegistry` and `EvolutionTemplateRegistry`.

use bevy::prelude::*;
use iyes_progress::prelude::*;
use rantzsoft_defaults::prelude::RegistryHandles;

use crate::chips::{
    definition::{
        ChipTemplate, EvolutionTemplate, expand_chip_template, expand_evolution_template,
    },
    resources::{ChipCatalog, ChipTemplateRegistry, EvolutionTemplateRegistry, Recipe},
};

/// Builds the `ChipCatalog` resource by expanding all chip templates and
/// evolution templates into `ChipDefinition`s.
///
/// Returns `Progress { done: 0, total: 1 }` while both `RegistryHandles`
/// are not yet loaded. Returns `Progress { done: 1, total: 1 }` once the
/// catalog is built (or was already built on a previous tick).
pub(crate) fn build_chip_catalog(
    template_registry: Res<ChipTemplateRegistry>,
    evolution_registry: Res<EvolutionTemplateRegistry>,
    template_handles: Res<RegistryHandles<ChipTemplate>>,
    evolution_handles: Res<RegistryHandles<EvolutionTemplate>>,
    mut commands: Commands,
    mut built: Local<bool>,
) -> Progress {
    if *built {
        return Progress { done: 1, total: 1 };
    }

    if !template_handles.loaded || !evolution_handles.loaded {
        return Progress { done: 0, total: 1 };
    }

    let mut catalog = ChipCatalog::default();
    populate_catalog(&mut catalog, &template_registry, &evolution_registry);

    commands.insert_resource(catalog);
    *built = true;
    Progress { done: 1, total: 1 }
}

/// Rebuilds `ChipCatalog` when either source registry is updated by hot-reload.
#[cfg(feature = "dev")]
pub(crate) fn propagate_chip_catalog(
    template_registry: Res<ChipTemplateRegistry>,
    evolution_registry: Res<EvolutionTemplateRegistry>,
    mut catalog: ResMut<ChipCatalog>,
) {
    let templates_changed = template_registry.is_changed() && !template_registry.is_added();
    let evolutions_changed = evolution_registry.is_changed() && !evolution_registry.is_added();

    if !templates_changed && !evolutions_changed {
        return;
    }

    *catalog = ChipCatalog::default();
    populate_catalog(&mut catalog, &template_registry, &evolution_registry);
}

fn populate_catalog(
    catalog: &mut ChipCatalog,
    template_registry: &ChipTemplateRegistry,
    evolution_registry: &EvolutionTemplateRegistry,
) {
    let mut templates: Vec<_> = template_registry.templates().collect();
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    for template in templates {
        for def in expand_chip_template(template) {
            catalog.insert(def);
        }
    }

    let mut evolutions: Vec<_> = evolution_registry.templates().collect();
    evolutions.sort_by(|a, b| a.name.cmp(&b.name));
    for template in evolutions {
        let def = expand_evolution_template(template);
        let recipe = Recipe {
            ingredients: template.ingredients.clone(),
            result_name: template.name.clone(),
        };
        catalog.insert_recipe(recipe);
        catalog.insert(def);
    }

    validate_recipe_ingredients(catalog);
}

/// Logs a warning for any recipe ingredient that doesn't match a known template name.
fn validate_recipe_ingredients(catalog: &ChipCatalog) {
    use std::collections::HashSet;

    use tracing::warn;

    let template_names: HashSet<&str> = catalog
        .ordered_values()
        .filter_map(|def| def.template_name.as_deref())
        .collect();

    for recipe in catalog.recipes() {
        for ing in &recipe.ingredients {
            if !template_names.contains(ing.chip_name.as_str()) {
                warn!(
                    "Recipe '{}' ingredient '{}' does not match any known template name",
                    recipe.result_name, ing.chip_name
                );
            }
        }
    }
}
