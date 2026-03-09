#![cfg_attr(test, allow(dead_code))]

use std::collections::HashMap;

use backend_api::BackendError;
use native_schema::{
    ColorValue, ElementKind, EventKind, FontWeight, LayoutFrame, PropKey, PropValue, UiEvent,
    UiNodeId,
};

use crate::executor::PlatformAdapter;

use super::events::{
    emit_appear_if_needed, queue_event, take_events, unregister_binding, update_binding,
};

pub(super) trait AndroidBridge {
    fn is_ui_thread(&self) -> bool;
    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<usize, BackendError>;
    fn attach_root(&mut self, node_id: UiNodeId, handle: usize) -> Result<(), BackendError>;
    fn detach_root(&mut self, node_id: UiNodeId, handle: usize) -> Result<(), BackendError>;
    fn insert_child(
        &mut self,
        parent: usize,
        child_id: UiNodeId,
        child: usize,
        index: usize,
    ) -> Result<(), BackendError>;
    fn remove_child(&mut self, parent: usize, child: usize) -> Result<(), BackendError>;
    fn remove_view(
        &mut self,
        node_id: UiNodeId,
        handle: usize,
        listeners: &[EventKind],
    ) -> Result<(), BackendError>;
    fn set_text(
        &mut self,
        kind: ElementKind,
        handle: usize,
        text: &str,
    ) -> Result<(), BackendError>;
    fn set_color(
        &mut self,
        kind: ElementKind,
        handle: usize,
        color: ColorValue,
    ) -> Result<(), BackendError>;
    fn set_background_color(
        &mut self,
        handle: usize,
        color: ColorValue,
    ) -> Result<(), BackendError>;
    fn set_font(
        &mut self,
        kind: ElementKind,
        handle: usize,
        size: f32,
        weight: FontWeight,
    ) -> Result<(), BackendError>;
    fn set_corner_radius(&mut self, handle: usize, radius: f32) -> Result<(), BackendError>;
    fn set_enabled(&mut self, handle: usize, enabled: bool) -> Result<(), BackendError>;
    fn set_source(&mut self, handle: usize, source: &str) -> Result<(), BackendError>;
    fn bind_tap(&mut self, handle: usize, node_id: UiNodeId) -> Result<(), BackendError>;
    fn bind_text_input(&mut self, handle: usize, node_id: UiNodeId) -> Result<(), BackendError>;
    fn apply_frame(&mut self, handle: usize, frame: LayoutFrame) -> Result<(), BackendError>;
    fn flush(&mut self) -> Result<(), BackendError>;
    fn drain_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}

pub(super) struct AndroidAdapter<B> {
    bridge: B,
}

impl<B> AndroidAdapter<B> {
    pub(super) fn new(bridge: B) -> Self {
        Self { bridge }
    }
}

impl<B> AndroidAdapter<B>
where
    B: AndroidBridge,
{
    pub(super) fn is_ui_thread(&self) -> bool {
        self.bridge.is_ui_thread()
    }
}

impl<B> PlatformAdapter for AndroidAdapter<B>
where
    B: AndroidBridge,
{
    type Handle = usize;

    fn create_view(
        &mut self,
        kind: ElementKind,
        text: Option<&str>,
    ) -> Result<Self::Handle, BackendError> {
        self.bridge.create_view(kind, text)
    }

    fn attach_root(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError> {
        self.bridge.attach_root(node_id, handle)?;
        emit_appear_if_needed(node_id, handle);
        Ok(())
    }

    fn detach_root(
        &mut self,
        node_id: UiNodeId,
        handle: Self::Handle,
    ) -> Result<(), BackendError> {
        self.bridge.detach_root(node_id, handle)
    }

    fn insert_child(
        &mut self,
        parent: Self::Handle,
        child_id: UiNodeId,
        child: Self::Handle,
        index: usize,
    ) -> Result<(), BackendError> {
        self.bridge.insert_child(parent, child_id, child, index)?;
        emit_appear_if_needed(child_id, child);
        Ok(())
    }

    fn remove_child(
        &mut self,
        parent: Self::Handle,
        child: Self::Handle,
    ) -> Result<(), BackendError> {
        self.bridge.remove_child(parent, child)
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
        self.bridge.remove_view(node_id, handle, listeners)
    }

    fn set_text(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        text: &str,
    ) -> Result<(), BackendError> {
        self.bridge.set_text(kind, handle, text)
    }

    fn set_prop(
        &mut self,
        kind: ElementKind,
        handle: Self::Handle,
        props: &HashMap<PropKey, PropValue>,
        key: PropKey,
    ) -> Result<(), BackendError> {
        match key {
            PropKey::Color => match props.get(&PropKey::Color) {
                Some(PropValue::Color(color)) => self.bridge.set_color(kind, handle, *color),
                _ => {
                    eprintln!("[backend_native/android] ignoring invalid Color prop");
                    Ok(())
                }
            },
            PropKey::BackgroundColor => match props.get(&PropKey::BackgroundColor) {
                Some(PropValue::Color(color)) => self.bridge.set_background_color(handle, *color),
                _ => {
                    eprintln!("[backend_native/android] ignoring invalid BackgroundColor prop");
                    Ok(())
                }
            },
            PropKey::FontSize | PropKey::FontWeight => {
                let size = match props.get(&PropKey::FontSize) {
                    Some(PropValue::Float(size)) => *size,
                    Some(_) => {
                        eprintln!("[backend_native/android] invalid FontSize prop");
                        return Ok(());
                    }
                    None => 14.0,
                };
                let weight = match props.get(&PropKey::FontWeight) {
                    Some(PropValue::FontWeight(weight)) => *weight,
                    Some(_) => {
                        eprintln!("[backend_native/android] invalid FontWeight prop");
                        return Ok(());
                    }
                    None => FontWeight::Regular,
                };
                self.bridge.set_font(kind, handle, size, weight)
            }
            PropKey::CornerRadius => match props.get(&PropKey::CornerRadius) {
                Some(PropValue::Float(radius)) => self.bridge.set_corner_radius(handle, *radius),
                _ => {
                    eprintln!("[backend_native/android] ignoring invalid CornerRadius prop");
                    Ok(())
                }
            },
            PropKey::Enabled => match props.get(&PropKey::Enabled) {
                Some(PropValue::Bool(enabled)) => self.bridge.set_enabled(handle, *enabled),
                _ => {
                    eprintln!("[backend_native/android] ignoring invalid Enabled prop");
                    Ok(())
                }
            },
            PropKey::Source => match props.get(&PropKey::Source) {
                Some(PropValue::String(source)) => self.bridge.set_source(handle, source),
                _ => {
                    eprintln!("[backend_native/android] ignoring invalid Source prop");
                    Ok(())
                }
            },
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
                eprintln!(
                    "[backend_native/android] ignoring layout prop {key:?}; layout is driven by Rust"
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
                    eprintln!("[backend_native/android] ignoring Tap listener for {kind:?}");
                    return Ok(());
                }
                let should_wire = update_binding(handle, node_id, |binding| {
                    let should_wire = !binding.tap;
                    binding.tap = true;
                    should_wire
                });
                if should_wire {
                    self.bridge.bind_tap(handle, node_id)?;
                }
            }
            EventKind::TextInput => {
                if kind != ElementKind::Input {
                    eprintln!("[backend_native/android] ignoring TextInput listener for {kind:?}");
                    return Ok(());
                }
                let should_wire = update_binding(handle, node_id, |binding| {
                    let should_wire = !binding.text_input;
                    binding.text_input = true;
                    should_wire
                });
                if should_wire {
                    self.bridge.bind_text_input(handle, node_id)?;
                }
            }
            EventKind::Appear => {
                update_binding(handle, node_id, |binding| binding.appear = true);
            }
            EventKind::Disappear => {
                update_binding(handle, node_id, |binding| binding.disappear = true);
            }
            EventKind::Scroll => {
                eprintln!("[backend_native/android] ignoring Scroll listener in P0-07");
            }
        }
        Ok(())
    }

    fn apply_frame(
        &mut self,
        handle: Self::Handle,
        frame: LayoutFrame,
    ) -> Result<(), BackendError> {
        self.bridge.apply_frame(handle, frame)
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        self.bridge.flush()
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        let mut events = take_events();
        events.extend(self.bridge.drain_events());
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Op {
        BindTap(usize, UiNodeId),
        BindTextInput(usize, UiNodeId),
        SetColor(ElementKind, usize, ColorValue),
        SetBackgroundColor(usize, ColorValue),
        SetFont(ElementKind, usize, f32, FontWeight),
        SetCornerRadius(usize, f32),
        SetEnabled(usize, bool),
        SetSource(usize, String),
    }

    #[derive(Default)]
    struct MockBridge {
        ops: Vec<Op>,
    }

    impl AndroidBridge for MockBridge {
        fn is_ui_thread(&self) -> bool {
            true
        }

        fn create_view(
            &mut self,
            _kind: ElementKind,
            _text: Option<&str>,
        ) -> Result<usize, BackendError> {
            Ok(1)
        }

        fn attach_root(&mut self, _node_id: UiNodeId, _handle: usize) -> Result<(), BackendError> {
            Ok(())
        }

        fn detach_root(&mut self, _node_id: UiNodeId, _handle: usize) -> Result<(), BackendError> {
            Ok(())
        }

        fn insert_child(
            &mut self,
            _parent: usize,
            _child_id: UiNodeId,
            _child: usize,
            _index: usize,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn remove_child(&mut self, _parent: usize, _child: usize) -> Result<(), BackendError> {
            Ok(())
        }

        fn remove_view(
            &mut self,
            _node_id: UiNodeId,
            _handle: usize,
            _listeners: &[EventKind],
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn set_text(
            &mut self,
            _kind: ElementKind,
            _handle: usize,
            _text: &str,
        ) -> Result<(), BackendError> {
            Ok(())
        }

        fn set_color(
            &mut self,
            kind: ElementKind,
            handle: usize,
            color: ColorValue,
        ) -> Result<(), BackendError> {
            self.ops.push(Op::SetColor(kind, handle, color));
            Ok(())
        }

        fn set_background_color(
            &mut self,
            handle: usize,
            color: ColorValue,
        ) -> Result<(), BackendError> {
            self.ops.push(Op::SetBackgroundColor(handle, color));
            Ok(())
        }

        fn set_font(
            &mut self,
            kind: ElementKind,
            handle: usize,
            size: f32,
            weight: FontWeight,
        ) -> Result<(), BackendError> {
            self.ops.push(Op::SetFont(kind, handle, size, weight));
            Ok(())
        }

        fn set_corner_radius(&mut self, handle: usize, radius: f32) -> Result<(), BackendError> {
            self.ops.push(Op::SetCornerRadius(handle, radius));
            Ok(())
        }

        fn set_enabled(&mut self, handle: usize, enabled: bool) -> Result<(), BackendError> {
            self.ops.push(Op::SetEnabled(handle, enabled));
            Ok(())
        }

        fn set_source(&mut self, handle: usize, source: &str) -> Result<(), BackendError> {
            self.ops.push(Op::SetSource(handle, source.to_string()));
            Ok(())
        }

        fn bind_tap(&mut self, handle: usize, node_id: UiNodeId) -> Result<(), BackendError> {
            self.ops.push(Op::BindTap(handle, node_id));
            Ok(())
        }

        fn bind_text_input(
            &mut self,
            handle: usize,
            node_id: UiNodeId,
        ) -> Result<(), BackendError> {
            self.ops.push(Op::BindTextInput(handle, node_id));
            Ok(())
        }

        fn apply_frame(&mut self, _handle: usize, _frame: LayoutFrame) -> Result<(), BackendError> {
            Ok(())
        }

        fn flush(&mut self) -> Result<(), BackendError> {
            Ok(())
        }
    }

    #[test]
    fn visual_props_forward_to_bridge_and_layout_props_are_ignored() {
        let mut adapter = AndroidAdapter::new(MockBridge::default());
        let mut props = HashMap::new();
        props.insert(
            PropKey::Color,
            PropValue::Color(ColorValue::new(1.0, 0.5, 0.0, 1.0)),
        );
        props.insert(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(0.1, 0.2, 0.3, 0.8)),
        );
        props.insert(PropKey::FontSize, PropValue::Float(18.0));
        props.insert(
            PropKey::FontWeight,
            PropValue::FontWeight(FontWeight::Bold),
        );
        props.insert(PropKey::CornerRadius, PropValue::Float(12.0));
        props.insert(PropKey::Enabled, PropValue::Bool(false));
        props.insert(
            PropKey::Source,
            PropValue::String("cover.png".to_string()),
        );
        props.insert(PropKey::Padding, PropValue::Insets(native_schema::EdgeInsets::all(8.0)));

        adapter
            .set_prop(ElementKind::Button, 7, &props, PropKey::Color)
            .unwrap();
        adapter
            .set_prop(ElementKind::Button, 7, &props, PropKey::BackgroundColor)
            .unwrap();
        adapter
            .set_prop(ElementKind::Button, 7, &props, PropKey::FontSize)
            .unwrap();
        adapter
            .set_prop(ElementKind::Button, 7, &props, PropKey::CornerRadius)
            .unwrap();
        adapter
            .set_prop(ElementKind::Button, 7, &props, PropKey::Enabled)
            .unwrap();
        adapter
            .set_prop(ElementKind::Image, 7, &props, PropKey::Source)
            .unwrap();
        adapter
            .set_prop(ElementKind::Stack, 7, &props, PropKey::Padding)
            .unwrap();

        assert_eq!(
            adapter.bridge.ops,
            vec![
                Op::SetColor(ElementKind::Button, 7, ColorValue::new(1.0, 0.5, 0.0, 1.0)),
                Op::SetBackgroundColor(7, ColorValue::new(0.1, 0.2, 0.3, 0.8)),
                Op::SetFont(ElementKind::Button, 7, 18.0, FontWeight::Bold),
                Op::SetCornerRadius(7, 12.0),
                Op::SetEnabled(7, false),
                Op::SetSource(7, "cover.png".to_string()),
            ]
        );
    }

    #[test]
    fn tap_and_text_input_listeners_bind_once() {
        let mut adapter = AndroidAdapter::new(MockBridge::default());

        adapter
            .attach_listener(ElementKind::Button, 3, 11, EventKind::Tap)
            .unwrap();
        adapter
            .attach_listener(ElementKind::Button, 3, 11, EventKind::Tap)
            .unwrap();
        adapter
            .attach_listener(ElementKind::Input, 4, 12, EventKind::TextInput)
            .unwrap();
        adapter
            .attach_listener(ElementKind::Input, 4, 12, EventKind::TextInput)
            .unwrap();

        assert_eq!(
            adapter.bridge.ops,
            vec![Op::BindTap(3, 11), Op::BindTextInput(4, 12)]
        );
    }

    #[test]
    fn appear_and_disappear_are_queued_from_binding_state() {
        let mut adapter = AndroidAdapter::new(MockBridge::default());

        adapter
            .attach_listener(ElementKind::Button, 9, 21, EventKind::Appear)
            .unwrap();
        adapter.attach_root(21, 9).unwrap();
        adapter
            .remove_view(21, 9, &[EventKind::Disappear])
            .unwrap();

        assert_eq!(
            adapter.drain_events(),
            vec![UiEvent::Appear { id: 21 }, UiEvent::Disappear { id: 21 }]
        );
    }
}
