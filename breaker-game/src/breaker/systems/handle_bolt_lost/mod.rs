//! `handle_bolt_lost` — applies `BoltLossBehavior` per `BoltLost` message.

mod system;

pub(crate) use system::handle_bolt_lost;

#[cfg(test)]
mod tests;
