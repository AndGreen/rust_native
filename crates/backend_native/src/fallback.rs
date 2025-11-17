use backend_api::{debug_tree, Backend};
use mf_core::View;

/// Logging-only backend used on non-iOS targets.
#[derive(Default)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn mount(&mut self, view: &View) {
        println!("[native] mount\n{}", debug_tree(view));
    }

    fn update(&mut self, view: &View) {
        println!("[native] update\n{}", debug_tree(view));
    }
}
