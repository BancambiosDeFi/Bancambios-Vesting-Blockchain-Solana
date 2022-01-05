#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub mod error;
pub mod instruction;
pub mod state;
mod utils;

pub mod processor;
