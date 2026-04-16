//! Breaker registry — maps breaker names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;
use tracing::warn;

use super::definition::BreakerDefinition;

/// Registry of all loaded breaker definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub struct BreakerRegistry {
    /// Map from breaker name to its definition.
    breakers: HashMap<String, BreakerDefinition>,
}

impl BreakerRegistry {
    /// Returns a reference to the definition for `name`, if it exists.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&BreakerDefinition> {
        self.breakers.get(name)
    }

    /// Returns `true` if the registry contains a definition for `name`.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.breakers.contains_key(name)
    }

    /// Inserts a definition into the registry under the given `name`.
    pub fn insert(&mut self, name: String, def: BreakerDefinition) {
        self.breakers.insert(name, def);
    }

    /// Returns an iterator over all breaker names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.breakers.keys()
    }

    /// Returns an iterator over all `(name, definition)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &BreakerDefinition)> {
        self.breakers.iter()
    }

    /// Returns an iterator over all definitions.
    pub fn values(&self) -> impl Iterator<Item = &BreakerDefinition> {
        self.breakers.values()
    }

    /// Removes all entries from the registry.
    pub fn clear(&mut self) {
        self.breakers.clear();
    }

    /// Returns the number of breakers in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.breakers.len()
    }

    /// Returns `true` if the registry contains no breakers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.breakers.is_empty()
    }
}

impl SeedableRegistry for BreakerRegistry {
    type Asset = BreakerDefinition;

    fn asset_dir() -> &'static str {
        "breakers"
    }

    fn extensions() -> &'static [&'static str] {
        &["breaker.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<BreakerDefinition>, BreakerDefinition)]) {
        self.breakers.clear();
        for (_id, def) in assets {
            if self.breakers.contains_key(&def.name) {
                warn!("duplicate breaker name '{}' — skipping", def.name);
                continue;
            }
            self.breakers.insert(def.name.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<BreakerDefinition>, asset: &BreakerDefinition) {
        self.breakers.insert(asset.name.clone(), asset.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_registry_is_empty() {
        let registry = BreakerRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn insert_and_lookup() {
        let mut registry = BreakerRegistry::default();
        let ron_str = include_str!("../../assets/breakers/aegis.breaker.ron");
        let def: BreakerDefinition = ron::de::from_str(ron_str).expect("aegis RON should parse");
        registry.insert(def.name.clone(), def);
        assert!(registry.contains("Aegis"));
    }

    // ── SeedableRegistry tests ──────────────────────────────────────

    /// Creates a `BreakerDefinition` for testing with the given name and
    /// optional `life_pool`.
    fn make_breaker(name: &str, life_pool: Option<u32>) -> BreakerDefinition {
        ron::de::from_str(&format!(
            r#"(name: "{name}", life_pool: {lp}, bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))), projectile_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#,
            lp = life_pool.map_or_else(|| "None".to_string(), |n| format!("Some({n})")),
        ))
        .expect("test RON should parse")
    }

    /// Helper: creates an `App` with `AssetPlugin` and returns `AssetId`s for
    /// a list of `BreakerDefinition` values. This gives us real (non-default)
    /// `AssetId`s backed by the Bevy asset system.
    fn asset_ids_for(
        defs: &[BreakerDefinition],
    ) -> (App, Vec<(AssetId<BreakerDefinition>, BreakerDefinition)>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<BreakerDefinition>();

        let pairs: Vec<_> = {
            let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
            defs.iter()
                .map(|d| {
                    let handle = assets.add(d.clone());
                    (handle.id(), d.clone())
                })
                .collect()
        };

        (app, pairs)
    }

    // ── Behavior 1: seed() populates from asset pairs ───────────────

    /// `seed()` populates the registry from 2 `BreakerDefinition` assets
    /// ("Aegis" with `life_pool` 3, "Vortex" with no `life_pool`).
    #[test]
    fn seed_populates_registry_from_breaker_definitions() {
        let aegis = make_breaker("Aegis", Some(3));
        let vortex = make_breaker("Vortex", None);
        let (_app, pairs) = asset_ids_for(&[aegis, vortex]);

        let mut registry = BreakerRegistry::default();
        registry.seed(&pairs);

        assert_eq!(registry.len(), 2, "registry should contain 2 breakers");
        assert!(
            registry.get("Aegis").is_some(),
            "registry should contain Aegis"
        );
        assert!(
            registry.get("Vortex").is_some(),
            "registry should contain Vortex"
        );
        assert_eq!(
            registry.get("Aegis").unwrap().life_pool,
            Some(3),
            "Aegis life_pool should be Some(3)"
        );
    }

    // ── Behavior 2: seed() clears existing entries ──────────────────

    /// `seed()` clears previously inserted entries before populating.
    #[test]
    fn seed_clears_existing_entries() {
        let old = make_breaker("Old", Some(1));
        let aegis = make_breaker("Aegis", Some(3));
        let (_app, pairs_old) = asset_ids_for(&[old]);
        let (_app2, pairs_new) = asset_ids_for(&[aegis]);

        let mut registry = BreakerRegistry::default();
        registry.seed(&pairs_old);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Old").is_some());

        // Seed again with only Aegis
        registry.seed(&pairs_new);

        assert_eq!(registry.len(), 1, "after re-seed, only Aegis should remain");
        assert!(
            registry.get("Aegis").is_some(),
            "Aegis should be present after re-seed"
        );
        assert!(
            registry.get("Old").is_none(),
            "Old should be gone after re-seed"
        );
    }

    // ── Behavior 3: seed() skips duplicate name with warning ────────

    /// `seed()` skips the second definition when two share the same name,
    /// keeping the first occurrence.
    #[test]
    fn seed_skips_duplicate_breaker_name() {
        let aegis1 = make_breaker("Aegis", Some(3));
        let aegis2 = make_breaker("Aegis", Some(5));
        let (_app, pairs) = asset_ids_for(&[aegis1, aegis2]);

        let mut registry = BreakerRegistry::default();
        registry.seed(&pairs);

        assert_eq!(registry.len(), 1, "duplicate should be skipped");
        assert_eq!(
            registry.get("Aegis").unwrap().life_pool,
            Some(3),
            "first occurrence (life_pool=3) should win"
        );
    }

    // ── Behavior 4: update_single() upserts by name ─────────────────

    /// `update_single()` updates an existing breaker's fields by name.
    #[test]
    fn update_single_upserts_existing_breaker_by_name() {
        let aegis = make_breaker("Aegis", Some(3));
        let (_app, pairs) = asset_ids_for(&[aegis]);

        let mut registry = BreakerRegistry::default();
        registry.seed(&pairs);
        assert_eq!(
            registry.get("Aegis").unwrap().life_pool,
            Some(3),
            "Aegis life_pool should be Some(3) initially"
        );

        // Update with life_pool = 5
        let updated = make_breaker("Aegis", Some(5));
        let id = pairs[0].0;
        registry.update_single(id, &updated);

        assert_eq!(
            registry.get("Aegis").unwrap().life_pool,
            Some(5),
            "Aegis life_pool should be Some(5) after update_single"
        );
    }

    // ── Behavior 5: update_all() resets and re-seeds ────────────────

    /// `update_all()` resets to default then seeds, removing old entries.
    #[test]
    fn update_all_resets_and_reseeds() {
        let old = make_breaker("Old", Some(1));
        let aegis = make_breaker("Aegis", Some(3));
        let (_app, pairs_old) = asset_ids_for(&[old]);
        let (_app2, pairs_new) = asset_ids_for(&[aegis]);

        let mut registry = BreakerRegistry::default();
        registry.seed(&pairs_old);
        assert!(registry.get("Old").is_some());

        registry.update_all(&pairs_new);

        assert_eq!(
            registry.len(),
            1,
            "after update_all, only Aegis should remain"
        );
        assert!(
            registry.get("Aegis").is_some(),
            "Aegis should be present after update_all"
        );
        assert!(
            registry.get("Old").is_none(),
            "Old should be gone after update_all"
        );
    }

    // ── Behavior 6: asset_dir() and extensions() ────────────────────

    /// `asset_dir()` returns "breakers".
    #[test]
    fn asset_dir_returns_breakers() {
        assert_eq!(
            BreakerRegistry::asset_dir(),
            "breakers",
            "asset_dir() should return \"breakers\""
        );
    }

    /// `extensions()` returns `&["breaker.ron"]`.
    #[test]
    fn extensions_returns_breaker_ron() {
        assert_eq!(
            BreakerRegistry::extensions(),
            &["breaker.ron"],
            "extensions() should return [\"breaker.ron\"]"
        );
    }
}
