use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{ElementKind, EventKind, LayoutFrame, PropKey, PropValue, UiEvent, UiNodeId};
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

use crate::executor::PlatformAdapter;

use super::events::{
    emit_appear_if_needed, queue_event, shared_event_target, take_events, unregister_binding,
    update_binding,
};
use super::uikit::{
    add_subview, apply_background_color, apply_color, apply_corner_radius, apply_enabled,
    apply_font, apply_image_source, bootstrap_host, create_button, create_image_view, create_label,
    create_plain_view, create_text_field, insert_subview, release_object, remove_from_superview,
    set_text_on_view, CGRect, HostViews,
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
            ElementKind::Stack | ElementKind::SafeArea | ElementKind::List => create_plain_view(),
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
            PropKey::BackgroundColor => {
                apply_background_color(handle, props.get(&PropKey::BackgroundColor))
            }
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
        let rect = CGRect::new(
            frame.x as f64,
            frame.y as f64,
            frame.width as f64,
            frame.height as f64,
        );
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
