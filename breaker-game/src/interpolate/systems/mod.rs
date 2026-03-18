//! Interpolation systems — one file per system function.

mod interpolate_transform;
mod restore_authoritative;
mod store_authoritative;

pub(crate) use interpolate_transform::interpolate_transform;
pub(crate) use restore_authoritative::restore_authoritative;
pub(crate) use store_authoritative::store_authoritative;
