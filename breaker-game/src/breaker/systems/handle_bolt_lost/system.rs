//! `handle_bolt_lost` — applies the breaker's [`BoltLossBehavior`] when a
//! [`BoltLost`] message fires for that breaker.

use bevy::prelude::*;

use crate::{
    breaker::components::BoltLossBehavior, prelude::*, state::run::node::messages::ReduceNodeTimer,
};

/// Reads [`BoltLost`] messages and applies the targeted breaker's
/// [`BoltLossBehavior`].
///
/// - `LifeLoss(n)` decrements `Hp.current` by `n` (clamped at 0.0). No-op if
///   the breaker has no `Hp` component.
/// - `TimeLoss(delta)` writes one `ReduceNodeTimer { delta }` per message.
///   Does not touch `Hp` even when present.
/// - `None` is a complete no-op.
///
/// Messages targeting an entity that is no longer in the query (despawned or
/// missing the required components) are silently skipped via `continue`.
pub(crate) fn handle_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut breakers: Query<(&BoltLossBehavior, Option<&mut Hp>), With<Breaker>>,
    mut writer: MessageWriter<ReduceNodeTimer>,
) {
    for msg in reader.read() {
        let Ok((behavior, hp_opt)) = breakers.get_mut(msg.breaker) else {
            continue;
        };
        match *behavior {
            BoltLossBehavior::LifeLoss(n) => {
                if let Some(mut hp) = hp_opt {
                    hp.current = (hp.current - n as f32).max(0.0);
                }
            }
            BoltLossBehavior::TimeLoss(delta) => {
                writer.write(ReduceNodeTimer { delta });
            }
            BoltLossBehavior::None => {}
        }
    }
}
