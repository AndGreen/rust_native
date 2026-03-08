use backend_api::{debug_layout, debug_mutations, Backend, BackendError};
use native_schema::{LayoutFrame, Mutation, UiEvent};

/// Logging-only backend used on non-iOS targets.
#[derive(Default)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        println!("[native] mutations\n{}", debug_mutations(mutations));
        Ok(())
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        println!("[native] layout\n{}", debug_layout(frames));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        println!("[native] flush");
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}
