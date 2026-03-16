use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{
    ColorValue, ElementKind, EventKind, LayoutFrame, PropKey, PropValue, UiEvent, UiNodeId,
};
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

use crate::executor::PlatformAdapter;
use crate::shared::bindings::{
    emit_appear_if_needed, queue_event, take_events, unregister_binding, update_binding,
};
use crate::shared::props;

use super::events::shared_event_target;
use super::uikit::{
    add_subview, apply_background_color, apply_color, apply_corner_radius, apply_enabled,
    apply_focus, apply_font, apply_image_source, apply_opacity, bootstrap_host, create_button,
    create_image_view, create_label, create_plain_view, create_text_field, insert_subview,
    release_object, remove_from_superview, set_text_on_view, CGRect, CGSize, HostViews,
};

#[derive(Default)]
pub(super) struct IosAdapter {
    host: Option<HostViews>,
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
            ElementKind::Stack
            | ElementKind::Container
            | ElementKind::SafeArea
            | ElementKind::List => create_plain_view(),
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

    fn attach_root(&mut self, node_id: UiNodeId, handle: Self::Handle) -> Result<(), BackendError> {
        let host = self.ensure_host()?;
        add_subview(host.host_view, handle);
        emit_appear_if_needed(node_id, handle_key(handle));
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
        emit_appear_if_needed(child_id, handle_key(child));
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
        unregister_binding(handle_key(handle));
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
            PropKey::Color => apply_color(
                kind,
                handle,
                props::color(props, PropKey::Color)
                    .and_then(Result::ok)
                    .map(PropValue::Color)
                    .as_ref(),
            ),
            PropKey::BackgroundColor
            | PropKey::Opacity
            | PropKey::CornerRadius
            | PropKey::CornerRadii
            | PropKey::FullRound
            | PropKey::Border
            | PropKey::Stroke
            | PropKey::Shadow => reapply_visual_style(handle, props, None),
            PropKey::FontSize | PropKey::FontWeight => apply_font(kind, handle, props),
            PropKey::Enabled => {
                let value = props::bool_value(props, PropKey::Enabled)
                    .and_then(Result::ok)
                    .map(PropValue::Bool);
                apply_enabled(handle, value.as_ref());
                Ok(())
            }
            PropKey::Focused => {
                if kind != ElementKind::Input {
                    return Err(BackendError::BatchRejected(format!(
                        "Focused is unsupported for {kind:?}"
                    )));
                }
                let value = props::bool_value(props, PropKey::Focused)
                    .and_then(Result::ok)
                    .map(PropValue::Bool);
                apply_focus(handle, value.as_ref())
            }
            PropKey::Source => {
                let value = props::string(props, PropKey::Source)
                    .and_then(Result::ok)
                    .map(|value| PropValue::String(value.to_string()));
                apply_image_source(handle, value.as_ref())
            }
            PropKey::Offset => reapply_frame_from_handle(handle, props),
            PropKey::Axis
            | PropKey::Spacing
            | PropKey::Padding
            | PropKey::Alignment
            | PropKey::JustifyContent
            | PropKey::SafeAreaEdges
            | PropKey::Width
            | PropKey::Height
            | PropKey::MinWidth
            | PropKey::MinHeight
            | PropKey::MaxWidth
            | PropKey::MaxHeight
            | PropKey::FlexGrow
            | PropKey::FlexShrink => {
                eprintln!(
                    "[backend_native/ios] ignoring layout prop {key:?}; layout is driven by Rust"
                );
                Ok(())
            }
        }
    }

    fn unset_prop(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        props: &HashMap<PropKey, PropValue>,
        key: PropKey,
    ) -> Result<(), BackendError> {
        match key {
            PropKey::Color => apply_color(
                kind,
                handle,
                props::color(props, PropKey::Color)
                    .and_then(Result::ok)
                    .map(PropValue::Color)
                    .as_ref(),
            ),
            PropKey::BackgroundColor
            | PropKey::Opacity
            | PropKey::CornerRadius
            | PropKey::CornerRadii
            | PropKey::FullRound
            | PropKey::Border
            | PropKey::Stroke
            | PropKey::Shadow => reapply_visual_style(handle, props, None),
            PropKey::FontSize | PropKey::FontWeight => apply_font(kind, handle, props),
            PropKey::Enabled => {
                let value = props::bool_value(props, PropKey::Enabled)
                    .and_then(Result::ok)
                    .map(PropValue::Bool);
                apply_enabled(handle, value.as_ref());
                Ok(())
            }
            PropKey::Focused => {
                if kind != ElementKind::Input {
                    return Err(BackendError::BatchRejected(format!(
                        "Focused is unsupported for {kind:?}"
                    )));
                }
                let value = props::bool_value(props, PropKey::Focused)
                    .and_then(Result::ok)
                    .map(PropValue::Bool);
                apply_focus(handle, value.as_ref())
            }
            PropKey::Source => {
                let value = props::string(props, PropKey::Source)
                    .and_then(Result::ok)
                    .map(|value| PropValue::String(value.to_string()));
                apply_image_source(handle, value.as_ref())
            }
            PropKey::Offset => reapply_frame_from_handle(handle, props),
            PropKey::Axis
            | PropKey::Spacing
            | PropKey::Padding
            | PropKey::Alignment
            | PropKey::JustifyContent
            | PropKey::SafeAreaEdges
            | PropKey::Width
            | PropKey::Height
            | PropKey::MinWidth
            | PropKey::MinHeight
            | PropKey::MaxWidth
            | PropKey::MaxHeight
            | PropKey::FlexGrow
            | PropKey::FlexShrink => Ok(()),
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
                let should_wire = update_binding(handle_key(handle), node_id, |binding| {
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
                let should_wire = update_binding(handle_key(handle), node_id, |binding| {
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
            EventKind::FocusChanged => {
                if kind != ElementKind::Input {
                    eprintln!("[backend_native/ios] ignoring FocusChanged listener for {kind:?}");
                    return Ok(());
                }
                let should_wire = update_binding(handle_key(handle), node_id, |binding| {
                    let should_wire = !binding.focus_changed;
                    binding.focus_changed = true;
                    should_wire
                });
                if should_wire {
                    let target = shared_event_target();
                    unsafe {
                        let _: () = msg_send![
                            handle,
                            addTarget: target
                            action: sel!(handleEditingDidBegin:)
                            forControlEvents: 1u64 << 16
                        ];
                        let _: () = msg_send![
                            handle,
                            addTarget: target
                            action: sel!(handleEditingDidEnd:)
                            forControlEvents: 1u64 << 18
                        ];
                    }
                }
            }
            EventKind::Appear => {
                update_binding(handle_key(handle), node_id, |binding| binding.appear = true);
            }
            EventKind::Disappear => {
                update_binding(handle_key(handle), node_id, |binding| {
                    binding.disappear = true
                });
            }
            EventKind::Scroll => {
                eprintln!("[backend_native/ios] ignoring Scroll listener in P0-06");
            }
        }
        Ok(())
    }

    fn apply_frame(
        &mut self,
        _kind: ElementKind,
        handle: Self::Handle,
        props: &HashMap<PropKey, PropValue>,
        frame: LayoutFrame,
    ) -> Result<(), BackendError> {
        let rect = offset_rect(frame, props);
        unsafe {
            let _: () = msg_send![handle, setFrame: rect];
        }
        reapply_visual_style(handle, props, Some(frame))?;
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

fn handle_key(handle: *mut Object) -> usize {
    handle as usize
}

fn reapply_visual_style(
    handle: *mut Object,
    props: &HashMap<PropKey, PropValue>,
    frame: Option<LayoutFrame>,
) -> Result<(), BackendError> {
    apply_background_color(
        handle,
        props::color(props, PropKey::BackgroundColor)
            .and_then(Result::ok)
            .map(PropValue::Color)
            .as_ref(),
    )?;
    apply_opacity(
        handle,
        props::float(props, PropKey::Opacity)
            .and_then(Result::ok)
            .map(PropValue::Float)
            .as_ref(),
    );
    apply_border(handle, props);
    apply_shadow(handle, props);
    let radius = resolved_corner_radius(props, frame);
    let value = if radius > 0.0 {
        Some(PropValue::Float(radius))
    } else {
        None
    };
    apply_corner_radius(handle, value.as_ref());
    Ok(())
}

fn reapply_frame_from_handle(
    handle: *mut Object,
    props: &HashMap<PropKey, PropValue>,
) -> Result<(), BackendError> {
    let current: CGRect = unsafe { msg_send![handle, frame] };
    let frame = LayoutFrame {
        id: 0,
        x: current.origin.x as f32,
        y: current.origin.y as f32,
        width: current.size.width as f32,
        height: current.size.height as f32,
    };
    let rect = offset_rect(frame, props);
    unsafe {
        let _: () = msg_send![handle, setFrame: rect];
    }
    reapply_visual_style(handle, props, Some(frame))
}

fn offset_rect(frame: LayoutFrame, props: &HashMap<PropKey, PropValue>) -> CGRect {
    let offset = props::point(props, PropKey::Offset)
        .and_then(Result::ok)
        .unwrap_or(native_schema::PointValue::new(0.0, 0.0));
    CGRect::new(
        (frame.x + offset.x) as f64,
        (frame.y + offset.y) as f64,
        frame.width as f64,
        frame.height as f64,
    )
}

fn resolved_corner_radius(props: &HashMap<PropKey, PropValue>, frame: Option<LayoutFrame>) -> f32 {
    let full_round = props::bool_value(props, PropKey::FullRound)
        .and_then(Result::ok)
        .unwrap_or(false);
    if full_round {
        return frame
            .map(|frame| frame.width.min(frame.height) / 2.0)
            .unwrap_or(0.0);
    }

    if let Some(radii) = props::corner_radii(props, PropKey::CornerRadii).and_then(Result::ok) {
        return radii
            .top_left
            .max(radii.top_right)
            .max(radii.bottom_right)
            .max(radii.bottom_left);
    }

    props::float(props, PropKey::CornerRadius)
        .and_then(Result::ok)
        .unwrap_or(0.0)
}

fn apply_border(handle: *mut Object, props: &HashMap<PropKey, PropValue>) {
    let style = props::line_style(props, PropKey::Border)
        .and_then(Result::ok)
        .or_else(|| props::line_style(props, PropKey::Stroke).and_then(Result::ok));
    unsafe {
        let layer: *mut Object = msg_send![handle, layer];
        match style {
            Some(style) => {
                let cg_color = cg_color(style.color);
                let _: () = msg_send![layer, setBorderWidth: style.width as f64];
                let _: () = msg_send![layer, setBorderColor: cg_color];
            }
            None => {
                let _: () = msg_send![layer, setBorderWidth: 0.0f64];
                let _: () = msg_send![layer, setBorderColor: std::ptr::null_mut::<Object>()];
            }
        }
    }
}

fn apply_shadow(handle: *mut Object, props: &HashMap<PropKey, PropValue>) {
    let shadow = props::shadow(props, PropKey::Shadow).and_then(Result::ok);
    unsafe {
        let layer: *mut Object = msg_send![handle, layer];
        match shadow {
            Some(shadow) => {
                let _: () = msg_send![layer, setShadowColor: cg_color(shadow.color)];
                let _: () = msg_send![layer, setShadowOpacity: shadow.color.a];
                let _: () = msg_send![layer, setShadowRadius: shadow.radius as f64];
                let _: () = msg_send![
                    layer,
                    setShadowOffset: CGSize {
                        width: shadow.offset.x as f64,
                        height: shadow.offset.y as f64,
                    }
                ];
            }
            None => {
                let _: () = msg_send![layer, setShadowOpacity: 0.0f32];
                let _: () = msg_send![layer, setShadowRadius: 0.0f64];
                let _: () = msg_send![
                    layer,
                    setShadowOffset: CGSize {
                        width: 0.0,
                        height: 0.0,
                    }
                ];
                let _: () = msg_send![layer, setShadowColor: std::ptr::null_mut::<Object>()];
            }
        }
    }
}

fn cg_color(color: ColorValue) -> *mut Object {
    let color = ui_color(color);
    unsafe { msg_send![color, CGColor] }
}

fn ui_color(color: ColorValue) -> *mut Object {
    unsafe {
        msg_send![
            objc::class!(UIColor),
            colorWithRed: color.r as f64
            green: color.g as f64
            blue: color.b as f64
            alpha: color.a as f64
        ]
    }
}
