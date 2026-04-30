use crate::breaker::components::BumpState;

pub(super) fn perfect_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.1,
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}
