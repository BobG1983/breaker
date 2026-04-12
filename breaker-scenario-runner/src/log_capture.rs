//! Log capture layer for scenario runs.
//!
//! Captures WARN/ERROR log events from the `breaker` target into a shared
//! buffer, which is polled each frame into [`CapturedLogs`] resource.
//! Any captured log = scenario fail.

use std::sync::{Arc, Mutex};

use bevy::{
    log::{BoxedLayer, Level},
    prelude::*,
};
use tracing::Subscriber;
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

/// A single captured log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level (WARN or ERROR).
    pub level:   Level,
    /// Module target (e.g. `breaker::bolt::systems`).
    pub target:  String,
    /// Log message text.
    pub message: String,
    /// Fixed-update frame when captured (populated by the poll system).
    pub frame:   u32,
}

/// Accumulated log entries captured during the scenario run.
///
/// Any entry in this resource causes the scenario to fail.
#[derive(Resource, Default)]
pub struct CapturedLogs(pub Vec<LogEntry>);

/// Shared log buffer written by the tracing layer, drained by Bevy system.
#[derive(Clone, Default, Resource)]
pub struct LogBuffer(pub Arc<Mutex<Vec<(Level, String, String)>>>);

/// Builds the custom tracing layer and returns it alongside its shared buffer.
///
/// Filters to WARN and above from the `breaker` or `breaker_scenario_runner` targets.
#[must_use]
pub fn build_log_layer() -> (BoxedLayer, LogBuffer) {
    let buffer = LogBuffer::default();
    let buffer_clone = buffer.clone();
    let layer = ScenarioLogLayer {
        buffer: buffer_clone,
    };
    (Box::new(layer), buffer)
}

/// Factory function matching `LogPlugin::custom_layer` signature.
///
/// Registers the log buffer as a resource and returns the tracing layer.
pub fn scenario_log_layer_factory(app: &mut App) -> Option<BoxedLayer> {
    let (layer, buffer) = build_log_layer();
    app.insert_resource(buffer);
    Some(layer)
}

/// Polls the log buffer and drains captured entries into [`CapturedLogs`].
pub fn poll_log_buffer(
    buffer: Res<LogBuffer>,
    frame: Res<crate::invariants::ScenarioFrame>,
    mut logs: ResMut<CapturedLogs>,
) {
    let Ok(mut guard) = buffer.0.lock() else {
        return;
    };
    for (level, target, message) in guard.drain(..) {
        logs.0.push(LogEntry {
            level,
            target,
            message,
            frame: frame.0,
        });
    }
}

/// Tracing layer that captures WARN/ERROR from breaker targets.
struct ScenarioLogLayer {
    buffer: LogBuffer,
}

impl<S> Layer<S> for ScenarioLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = *meta.level();

        // Only capture WARN and above
        if level > Level::WARN {
            return;
        }

        // Only capture breaker targets
        let target = meta.target();
        if !target.starts_with("breaker") {
            return;
        }

        // Extract message field
        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        if let Ok(mut guard) = self.buffer.0.lock() {
            guard.push((level, target.to_owned(), visitor.0));
        }
    }
}

/// Visitor that extracts the `message` field from a tracing event.
struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            value.clone_into(&mut self.0);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}
