use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Mutex, OnceLock};

use native_schema::{UiEvent, UiNodeId};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

#[derive(Clone, Copy)]
pub(super) struct ControlBinding {
    node_id: UiNodeId,
    pub(super) tap: bool,
    pub(super) text_input: bool,
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
    handle: *mut Object,
    node_id: UiNodeId,
    f: impl FnOnce(&mut ControlBinding) -> R,
) -> R {
    let mut store = binding_store().lock().unwrap();
    let entry = store.entry(handle as usize).or_insert(ControlBinding {
        node_id,
        tap: false,
        text_input: false,
        appear: false,
        disappear: false,
    });
    entry.node_id = node_id;
    f(entry)
}

pub(super) fn unregister_binding(handle: *mut Object) {
    binding_store().lock().unwrap().remove(&(handle as usize));
}

pub(super) fn emit_appear_if_needed(node_id: UiNodeId, handle: *mut Object) {
    let store = binding_store().lock().unwrap();
    if let Some(binding) = store.get(&(handle as usize)) {
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

pub(super) fn shared_event_target() -> *mut Object {
    static TARGET: OnceLock<usize> = OnceLock::new();
    let ptr = TARGET.get_or_init(|| {
        let class = event_target_class();
        let target: *mut Object = unsafe { msg_send![class, new] };
        target as usize
    });
    *ptr as *mut Object
}

fn event_target_class() -> &'static Class {
    static CLASS: OnceLock<&'static Class> = OnceLock::new();
    CLASS.get_or_init(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("RustNativeEventTarget", superclass)
            .expect("RustNativeEventTarget is not declared");
        unsafe {
            decl.add_method(
                sel!(handleTap:),
                handle_tap as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(handleEditingChanged:),
                handle_editing_changed as extern "C" fn(&Object, Sel, *mut Object),
            );
        }
        decl.register()
    })
}

extern "C" fn handle_tap(_this: &Object, _cmd: Sel, sender: *mut Object) {
    if let Some(binding) = binding_store().lock().unwrap().get(&(sender as usize)).copied() {
        if binding.tap {
            queue_event(UiEvent::Tap { id: binding.node_id });
        }
    }
}

extern "C" fn handle_editing_changed(_this: &Object, _cmd: Sel, sender: *mut Object) {
    let binding = binding_store()
        .lock()
        .unwrap()
        .get(&(sender as usize))
        .copied();
    let Some(binding) = binding else {
        return;
    };
    if !binding.text_input {
        return;
    }

    let text: *mut Object = unsafe { msg_send![sender, text] };
    let utf8: *const i8 = unsafe { msg_send![text, UTF8String] };
    if utf8.is_null() {
        queue_event(UiEvent::TextInput {
            id: binding.node_id,
            value: String::new(),
        });
        return;
    }
    let value = unsafe { CStr::from_ptr(utf8) }
        .to_string_lossy()
        .into_owned();
    queue_event(UiEvent::TextInput {
        id: binding.node_id,
        value,
    });
}
