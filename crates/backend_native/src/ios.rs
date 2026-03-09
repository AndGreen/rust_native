#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unexpected_cfgs)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};

use backend_api::{Backend, BackendError};
use native_schema::{
    ColorValue, ElementKind, EventKind, FontWeight, LayoutFrame, Mutation, PropKey, PropValue,
    UiEvent, UiNodeId,
};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, NO, YES};
use objc::{class, msg_send, sel, sel_impl};

use crate::executor::{ExecutorState, PlatformAdapter};

pub struct NativeBackend {
    state: ExecutorState<*mut Object>,
    adapter: IosAdapter,
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self {
            state: ExecutorState::default(),
            adapter: IosAdapter::default(),
        }
    }
}

// Objective-C handles are raw pointers but all UI work is guarded to the main thread.
unsafe impl Send for NativeBackend {}

impl Backend for NativeBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.apply_mutations(&mut self.adapter, mutations)
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.apply_layout(frames)
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.flush(&mut self.adapter)
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        self.state.drain_events(&mut self.adapter)
    }
}

#[derive(Default)]
struct IosAdapter {
    host: Option<HostViews>,
}

struct HostViews {
    _window: *mut Object,
    _controller: *mut Object,
    host_view: *mut Object,
}

impl PlatformAdapter for IosAdapter {
    type Handle = *mut Object;

    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<Self::Handle, BackendError> {
        self.ensure_host()?;

        let view = match kind {
            ElementKind::Stack | ElementKind::List => create_plain_view(),
            ElementKind::Text => {
                let label = create_label();
                if let Some(text) = text {
                    set_text_on_view(ElementKind::Text, label, text)?;
                }
                label
            }
            ElementKind::Button => {
                let button = create_button();
                if let Some(text) = text {
                    set_text_on_view(ElementKind::Button, button, text)?;
                }
                button
            }
            ElementKind::Image => create_image_view(),
            ElementKind::Input => {
                let input = create_text_field();
                if let Some(text) = text {
                    set_text_on_view(ElementKind::Input, input, text)?;
                }
                input
            }
        };

        Ok(view)
    }

    fn attach_root(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError> {
        let host = self.ensure_host()?;
        add_subview(host.host_view, handle);
        emit_appear_if_needed(node_id, handle);
        Ok(())
    }

    fn detach_root(
        &mut self,
        _node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError> {
        remove_from_superview(handle);
        Ok(())
    }

    fn insert_child(
        &mut self,
        parent: Self::Handle,
        child_id: UiNodeId,
        child: Self::Handle,
        index: usize,
    ) -> Result<(), BackendError> {
        insert_subview(parent, child, index);
        emit_appear_if_needed(child_id, child);
        Ok(())
    }

    fn remove_child(
        &mut self,
        _parent: Self::Handle,
        child: Self::Handle,
    ) -> Result<(), BackendError> {
        remove_from_superview(child);
        Ok(())
    }

    fn remove_view(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
        listeners: &[EventKind],
    ) -> Result<(), BackendError> {
        if listeners.contains(&EventKind::Disappear) {
            queue_event(UiEvent::Disappear { id: node_id });
        }
        unregister_binding(handle);
        remove_from_superview(handle);
        release_object(handle);
        Ok(())
    }

    fn set_text(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        text: &str,
    ) -> Result<(), BackendError> {
        set_text_on_view(kind, handle, text)
    }

    fn set_prop(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        props: &HashMap<PropKey, PropValue>,
        key: PropKey,
    ) -> Result<(), BackendError> {
        match key {
            PropKey::Color => apply_color(kind, handle, props.get(&PropKey::Color)),
            PropKey::FontSize | PropKey::FontWeight => apply_font(kind, handle, props),
            PropKey::CornerRadius => {
                apply_corner_radius(handle, props.get(&PropKey::CornerRadius));
                Ok(())
            }
            PropKey::Enabled => {
                apply_enabled(handle, props.get(&PropKey::Enabled));
                Ok(())
            }
            PropKey::Source => apply_image_source(handle, props.get(&PropKey::Source)),
            PropKey::Axis
            | PropKey::Spacing
            | PropKey::Padding
            | PropKey::Alignment
            | PropKey::Width
            | PropKey::Height
            | PropKey::MinWidth
            | PropKey::MinHeight
            | PropKey::MaxWidth
            | PropKey::MaxHeight
            | PropKey::FlexGrow
            | PropKey::FlexShrink => {
                eprintln!("[backend_native/ios] ignoring layout prop {key:?}; layout is driven by Rust");
                Ok(())
            }
        }
    }

    fn attach_listener(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        node_id: UiNodeId,
        event: EventKind,
    ) -> Result<(), BackendError> {
        match event {
            EventKind::Tap => {
                if kind != ElementKind::Button {
                    eprintln!("[backend_native/ios] ignoring Tap listener for {kind:?}");
                    return Ok(());
                }
                let should_wire = update_binding(handle, node_id, |binding| {
                    let should_wire = !binding.tap;
                    binding.tap = true;
                    should_wire
                });
                if should_wire {
                    let target = shared_event_target();
                    unsafe {
                        let _: () = msg_send![
                            handle,
                            addTarget: target
                            action: sel!(handleTap:)
                            forControlEvents: 1u64 << 6
                        ];
                    }
                }
            }
            EventKind::TextInput => {
                if kind != ElementKind::Input {
                    eprintln!("[backend_native/ios] ignoring TextInput listener for {kind:?}");
                    return Ok(());
                }
                let should_wire = update_binding(handle, node_id, |binding| {
                    let should_wire = !binding.text_input;
                    binding.text_input = true;
                    should_wire
                });
                if should_wire {
                    let target = shared_event_target();
                    unsafe {
                        let _: () = msg_send![
                            handle,
                            addTarget: target
                            action: sel!(handleEditingChanged:)
                            forControlEvents: 1u64 << 17
                        ];
                    }
                }
            }
            EventKind::Appear => {
                update_binding(handle, node_id, |binding| binding.appear = true);
            }
            EventKind::Disappear => {
                update_binding(handle, node_id, |binding| binding.disappear = true);
            }
            EventKind::Scroll => {
                eprintln!("[backend_native/ios] ignoring Scroll listener in P0-06");
            }
        }
        Ok(())
    }

    fn apply_frame(
        &mut self,
        handle: Self::Handle,
        frame: LayoutFrame,
    ) -> Result<(), BackendError> {
        let rect = CGRect::new(frame.x as f64, frame.y as f64, frame.width as f64, frame.height as f64);
        unsafe {
            let _: () = msg_send![handle, setFrame: rect];
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        take_events()
    }
}

impl IosAdapter {
    fn ensure_host(&mut self) -> Result<&HostViews, BackendError> {
        if self.host.is_none() {
            self.host = Some(bootstrap_host()?);
        }
        Ok(self.host.as_ref().expect("host is initialized"))
    }
}

fn ensure_main_thread() -> Result<(), BackendError> {
    let is_main: BOOL = unsafe { msg_send![class!(NSThread), isMainThread] };
    if is_main == YES {
        Ok(())
    } else {
        Err(BackendError::BatchRejected(
            "ios backend requires main thread".to_string(),
        ))
    }
}

fn bootstrap_host() -> Result<HostViews, BackendError> {
    let screen: *mut Object = unsafe { msg_send![class!(UIScreen), mainScreen] };
    let bounds: CGRect = unsafe { msg_send![screen, bounds] };

    let window: *mut Object = unsafe {
        let window: *mut Object = msg_send![class!(UIWindow), alloc];
        msg_send![window, initWithFrame: bounds]
    };
    let controller: *mut Object = unsafe { msg_send![class!(UIViewController), new] };
    let host_view = create_plain_view();
    unsafe {
        let white: *mut Object = msg_send![class!(UIColor), whiteColor];
        let _: () = msg_send![host_view, setFrame: bounds];
        let _: () = msg_send![host_view, setBackgroundColor: white];
        let _: () = msg_send![controller, setView: host_view];
        let _: () = msg_send![window, setRootViewController: controller];
        let _: () = msg_send![window, makeKeyAndVisible];
    }

    Ok(HostViews {
        _window: window,
        _controller: controller,
        host_view,
    })
}

fn create_plain_view() -> *mut Object {
    unsafe {
        let view: *mut Object = msg_send![class!(UIView), new];
        let _: () = msg_send![view, setClipsToBounds: YES];
        view
    }
}

fn create_label() -> *mut Object {
    unsafe {
        let label: *mut Object = msg_send![class!(UILabel), new];
        let clear: *mut Object = msg_send![class!(UIColor), clearColor];
        let _: () = msg_send![label, setBackgroundColor: clear];
        label
    }
}

fn create_button() -> *mut Object {
    unsafe {
        let button: *mut Object = msg_send![class!(UIButton), buttonWithType: 0usize];
        retain_object(button)
    }
}

fn create_image_view() -> *mut Object {
    unsafe {
        let view: *mut Object = msg_send![class!(UIImageView), new];
        let _: () = msg_send![view, setClipsToBounds: YES];
        view
    }
}

fn create_text_field() -> *mut Object {
    unsafe { msg_send![class!(UITextField), new] }
}

fn set_text_on_view(
    kind: ElementKind,
    handle: *mut Object,
    text: &str,
) -> Result<(), BackendError> {
    let value = nsstring(text)?;
    unsafe {
        match kind {
            ElementKind::Text => {
                let _: () = msg_send![handle, setText: value];
            }
            ElementKind::Button => {
                let _: () = msg_send![handle, setTitle: value forState: 0usize];
            }
            ElementKind::Input => {
                let _: () = msg_send![handle, setText: value];
            }
            other => {
                return Err(BackendError::BatchRejected(format!(
                    "set_text is unsupported for {other:?}"
                )));
            }
        }
    }
    Ok(())
}

fn apply_color(
    kind: ElementKind,
    handle: *mut Object,
    value: Option<&PropValue>,
) -> Result<(), BackendError> {
    let Some(PropValue::Color(ColorValue { r, g, b, a })) = value else {
        eprintln!("[backend_native/ios] ignoring invalid Color prop");
        return Ok(());
    };
    unsafe {
        let color: *mut Object = msg_send![
            class!(UIColor),
            colorWithRed: *r as f64
            green: *g as f64
            blue: *b as f64
            alpha: *a as f64
        ];
        match kind {
            ElementKind::Text => {
                let _: () = msg_send![handle, setTextColor: color];
            }
            ElementKind::Button => {
                let _: () = msg_send![handle, setTitleColor: color forState: 0usize];
            }
            ElementKind::Input => {
                let _: () = msg_send![handle, setTextColor: color];
            }
            _ => {
                let _: () = msg_send![handle, setBackgroundColor: color];
            }
        }
    }
    Ok(())
}

fn apply_font(
    kind: ElementKind,
    handle: *mut Object,
    props: &HashMap<PropKey, PropValue>,
) -> Result<(), BackendError> {
    if !matches!(kind, ElementKind::Text | ElementKind::Button | ElementKind::Input) {
        eprintln!("[backend_native/ios] ignoring font prop for {kind:?}");
        return Ok(());
    }

    let size = match props.get(&PropKey::FontSize) {
        Some(PropValue::Float(size)) => *size as f64,
        Some(_) => {
            eprintln!("[backend_native/ios] invalid FontSize prop");
            return Ok(());
        }
        None => 17.0,
    };
    let weight = match props.get(&PropKey::FontWeight) {
        Some(PropValue::FontWeight(weight)) => *weight,
        Some(_) => {
            eprintln!("[backend_native/ios] invalid FontWeight prop");
            return Ok(());
        }
        None => FontWeight::Regular,
    };

    unsafe {
        let font: *mut Object = match weight {
            FontWeight::Regular => msg_send![class!(UIFont), systemFontOfSize: size],
            FontWeight::SemiBold | FontWeight::Bold => {
                msg_send![class!(UIFont), boldSystemFontOfSize: size]
            }
        };
        let _: () = msg_send![handle, setFont: font];
    }
    Ok(())
}

fn apply_corner_radius(handle: *mut Object, value: Option<&PropValue>) {
    let Some(PropValue::Float(radius)) = value else {
        eprintln!("[backend_native/ios] ignoring invalid CornerRadius prop");
        return;
    };
    unsafe {
        let layer: *mut Object = msg_send![handle, layer];
        let _: () = msg_send![layer, setCornerRadius: *radius as f64];
        let _: () = msg_send![handle, setClipsToBounds: if *radius > 0.0 { YES } else { NO }];
    }
}

fn apply_enabled(handle: *mut Object, value: Option<&PropValue>) {
    let Some(PropValue::Bool(enabled)) = value else {
        eprintln!("[backend_native/ios] ignoring invalid Enabled prop");
        return;
    };
    unsafe {
        if responds_to(handle, sel!(setEnabled:)) {
            let _: () = msg_send![handle, setEnabled: if *enabled { YES } else { NO }];
        } else {
            let _: () = msg_send![handle, setUserInteractionEnabled: if *enabled { YES } else { NO }];
        }
    }
}

fn apply_image_source(
    handle: *mut Object,
    value: Option<&PropValue>,
) -> Result<(), BackendError> {
    let Some(PropValue::String(source)) = value else {
        eprintln!("[backend_native/ios] ignoring invalid Source prop");
        return Ok(());
    };
    let source = nsstring(source)?;
    unsafe {
        let image: *mut Object = msg_send![class!(UIImage), imageNamed: source];
        let _: () = msg_send![handle, setImage: image];
    }
    Ok(())
}

fn add_subview(parent: *mut Object, child: *mut Object) {
    unsafe {
        let _: () = msg_send![parent, addSubview: child];
    }
}

fn insert_subview(parent: *mut Object, child: *mut Object, index: usize) {
    unsafe {
        let subviews: *mut Object = msg_send![parent, subviews];
        let count: usize = msg_send![subviews, count];
        if index >= count {
            let _: () = msg_send![parent, addSubview: child];
        } else {
            let _: () = msg_send![parent, insertSubview: child atIndex: index];
        }
    }
}

fn remove_from_superview(view: *mut Object) {
    unsafe {
        let _: () = msg_send![view, removeFromSuperview];
    }
}

fn retain_object(object: *mut Object) -> *mut Object {
    unsafe {
        let _: *mut Object = msg_send![object, retain];
    }
    object
}

fn release_object(object: *mut Object) {
    unsafe {
        let _: () = msg_send![object, release];
    }
}

fn responds_to(object: *mut Object, selector: Sel) -> bool {
    let result: BOOL = unsafe { msg_send![object, respondsToSelector: selector] };
    result == YES
}

fn nsstring(value: &str) -> Result<*mut Object, BackendError> {
    let sanitized = value.replace('\0', "");
    let cstring = CString::new(sanitized).map_err(|err| {
        BackendError::BatchRejected(format!("failed to build NSString payload: {err}"))
    })?;
    let string = unsafe { msg_send![class!(NSString), stringWithUTF8String: cstring.as_ptr()] };
    Ok(string)
}

#[derive(Clone, Copy)]
struct ControlBinding {
    node_id: UiNodeId,
    tap: bool,
    text_input: bool,
    appear: bool,
    disappear: bool,
}

fn binding_store() -> &'static Mutex<HashMap<usize, ControlBinding>> {
    static STORE: OnceLock<Mutex<HashMap<usize, ControlBinding>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn event_queue() -> &'static Mutex<Vec<UiEvent>> {
    static QUEUE: OnceLock<Mutex<Vec<UiEvent>>> = OnceLock::new();
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

fn update_binding<R>(
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

fn unregister_binding(handle: *mut Object) {
    binding_store().lock().unwrap().remove(&(handle as usize));
}

fn emit_appear_if_needed(node_id: UiNodeId, handle: *mut Object) {
    let store = binding_store().lock().unwrap();
    if let Some(binding) = store.get(&(handle as usize)) {
        if binding.appear && binding.node_id == node_id {
            drop(store);
            queue_event(UiEvent::Appear { id: node_id });
        }
    }
}

fn queue_event(event: UiEvent) {
    event_queue().lock().unwrap().push(event);
}

fn take_events() -> Vec<UiEvent> {
    std::mem::take(&mut *event_queue().lock().unwrap())
}

fn shared_event_target() -> *mut Object {
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

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

impl CGRect {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: CGPoint { x, y },
            size: CGSize { width, height },
        }
    }
}
