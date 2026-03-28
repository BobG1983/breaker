//! Trigger bridge systems — one per trigger type, plus shared evaluation helpers.

/// Shared chain evaluation helpers.
pub mod evaluate;

/// Global — any successful bump.
pub mod bump;
/// Global — bump timing missed.
pub mod bump_whiff;
/// Global — early bump.
pub mod early_bump;
/// Global — late bump.
pub mod late_bump;
/// Global — bolt hit breaker with no bump input.
pub mod no_bump;
/// Global — perfect bump.
pub mod perfect_bump;

/// Targeted on bolt — any successful bump.
pub mod bumped;
/// Targeted on bolt — early bump.
pub mod early_bumped;
/// Targeted on bolt — late bump.
pub mod late_bumped;
/// Targeted on bolt — perfect bump.
pub mod perfect_bumped;

/// Global impact triggers (one system per collision type).
pub mod impact;
/// Targeted impacted triggers on both collision participants.
pub mod impacted;

/// Global — bolt was lost.
pub mod bolt_lost;
/// Global — a cell was destroyed.
pub mod cell_destroyed;
/// Global — something died; cell destroyed.
pub mod death;
/// Targeted — this entity died.
pub mod died;

/// Global — node ended.
pub mod node_end;
/// Global — node started.
pub mod node_start;

/// `TimeExpires` ticker system.
pub mod timer;
/// Until desugaring system.
pub mod until;

/// Register all trigger bridge systems.
pub(crate) fn register(app: &mut bevy::prelude::App) {
    bump::register(app);
    perfect_bump::register(app);
    early_bump::register(app);
    late_bump::register(app);
    bump_whiff::register(app);
    no_bump::register(app);

    bumped::register(app);
    perfect_bumped::register(app);
    early_bumped::register(app);
    late_bumped::register(app);

    impact::register(app);
    impacted::register(app);

    cell_destroyed::register(app);
    death::register(app);
    died::register(app);
    bolt_lost::register(app);

    node_start::register(app);
    node_end::register(app);

    timer::register(app);
    until::register(app);
}
