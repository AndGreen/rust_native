use std::collections::HashMap;
use std::ffi::CString;

use backend_api::BackendError;
use native_schema::{ColorValue, ElementKind, FontWeight, PropKey, PropValue};
use objc::runtime::{Object, Sel, BOOL, NO, YES};
use objc::{class, msg_send, sel, sel_impl};

pub(super) struct HostViews {
    pub(super) _window: *mut Object,
    pub(super) _controller: *mut Object,
    pub(super) host_view: *mut Object,
}

const UIViewAutoresizingFlexibleWidth: usize = 1 << 1;
const UIViewAutoresizingFlexibleHeight: usize = 1 << 4;
const FULLSCREEN_AUTOREZISING_MASK: usize =
    UIViewAutoresizingFlexibleWidth | UIViewAutoresizingFlexibleHeight;

pub(super) fn bootstrap_host() -> Result<HostViews, BackendError> {
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
        let _: () = msg_send![host_view, setAutoresizingMask: FULLSCREEN_AUTOREZISING_MASK];
        let _: () = msg_send![host_view, setBackgroundColor: white];
        let _: () = msg_send![controller, setView: host_view];
        let view: *mut Object = msg_send![controller, view];
        let _: () = msg_send![view, setFrame: bounds];
        let _: () = msg_send![view, setAutoresizingMask: FULLSCREEN_AUTOREZISING_MASK];
        let _: () = msg_send![window, setFrame: bounds];
        let _: () = msg_send![window, setRootViewController: controller];
        let _: () = msg_send![window, makeKeyAndVisible];
    }

    Ok(HostViews {
        _window: window,
        _controller: controller,
        host_view,
    })
}

pub(super) fn create_plain_view() -> *mut Object {
    unsafe {
        let view: *mut Object = msg_send![class!(UIView), new];
        let _: () = msg_send![view, setClipsToBounds: YES];
        view
    }
}

pub(super) fn create_label() -> *mut Object {
    unsafe {
        let label: *mut Object = msg_send![class!(UILabel), new];
        let clear: *mut Object = msg_send![class!(UIColor), clearColor];
        let _: () = msg_send![label, setBackgroundColor: clear];
        label
    }
}

pub(super) fn create_button() -> *mut Object {
    const UI_BUTTON_TYPE_SYSTEM: usize = 1;

    unsafe {
        // System buttons keep the default UIKit title rendering and contrast on iOS.
        let button: *mut Object =
            msg_send![class!(UIButton), buttonWithType: UI_BUTTON_TYPE_SYSTEM];
        retain_object(button)
    }
}

pub(super) fn create_image_view() -> *mut Object {
    unsafe {
        let view: *mut Object = msg_send![class!(UIImageView), new];
        let _: () = msg_send![view, setClipsToBounds: YES];
        view
    }
}

pub(super) fn create_text_field() -> *mut Object {
    unsafe { msg_send![class!(UITextField), new] }
}

pub(super) fn set_text_on_view(
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

pub(super) fn apply_color(
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

pub(super) fn apply_background_color(
    handle: *mut Object,
    value: Option<&PropValue>,
) -> Result<(), BackendError> {
    let Some(PropValue::Color(ColorValue { r, g, b, a })) = value else {
        eprintln!("[backend_native/ios] ignoring invalid BackgroundColor prop");
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
        let _: () = msg_send![handle, setBackgroundColor: color];
    }
    Ok(())
}

pub(super) fn apply_font(
    kind: ElementKind,
    handle: *mut Object,
    props: &HashMap<PropKey, PropValue>,
) -> Result<(), BackendError> {
    if !matches!(
        kind,
        ElementKind::Text | ElementKind::Button | ElementKind::Input
    ) {
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

pub(super) fn apply_corner_radius(handle: *mut Object, value: Option<&PropValue>) {
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

pub(super) fn apply_enabled(handle: *mut Object, value: Option<&PropValue>) {
    let Some(PropValue::Bool(enabled)) = value else {
        eprintln!("[backend_native/ios] ignoring invalid Enabled prop");
        return;
    };
    unsafe {
        if responds_to(handle, sel!(setEnabled:)) {
            let _: () = msg_send![handle, setEnabled: if *enabled { YES } else { NO }];
        } else {
            let _: () = msg_send![
                handle,
                setUserInteractionEnabled: if *enabled { YES } else { NO }
            ];
        }
    }
}

pub(super) fn apply_focus(
    handle: *mut Object,
    value: Option<&PropValue>,
) -> Result<(), BackendError> {
    let Some(PropValue::Bool(focused)) = value else {
        eprintln!("[backend_native/ios] ignoring invalid Focused prop");
        return Ok(());
    };

    unsafe {
        if *focused {
            let _: BOOL = msg_send![handle, becomeFirstResponder];
        } else {
            let _: BOOL = msg_send![handle, resignFirstResponder];
        }
    }

    Ok(())
}

pub(super) fn apply_image_source(
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

pub(super) fn add_subview(parent: *mut Object, child: *mut Object) {
    unsafe {
        let _: () = msg_send![parent, addSubview: child];
    }
}

pub(super) fn insert_subview(parent: *mut Object, child: *mut Object, index: usize) {
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

pub(super) fn remove_from_superview(view: *mut Object) {
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

pub(super) fn release_object(object: *mut Object) {
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

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGPoint {
    pub(super) x: f64,
    pub(super) y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGSize {
    pub(super) width: f64,
    pub(super) height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct CGRect {
    pub(super) origin: CGPoint,
    pub(super) size: CGSize,
}

impl CGRect {
    pub(super) fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: CGPoint { x, y },
            size: CGSize { width, height },
        }
    }
}
