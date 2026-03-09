mod adapter;
mod mutations;
mod state;

#[cfg(test)]
mod tests;

pub use adapter::PlatformAdapter;
pub use state::{ExecutorState, NodeRecord};
