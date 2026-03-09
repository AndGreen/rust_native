use std::sync::{Mutex, OnceLock};

use backend_native::NativeBackend;
use mf_runtime::{App, HostSize};
use native_schema::EdgeInsets;

use crate::create_album_list_app;

static APP: OnceLock<Mutex<Option<App<NativeBackend>>>> = OnceLock::new();

fn app_slot() -> &'static Mutex<Option<App<NativeBackend>>> {
    APP.get_or_init(|| Mutex::new(None))
}

#[no_mangle]
pub extern "C" fn mf_app_start(
    width: f32,
    height: f32,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) -> bool {
    let host_size =
        HostSize::with_safe_area(width, height, EdgeInsets::new(top, right, bottom, left));
    let app = create_album_list_app(host_size);
    app.repaint();

    let mut slot = app_slot().lock().unwrap();
    *slot = Some(app);
    true
}

#[no_mangle]
pub extern "C" fn mf_app_tick() {
    let app = {
        let slot = app_slot().lock().unwrap();
        slot.as_ref().cloned()
    };

    if let Some(app) = app {
        app.tick();
    }
}

#[no_mangle]
pub extern "C" fn mf_app_resize(
    width: f32,
    height: f32,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    let app = {
        let slot = app_slot().lock().unwrap();
        slot.as_ref().cloned()
    };

    if let Some(app) = app {
        app.set_host_size(HostSize::with_safe_area(
            width,
            height,
            EdgeInsets::new(top, right, bottom, left),
        ));
    }
}
