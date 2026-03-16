use std::slice;

use native_schema::{UiEvent, UiNodeId};

use crate::shared::bindings::queue_event;

#[no_mangle]
pub extern "C" fn rust_native_android_queue_tap(node_id: UiNodeId) {
    queue_event(UiEvent::Tap { id: node_id });
}

#[no_mangle]
pub unsafe extern "C" fn rust_native_android_queue_text_input(
    node_id: UiNodeId,
    text_ptr: *const u8,
    text_len: usize,
) {
    let value = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { slice::from_raw_parts(text_ptr, text_len) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    queue_event(UiEvent::TextInput { id: node_id, value });
}

#[no_mangle]
pub extern "C" fn rust_native_android_queue_focus_changed(node_id: UiNodeId, focused: bool) {
    queue_event(UiEvent::FocusChanged {
        id: node_id,
        focused,
    });
}
