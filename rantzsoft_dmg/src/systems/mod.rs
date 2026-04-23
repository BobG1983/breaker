//! Internal systems for the damage pipeline. P4 ships a stub
//! `process_despawn_requests`; the real body arrives in P6.

mod despawn;

pub(crate) use despawn::process_despawn_requests;
