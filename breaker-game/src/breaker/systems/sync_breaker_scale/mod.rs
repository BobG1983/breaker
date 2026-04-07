pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::sync_breaker_scale;
