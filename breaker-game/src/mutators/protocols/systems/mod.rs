//! Protocol domain systems — offering generation + dispatch.

pub(crate) mod dispatch_protocol_selection;
pub(crate) mod gate;
pub(crate) mod generate_protocol_offering;
pub(crate) mod reseed_protocol_rng;

pub(crate) use dispatch_protocol_selection::dispatch_protocol_selection;
pub(crate) use gate::ProtocolGate;
pub(crate) use generate_protocol_offering::generate_protocol_offering;
pub(crate) use reseed_protocol_rng::reseed_protocol_rng;
