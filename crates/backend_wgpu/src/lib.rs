use backend_api::{debug_tree, Backend};
use mf_core::View;

#[derive(Default)]
pub struct GpuBackend;

impl Backend for GpuBackend {
    fn mount(&mut self, view: &View) {
        println!("[wgpu] mount\n{}", debug_tree(view));
    }

    fn update(&mut self, view: &View) {
        println!("[wgpu] update\n{}", debug_tree(view));
    }
}
