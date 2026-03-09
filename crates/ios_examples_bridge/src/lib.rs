use std::sync::{Mutex, OnceLock};

use album_list::create_album_list_app;
use backend_native::NativeBackend;
use counter::create_counter_app;
use mf_runtime::{App, HostSize};

const EXAMPLE_COUNTER: u32 = 1;
const EXAMPLE_ALBUM_LIST: u32 = 2;

static APP: OnceLock<Mutex<Option<App<NativeBackend>>>> = OnceLock::new();

fn app_slot() -> &'static Mutex<Option<App<NativeBackend>>> {
    APP.get_or_init(|| Mutex::new(None))
}

fn build_app(example_id: u32, host_size: HostSize) -> Option<App<NativeBackend>> {
    match example_id {
        EXAMPLE_COUNTER => Some(create_counter_app(host_size)),
        EXAMPLE_ALBUM_LIST => Some(create_album_list_app(host_size)),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn mf_examples_start(example_id: u32, width: f32, height: f32) -> bool {
    let Some(app) = build_app(example_id, HostSize::new(width, height)) else {
        eprintln!("[ios_examples_bridge] unknown example id: {example_id}");
        return false;
    };

    app.repaint();

    let mut slot = app_slot().lock().unwrap();
    *slot = Some(app);
    true
}

#[no_mangle]
pub extern "C" fn mf_examples_tick() {
    let app = {
        let slot = app_slot().lock().unwrap();
        slot.as_ref().cloned()
    };

    if let Some(app) = app {
        app.tick();
    }
}
