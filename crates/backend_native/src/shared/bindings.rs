use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use native_schema::{UiEvent, UiNodeId};

#[derive(Clone, Copy)]
pub(crate) struct ControlBinding {
    pub(crate) node_id: UiNodeId,
    pub(crate) tap: bool,
    pub(crate) text_input: bool,
    pub(crate) focus_changed: bool,
    pub(crate) appear: bool,
    pub(crate) disappear: bool,
}

fn binding_store() -> &'static Mutex<HashMap<usize, ControlBinding>> {
    static STORE: OnceLock<Mutex<HashMap<usize, ControlBinding>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn event_queue() -> &'static Mutex<Vec<UiEvent>> {
    static QUEUE: OnceLock<Mutex<Vec<UiEvent>>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn binding(handle: usize) -> Option<ControlBinding> {
    binding_store().lock().unwrap().get(&handle).copied()
}

pub(crate) fn update_binding<R>(
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

pub(crate) fn unregister_binding(handle: usize) {
    binding_store().lock().unwrap().remove(&handle);
}

pub(crate) fn emit_appear_if_needed(node_id: UiNodeId, handle: usize) {
    if let Some(binding) = binding(handle) {
        if binding.appear && binding.node_id == node_id {
            queue_event(UiEvent::Appear { id: node_id });
        }
    }
}

pub(crate) fn queue_event(event: UiEvent) {
    event_queue().lock().unwrap().push(event);
}

pub(crate) fn take_events() -> Vec<UiEvent> {
    std::mem::take(&mut *event_queue().lock().unwrap())
}
