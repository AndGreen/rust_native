use std::ffi::{c_char, CStr, CString};
use std::sync::{Mutex, OnceLock};

use backend_api::Backend;
use backend_native::NativeBackend;
use dev_protocol::ServerMessage;

#[derive(Default)]
struct DevRenderer {
    backend: NativeBackend,
}

static DEV_RENDERER: OnceLock<Mutex<DevRenderer>> = OnceLock::new();
static DEV_RENDERER_EVENTS: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

fn renderer_slot() -> &'static Mutex<DevRenderer> {
    DEV_RENDERER.get_or_init(|| Mutex::new(DevRenderer::default()))
}

fn renderer_events_slot() -> &'static Mutex<Option<CString>> {
    DEV_RENDERER_EVENTS.get_or_init(|| Mutex::new(None))
}

pub fn mf_dev_renderer_reset() {
    let mut renderer = renderer_slot().lock().unwrap();
    *renderer = DevRenderer::default();
}

pub fn mf_dev_renderer_apply_server_message(message: &ServerMessage) -> bool {
    match message {
        ServerMessage::ResetUi => {
            mf_dev_renderer_reset();
            true
        }
        ServerMessage::RenderBatch {
            mutations, layout, ..
        } => {
            let mut renderer = renderer_slot().lock().unwrap();
            renderer
                .backend
                .apply_mutations(mutations)
                .and_then(|_| renderer.backend.apply_layout(layout))
                .and_then(|_| renderer.backend.flush())
                .is_ok()
        }
        ServerMessage::Reloading
        | ServerMessage::BuildFailed { .. }
        | ServerMessage::HelloAck { .. } => true,
    }
}

/// # Safety
///
/// `json` must be either null or point to a valid, NUL-terminated UTF-8 string
/// for the duration of this call.
pub unsafe extern "C" fn mf_dev_renderer_apply_message(json: *const c_char) -> bool {
    if json.is_null() {
        return false;
    }

    let payload = unsafe { CStr::from_ptr(json) };
    let Ok(payload) = payload.to_str() else {
        return false;
    };
    let Ok(message) = serde_json::from_str::<ServerMessage>(payload) else {
        return false;
    };

    mf_dev_renderer_apply_server_message(&message)
}

pub fn mf_dev_renderer_take_events_json() -> *const c_char {
    let mut renderer = renderer_slot().lock().unwrap();
    let events = renderer.backend.drain_events();
    let serialized = match serde_json::to_string(&events) {
        Ok(value) => value,
        Err(_) => "[]".to_string(),
    };
    let cstring = CString::new(serialized).unwrap_or_else(|_| CString::new("[]").unwrap());
    let ptr = cstring.as_ptr();
    *renderer_events_slot().lock().unwrap() = Some(cstring);
    ptr
}

pub fn mf_dev_renderer_clear_events_json() {
    *renderer_events_slot().lock().unwrap() = None;
}
