use std::collections::HashMap;
use std::slice;
use std::sync::{Mutex, OnceLock};

use native_schema::{UiEvent, UiNodeId};

#[derive(Clone, Copy)]
pub(super) struct ControlBinding {
    node_id: UiNodeId,
    pub(super) tap: bool,
    pub(super) text_input: bool,
    pub(super) focus_changed: bool,
    pub(super) appear: bool,
    pub(super) disappear: bool,
}

fn binding_store() -> &'static Mutex<HashMap<usize, ControlBinding>> {
    static STORE: OnceLock<Mutex<HashMap<usize, ControlBinding>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn event_queue() -> &'static Mutex<Vec<UiEvent>> {
    static QUEUE: OnceLock<Mutex<Vec<UiEvent>>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn update_binding<R>(
    handle: usize,
    node_id: UiNodeId,
    f: impl FnOnce(&mut ControlBinding) -> R,
) -> R {
    let mut store = binding_store().lock().unwrap();
    let entry = store.entry(handle).or_insert(ControlBinding {
        node_id,
        tap: false,
        text_input: false,
        focus_changed: false,
        appear: false,
        disappear: false,
    });
    entry.node_id = node_id;
    f(entry)
}

pub(super) fn unregister_binding(handle: usize) {
    binding_store().lock().unwrap().remove(&handle);
}

pub(super) fn emit_appear_if_needed(node_id: UiNodeId, handle: usize) {
    let store = binding_store().lock().unwrap();
    if let Some(binding) = store.get(&handle) {
        if binding.appear && binding.node_id == node_id {
            drop(store);
            queue_event(UiEvent::Appear { id: node_id });
        }
    }
}

pub(super) fn queue_event(event: UiEvent) {
    event_queue().lock().unwrap().push(event);
}

pub(super) fn take_events() -> Vec<UiEvent> {
    std::mem::take(&mut *event_queue().lock().unwrap())
}

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
