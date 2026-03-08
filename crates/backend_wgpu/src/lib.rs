use backend_api::{debug_layout, debug_mutations, Backend, BackendError};
use native_schema::{LayoutFrame, Mutation, UiEvent};

#[derive(Default)]
pub struct GpuBackend;

impl Backend for GpuBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        println!("[wgpu] mutations\n{}", debug_mutations(mutations));
        Ok(())
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        println!("[wgpu] layout\n{}", debug_layout(frames));
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        println!("[wgpu] flush");
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}
