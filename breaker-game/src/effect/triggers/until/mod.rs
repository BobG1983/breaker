pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::desugar_until;
