#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::CString;
use std::sync::OnceLock;

use backend_api::Backend;
use mf_core::View;
use mf_widgets::button::ButtonView;
use mf_widgets::image::ImageView;
use mf_widgets::layout::{Axis, StackElement};
use mf_widgets::text::TextView;
use mf_widgets::{Color, Font, FontWeight};
use objc::rc::StrongPtr;
use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

type CGFloat = f64;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: CGFloat,
    y: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGSize {
    width: CGFloat,
    height: CGFloat,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct UIEdgeInsets {
    top: CGFloat,
    left: CGFloat,
    bottom: CGFloat,
    right: CGFloat,
}

struct UiRoot {
    window: *mut Object,
    host: *mut Object,
}

static ROOT: OnceLock<UiRoot> = OnceLock::new();

/// Native backend that maps the view tree to `UIViews` on iOS.
#[derive(Default)]
pub struct NativeBackend;

impl Backend for NativeBackend {
    fn mount(&mut self, view: &View) {
        with_host(|host| unsafe {
            replace_host_content(host, view);
        });
    }

    fn update(&mut self, view: &View) {
        with_host(|host| unsafe {
            replace_host_content(host, view);
        });
    }
}

fn with_host<F>(f: F)
where
    F: FnOnce(*mut Object),
{
    let root = ROOT.get_or_init(|| unsafe { build_root_window() });
    f(root.host);
}

/// Removes existing children and mounts the freshly built UIKit subtree.
unsafe fn replace_host_content(host: *mut Object, view: &View) {
    clear_subviews(host);
    let built = build_view(view);

    // Size the view to fill its host.
    let bounds: CGRect = msg_send![host, bounds];
    let () = msg_send![built.as_ptr(), setFrame: bounds];
    let () = msg_send![host, addSubview: built.as_ptr()];
    let () = msg_send![built.as_ptr(), setNeedsLayout];
    let () = msg_send![built.as_ptr(), layoutIfNeeded];
}

unsafe fn build_root_window() -> UiRoot {
    let screen: *mut Object = msg_send![class!(UIScreen), mainScreen];
    let bounds: CGRect = msg_send![screen, bounds];

    let window: *mut Object = msg_send![class!(UIWindow), alloc];
    let window: *mut Object = msg_send![window, initWithFrame: bounds];

    let controller: *mut Object = msg_send![class!(UIViewController), alloc];
    let controller: *mut Object = msg_send![controller, init];

    let host: *mut Object = msg_send![class!(UIView), alloc];
    let host: *mut Object = msg_send![host, initWithFrame: bounds];

    let background: *mut Object = msg_send![class!(UIColor), systemBackgroundColor];
    let () = msg_send![host, setBackgroundColor: background];
    let () = msg_send![controller, setView: host];
    let () = msg_send![window, setRootViewController: controller];
    let () = msg_send![window, makeKeyAndVisible];

    UiRoot {
        window: leak(window),
        host: leak(host),
    }
}

/// Leak a UIKit object to keep it alive for the app lifetime.
unsafe fn leak(obj: *mut Object) -> *mut Object {
    let owned = StrongPtr::new(obj);
    let ptr = owned.as_ptr();
    std::mem::forget(owned);
    ptr
}

/// Delete all subviews under a host container.
unsafe fn clear_subviews(view: *mut Object) {
    let subviews: *mut Object = msg_send![view, subviews];
    let count: usize = msg_send![subviews, count];
    for idx in 0..count {
        let sub: *mut Object = msg_send![subviews, objectAtIndex: idx];
        let () = msg_send![sub, removeFromSuperview];
    }
}

/// Recursively converts a Rust view tree into UIKit views.
unsafe fn build_view(view: &View) -> StrongPtr {
    // Text
    if let Some(text) = view.element().as_any().downcast_ref::<TextView>() {
        return build_label(text);
    }

    // Button
    if let Some(button) = view.element().as_any().downcast_ref::<ButtonView>() {
        return build_button(button);
    }

    // Image placeholder
    if let Some(image) = view.element().as_any().downcast_ref::<ImageView>() {
        return build_image(image);
    }

    // Stack layouts (HStack/VStack)
    if let Some(stack) = view
        .element()
        .as_any()
        .downcast_ref::<StackElement>()
    {
        return build_stack(stack, view.children());
    }

    // List -> vertical stack
    if view.element().name() == "List" {
        return build_list(view.children());
    }

    // Fallback container that simply nests children.
    build_container(view.children())
}

unsafe fn build_label(text: &TextView) -> StrongPtr {
    let label: *mut Object = msg_send![class!(UILabel), alloc];
    let label: *mut Object = msg_send![label, init];
    let ns_text = nsstring(text.content());
    let () = msg_send![label, setText: ns_text.as_ptr()];

    if let Some(color) = text.color_value() {
        let ui_color = ui_color(color);
        let () = msg_send![label, setTextColor: ui_color];
    }
    if let Some(font) = text.font_value() {
        let ui_font = ui_font(font);
        let () = msg_send![label, setFont: ui_font];
    }

    StrongPtr::new(label)
}

unsafe fn build_button(button: &ButtonView) -> StrongPtr {
    let btn: *mut Object = msg_send![class!(UIButton), buttonWithType: 0usize];
    let title = nsstring(button.label());
    let () = msg_send![btn, setTitle: title.as_ptr() forState: 0usize];
    StrongPtr::new(btn)
}

unsafe fn build_image(_image: &ImageView) -> StrongPtr {
    // Placeholder colored view; image decoding is left for later.
    let view: *mut Object = msg_send![class!(UIView), alloc];
    let view: *mut Object = msg_send![view, init];
    let placeholder: *mut Object = msg_send![class!(UIColor), tertiarySystemFillColor];
    let () = msg_send![view, setBackgroundColor: placeholder];
    let () = msg_send![view, setClipsToBounds: true];
    StrongPtr::new(view)
}

unsafe fn build_stack(stack: &StackElement, children: &[View]) -> StrongPtr {
    let stack_view: *mut Object = msg_send![class!(UIStackView), alloc];
    let stack_view: *mut Object = msg_send![stack_view, init];

    let axis: isize = match stack.axis() {
        Axis::Horizontal => 0,
        Axis::Vertical => 1,
    };
    let spacing: CGFloat = stack.spacing() as CGFloat;
    let padding: CGFloat = stack.padding() as CGFloat;

    let () = msg_send![stack_view, setAxis: axis];
    let () = msg_send![stack_view, setSpacing: spacing];
    let () = msg_send![stack_view, setLayoutMarginsRelativeArrangement: true];
    let insets = UIEdgeInsets {
        top: padding,
        left: padding,
        bottom: padding,
        right: padding,
    };
    let () = msg_send![stack_view, setLayoutMargins: insets];

    for child in children {
        let subview = build_view(child);
        let () = msg_send![stack_view, addArrangedSubview: subview.as_ptr()];
    }

    StrongPtr::new(stack_view)
}

unsafe fn build_list(children: &[View]) -> StrongPtr {
    let stack_view: *mut Object = msg_send![class!(UIStackView), alloc];
    let stack_view: *mut Object = msg_send![stack_view, init];

    let () = msg_send![stack_view, setAxis: 1isize];
    let () = msg_send![stack_view, setSpacing: 8.0f64];

    for child in children {
        let subview = build_view(child);
        let () = msg_send![stack_view, addArrangedSubview: subview.as_ptr()];
    }

    StrongPtr::new(stack_view)
}

unsafe fn build_container(children: &[View]) -> StrongPtr {
    let view: *mut Object = msg_send![class!(UIView), alloc];
    let view: *mut Object = msg_send![view, init];
    for child in children {
        let subview = build_view(child);
        let () = msg_send![view, addSubview: subview.as_ptr()];
    }
    StrongPtr::new(view)
}

fn nsstring(text: &str) -> StrongPtr {
    let cstring = CString::new(text).unwrap_or_default();
    unsafe {
        let ns: *mut Object = msg_send![class!(NSString), stringWithUTF8String: cstring.as_ptr()];
        StrongPtr::new(ns)
    }
}

unsafe fn ui_color(color: &Color) -> *mut Object {
    msg_send![
        class!(UIColor),
        colorWithRed: color.r as CGFloat
        green: color.g as CGFloat
        blue: color.b as CGFloat
        alpha: color.a as CGFloat
    ]
}

unsafe fn ui_font(font: &Font) -> *mut Object {
    match font.weight {
        FontWeight::Bold | FontWeight::SemiBold => {
            msg_send![class!(UIFont), boldSystemFontOfSize: font.size as CGFloat]
        }
        FontWeight::Regular => msg_send![class!(UIFont), systemFontOfSize: font.size as CGFloat],
    }
}
