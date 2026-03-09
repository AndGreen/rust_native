#![cfg_attr(test, allow(dead_code))]

use backend_api::BackendError;
use native_schema::{ColorValue, ElementKind, EventKind, FontWeight, LayoutFrame, UiNodeId};

use super::adapter::AndroidBridge;

pub(super) struct AndroidJniBridge;

impl AndroidBridge for AndroidJniBridge {
    fn is_ui_thread(&self) -> bool {
        unsafe { rust_native_android_is_ui_thread() }
    }

    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<usize, BackendError> {
        let (ptr, len) = text_payload(text);
        let handle = unsafe { rust_native_android_create_view(element_kind_code(kind), ptr, len) };
        if handle == 0 {
            Err(BackendError::BatchRejected(format!(
                "android bridge failed to create view for {kind:?}"
            )))
        } else {
            Ok(handle)
        }
    }

    fn attach_root(&mut self, node_id: UiNodeId, handle: usize) -> Result<(), BackendError> {
        unsafe { rust_native_android_attach_root(node_id, handle) };
        Ok(())
    }

    fn detach_root(&mut self, node_id: UiNodeId, handle: usize) -> Result<(), BackendError> {
        unsafe { rust_native_android_detach_root(node_id, handle) };
        Ok(())
    }

    fn insert_child(
        &mut self,
        parent: usize,
        child_id: UiNodeId,
        child: usize,
        index: usize,
    ) -> Result<(), BackendError> {
        unsafe { rust_native_android_insert_child(parent, child_id, child, index) };
        Ok(())
    }

    fn remove_child(&mut self, parent: usize, child: usize) -> Result<(), BackendError> {
        unsafe { rust_native_android_remove_child(parent, child) };
        Ok(())
    }

    fn remove_view(
        &mut self,
        node_id: UiNodeId,
        handle: usize,
        _listeners: &[EventKind],
    ) -> Result<(), BackendError> {
        unsafe { rust_native_android_remove_view(node_id, handle) };
        Ok(())
    }

    fn set_text(
        &mut self,
        kind: ElementKind,
        handle: usize,
        text: &str,
    ) -> Result<(), BackendError> {
        let (ptr, len) = text_payload(Some(text));
        unsafe { rust_native_android_set_text(element_kind_code(kind), handle, ptr, len) };
        Ok(())
    }

    fn set_color(
        &mut self,
        kind: ElementKind,
        handle: usize,
        color: ColorValue,
    ) -> Result<(), BackendError> {
        unsafe {
            rust_native_android_set_color(
                element_kind_code(kind),
                handle,
                color.r,
                color.g,
                color.b,
                color.a,
            )
        };
        Ok(())
    }

    fn set_font(
        &mut self,
        kind: ElementKind,
        handle: usize,
        size: f32,
        weight: FontWeight,
    ) -> Result<(), BackendError> {
        unsafe { rust_native_android_set_font(element_kind_code(kind), handle, size, font_weight_code(weight)) };
        Ok(())
    }

    fn set_corner_radius(&mut self, handle: usize, radius: f32) -> Result<(), BackendError> {
        unsafe { rust_native_android_set_corner_radius(handle, radius) };
        Ok(())
    }

    fn set_enabled(&mut self, handle: usize, enabled: bool) -> Result<(), BackendError> {
        unsafe { rust_native_android_set_enabled(handle, enabled) };
        Ok(())
    }

    fn set_source(&mut self, handle: usize, source: &str) -> Result<(), BackendError> {
        let (ptr, len) = text_payload(Some(source));
        unsafe { rust_native_android_set_source(handle, ptr, len) };
        Ok(())
    }

    fn bind_tap(&mut self, handle: usize, node_id: UiNodeId) -> Result<(), BackendError> {
        unsafe { rust_native_android_bind_listener(handle, node_id, event_kind_code(EventKind::Tap)) };
        Ok(())
    }

    fn bind_text_input(&mut self, handle: usize, node_id: UiNodeId) -> Result<(), BackendError> {
        unsafe {
            rust_native_android_bind_listener(handle, node_id, event_kind_code(EventKind::TextInput))
        };
        Ok(())
    }

    fn apply_frame(&mut self, handle: usize, frame: LayoutFrame) -> Result<(), BackendError> {
        unsafe {
            rust_native_android_apply_frame(handle, frame.x, frame.y, frame.width, frame.height)
        };
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        unsafe { rust_native_android_flush() };
        Ok(())
    }
}

fn text_payload(text: Option<&str>) -> (*const u8, usize) {
    match text {
        Some(text) => (text.as_ptr(), text.len()),
        None => (std::ptr::null(), 0),
    }
}

fn element_kind_code(kind: ElementKind) -> u32 {
    match kind {
        ElementKind::Stack => 0,
        ElementKind::Text => 1,
        ElementKind::Button => 2,
        ElementKind::Image => 3,
        ElementKind::List => 4,
        ElementKind::Input => 5,
    }
}

fn event_kind_code(kind: EventKind) -> u32 {
    match kind {
        EventKind::Tap => 0,
        EventKind::TextInput => 1,
        EventKind::Scroll => 2,
        EventKind::Appear => 3,
        EventKind::Disappear => 4,
    }
}

fn font_weight_code(weight: FontWeight) -> u32 {
    match weight {
        FontWeight::Regular => 0,
        FontWeight::SemiBold => 1,
        FontWeight::Bold => 2,
    }
}

#[cfg(target_os = "android")]
unsafe extern "C" {
    fn rust_native_android_is_ui_thread() -> bool;
    fn rust_native_android_create_view(kind: u32, text_ptr: *const u8, text_len: usize) -> usize;
    fn rust_native_android_attach_root(node_id: UiNodeId, handle: usize);
    fn rust_native_android_detach_root(node_id: UiNodeId, handle: usize);
    fn rust_native_android_insert_child(
        parent: usize,
        child_id: UiNodeId,
        child: usize,
        index: usize,
    );
    fn rust_native_android_remove_child(parent: usize, child: usize);
    fn rust_native_android_remove_view(node_id: UiNodeId, handle: usize);
    fn rust_native_android_set_text(kind: u32, handle: usize, text_ptr: *const u8, text_len: usize);
    fn rust_native_android_set_color(
        kind: u32,
        handle: usize,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    );
    fn rust_native_android_set_font(kind: u32, handle: usize, size: f32, weight: u32);
    fn rust_native_android_set_corner_radius(handle: usize, radius: f32);
    fn rust_native_android_set_enabled(handle: usize, enabled: bool);
    fn rust_native_android_set_source(handle: usize, text_ptr: *const u8, text_len: usize);
    fn rust_native_android_bind_listener(handle: usize, node_id: UiNodeId, event_kind: u32);
    fn rust_native_android_apply_frame(handle: usize, x: f32, y: f32, width: f32, height: f32);
    fn rust_native_android_flush();
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_is_ui_thread() -> bool {
    true
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_create_view(_kind: u32, _text_ptr: *const u8, _text_len: usize) -> usize {
    1
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_attach_root(_node_id: UiNodeId, _handle: usize) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_detach_root(_node_id: UiNodeId, _handle: usize) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_insert_child(
    _parent: usize,
    _child_id: UiNodeId,
    _child: usize,
    _index: usize,
) {
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_remove_child(_parent: usize, _child: usize) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_remove_view(_node_id: UiNodeId, _handle: usize) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_text(
    _kind: u32,
    _handle: usize,
    _text_ptr: *const u8,
    _text_len: usize,
) {
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_color(
    _kind: u32,
    _handle: usize,
    _r: f32,
    _g: f32,
    _b: f32,
    _a: f32,
) {
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_font(_kind: u32, _handle: usize, _size: f32, _weight: u32) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_corner_radius(_handle: usize, _radius: f32) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_enabled(_handle: usize, _enabled: bool) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_set_source(_handle: usize, _text_ptr: *const u8, _text_len: usize) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_bind_listener(_handle: usize, _node_id: UiNodeId, _event_kind: u32) {}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_apply_frame(
    _handle: usize,
    _x: f32,
    _y: f32,
    _width: f32,
    _height: f32,
) {
}

#[cfg(not(target_os = "android"))]
unsafe fn rust_native_android_flush() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_cover_protocol_v1_surface() {
        let kinds = [
            element_kind_code(ElementKind::Stack),
            element_kind_code(ElementKind::Text),
            element_kind_code(ElementKind::Button),
            element_kind_code(ElementKind::Image),
            element_kind_code(ElementKind::List),
            element_kind_code(ElementKind::Input),
        ];

        assert_eq!(kinds.len(), 6);
        assert_eq!(kinds, [0, 1, 2, 3, 4, 5]);
        assert_eq!(event_kind_code(EventKind::Tap), 0);
        assert_eq!(event_kind_code(EventKind::TextInput), 1);
        assert_eq!(font_weight_code(FontWeight::Bold), 2);
    }
}
