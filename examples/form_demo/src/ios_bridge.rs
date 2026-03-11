use std::ffi::c_char;
use std::sync::{Mutex, OnceLock};

use backend_native::NativeBackend;
use dev_support::{
    mf_dev_renderer_apply_message as dev_renderer_apply_message,
    mf_dev_renderer_clear_events_json as dev_renderer_clear_events_json,
    mf_dev_renderer_reset as dev_renderer_reset,
    mf_dev_renderer_take_events_json as dev_renderer_take_events_json,
};
use mf_runtime::{App, HostSize};
use native_schema::EdgeInsets;

use crate::create_form_demo_native_app;

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
    let app = create_form_demo_native_app(host_size);
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

#[no_mangle]
pub extern "C" fn mf_dev_renderer_apply_message(json: *const c_char) -> bool {
    dev_renderer_apply_message(json)
}

#[no_mangle]
pub extern "C" fn mf_dev_renderer_take_events_json() -> *const c_char {
    dev_renderer_take_events_json()
}

#[no_mangle]
pub extern "C" fn mf_dev_renderer_clear_events_json() {
    dev_renderer_clear_events_json()
}

#[no_mangle]
pub extern "C" fn mf_dev_renderer_reset() {
    dev_renderer_reset()
}
