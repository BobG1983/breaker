//! Debug UI panel system.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::system::SystemParam,
    prelude::*,
};
use bevy_egui::EguiContexts;

use crate::{
    debug::resources::{DebugOverlays, Overlay},
    prelude::*,
};

/// Bundled state queries for the state chain display.
#[derive(SystemParam)]
pub(crate) struct StateChain<'w> {
    app: Option<Res<'w, State<AppState>>>,
    game: Option<Res<'w, State<GameState>>>,
    menu: Option<Res<'w, State<MenuState>>>,
    run: Option<Res<'w, State<RunState>>>,
    node: Option<Res<'w, State<NodeState>>>,
    chip: Option<Res<'w, State<ChipSelectState>>>,
}

impl StateChain<'_> {
    /// Builds a display string showing the full active state hierarchy.
    ///
    /// Example outputs: `Loading`, `Game > Menu > Main`,
    /// `Game > Run > Node > Playing`, `Game > Run > ChipSelect > Selecting`.
    fn display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        let Some(ref app) = self.app else {
            return "--".into();
        };
        parts.push(format!("{:?}", app.get()));

        let Some(ref g) = self.game else {
            return parts.join(" > ");
        };
        parts.push(format!("{:?}", g.get()));

        // Exhaustive matches — adding a new state variant is a compile error
        // until the corresponding arm is added here.
        match g.get() {
            GameState::Menu => {
                if let Some(ref m) = self.menu {
                    parts.push(format!("{:?}", m.get()));
                }
            }
            GameState::Run => {
                if let Some(ref r) = self.run {
                    parts.push(format!("{:?}", r.get()));

                    match r.get() {
                        RunState::Node => {
                            if let Some(ref n) = self.node {
                                parts.push(format!("{:?}", n.get()));
                            }
                        }
                        RunState::ChipSelect => {
                            if let Some(ref c) = self.chip {
                                parts.push(format!("{:?}", c.get()));
                            }
                        }
                        RunState::Loading
                        | RunState::Setup
                        | RunState::RunEnd
                        | RunState::Teardown => {}
                    }
                }
            }
            GameState::Loading | GameState::Teardown => {}
        }

        parts.join(" > ")
    }
}

/// Renders the debug overlay toggles and FPS counter.
pub(crate) fn debug_ui_system(
    mut contexts: EguiContexts,
    mut overlays: ResMut<DebugOverlays>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Debug")
        .default_open(true)
        .show(ctx, |ui| {
            ui.heading("Overlays");
            ui.checkbox(overlays.flag_mut(Overlay::Fps), "FPS");
            ui.checkbox(overlays.flag_mut(Overlay::Hitboxes), "Hitboxes");
            ui.checkbox(
                overlays.flag_mut(Overlay::VelocityVectors),
                "Velocity Vectors",
            );
            ui.checkbox(overlays.flag_mut(Overlay::BoltInfo), "Bolt Info");
            ui.checkbox(overlays.flag_mut(Overlay::DashState), "Breaker State");
            ui.checkbox(overlays.flag_mut(Overlay::InputActions), "Input Actions");

            ui.separator();

            if overlays.is_active(Overlay::Fps) {
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(bevy::diagnostic::Diagnostic::smoothed)
                {
                    ui.label(format!("FPS: {fps:.1}"));
                } else {
                    ui.label("FPS: --");
                }
            }
        });
}

/// Renders the state chain in a small always-visible window anchored to the top right.
pub(crate) fn state_chain_ui(mut contexts: EguiContexts, states: StateChain) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("State")
        .title_bar(false)
        .resizable(false)
        .anchor(bevy_egui::egui::Align2::RIGHT_TOP, [-8.0, 8.0])
        .show(ctx, |ui| {
            ui.label(states.display());
        });
}
