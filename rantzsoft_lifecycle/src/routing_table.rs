//! Routing table — stores all routes for a single state type.

use std::fmt;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::route::{FinalizedRoute, IntoFinalizedRoute};

/// Stores all routes for state type `S`. One route per `from` variant.
///
/// Insert via [`RoutingTable::add`] or the [`RoutingTableAppExt::add_route`]
/// convenience method on `App`.
#[derive(Resource)]
pub struct RoutingTable<S: States> {
    /// The registered routes, keyed by source state variant.
    pub routes: HashMap<S, FinalizedRoute<S>>,
}

impl<S: States> Default for RoutingTable<S> {
    fn default() -> Self {
        Self {
            routes: HashMap::default(),
        }
    }
}

/// Error returned when a duplicate route is added for the same `from` variant.
#[derive(Debug)]
pub struct DuplicateRouteError {
    /// The state type name.
    pub state_type: &'static str,
    /// The `from` variant that was duplicated.
    pub variant: String,
}

impl fmt::Display for DuplicateRouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "duplicate route for {}::{} — only one route per source variant",
            self.state_type, self.variant
        )
    }
}

impl<S: States + Eq + std::hash::Hash> RoutingTable<S> {
    /// Add a finalized route to the table.
    ///
    /// # Errors
    ///
    /// Returns [`DuplicateRouteError`] if a route already exists for the
    /// same `from` variant.
    pub fn add<R: IntoFinalizedRoute<S>>(&mut self, route: R) -> Result<(), DuplicateRouteError> {
        let route = route.finalize();
        let from = route.from.clone();
        if self.routes.contains_key(&from) {
            return Err(DuplicateRouteError {
                state_type: std::any::type_name::<S>(),
                variant: format!("{from:?}"),
            });
        }
        self.routes.insert(from, route);
        Ok(())
    }

    /// Returns the number of registered routes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Returns `true` if no routes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

/// Convenience extension for adding routes to a Bevy `App`.
///
/// Panics on duplicate routes — use [`RoutingTable::add`] directly for
/// fallible insertion.
pub trait RoutingTableAppExt {
    /// Add a route to the app's routing table. Panics on duplicate.
    fn add_route<S: States + Eq + std::hash::Hash, R: IntoFinalizedRoute<S>>(
        &mut self,
        route: R,
    ) -> &mut Self;
}

impl RoutingTableAppExt for App {
    fn add_route<S: States + Eq + std::hash::Hash, R: IntoFinalizedRoute<S>>(
        &mut self,
        route: R,
    ) -> &mut Self {
        let result = self
            .world_mut()
            .resource_mut::<RoutingTable<S>>()
            .add(route);
        if let Err(e) = result {
            tracing::error!("{e}");
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::Route;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
        B,
        C,
    }

    #[test]
    fn add_route_succeeds() {
        let mut table = RoutingTable::<TestState>::default();
        let result = table.add(Route::from(TestState::A).to(TestState::B));
        assert!(result.is_ok());
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn add_multiple_routes_from_different_variants() {
        let mut table = RoutingTable::<TestState>::default();
        assert!(
            table
                .add(Route::from(TestState::A).to(TestState::B))
                .is_ok(),
            "first route should succeed"
        );
        assert!(
            table
                .add(Route::from(TestState::B).to(TestState::C))
                .is_ok(),
            "second route (different from) should succeed"
        );
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn duplicate_route_returns_error() {
        let mut table = RoutingTable::<TestState>::default();
        assert!(
            table
                .add(Route::from(TestState::A).to(TestState::B))
                .is_ok(),
            "first route should succeed"
        );
        let result = table.add(Route::from(TestState::A).to(TestState::C));
        assert!(result.is_err(), "expected duplicate route error");
        if let Err(err) = result {
            assert!(
                err.variant.contains('A'),
                "error should name the duplicate variant: {err}"
            );
        }
    }

    #[test]
    fn empty_table() {
        let table = RoutingTable::<TestState>::default();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn app_ext_adds_route() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RoutingTable<TestState>>();
        app.add_route(Route::from(TestState::A).to(TestState::B));

        let table = app.world().resource::<RoutingTable<TestState>>();
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn app_ext_logs_error_on_duplicate() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RoutingTable<TestState>>();
        app.add_route(Route::from(TestState::A).to(TestState::B));
        // Second add for same variant logs error but doesn't panic
        app.add_route(Route::from(TestState::A).to(TestState::C));

        let table = app.world().resource::<RoutingTable<TestState>>();
        assert_eq!(table.len(), 1, "duplicate should not be added");
    }

    #[test]
    fn dynamic_route_can_be_added() {
        let mut table = RoutingTable::<TestState>::default();
        let result = table.add(Route::from(TestState::A).to_dynamic(|_| TestState::C));
        assert!(result.is_ok());
    }

    #[test]
    fn condition_triggered_route_can_be_added() {
        let mut table = RoutingTable::<TestState>::default();
        let result = table.add(Route::from(TestState::A).to(TestState::B).when(|_| true));
        assert!(result.is_ok());
    }
}
