use std::collections::HashMap;
use std::sync::Arc;

use mf_core::{Fragment, View};
use mf_widgets::button::ButtonAction;
use mf_widgets::button::ButtonView;
use mf_widgets::image::ImageView;
use mf_widgets::layout::{Alignment as WidgetAlignment, Axis as WidgetAxis, StackElement};
use mf_widgets::text::TextView;
use native_schema::{
    Alignment, Axis, ColorValue, EdgeInsets, ElementKind, EventKind, FontWeight, LayoutFrame,
    Mutation, PropKey, PropValue, ProtocolVersion, UiEvent, UiNodeId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderBatch {
    pub protocol_version: ProtocolVersion,
    pub mutations: Vec<Mutation>,
    pub layout: Vec<LayoutFrame>,
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self {
            protocol_version: ProtocolVersion::V1,
            mutations: Vec::new(),
            layout: Vec::new(),
        }
    }
}

pub struct VdomRuntime {
    current: Option<CanonicalNode>,
    next_id: UiNodeId,
    tap_handlers: HashMap<UiNodeId, ButtonAction>,
    force_full_resync: bool,
}

impl Default for VdomRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl VdomRuntime {
    pub fn new() -> Self {
        Self {
            current: None,
            next_id: 1,
            tap_handlers: HashMap::new(),
            force_full_resync: false,
        }
    }

    pub fn render(&mut self, view: &View) -> RenderBatch {
        let previous = if self.force_full_resync {
            None
        } else {
            self.current.clone()
        };

        let Some(next) = self.build_root(previous.as_ref(), view) else {
            self.current = None;
            self.tap_handlers.clear();
            self.force_full_resync = false;
            return RenderBatch::default();
        };

        let mut mutations = Vec::new();
        match previous.as_ref() {
            None => emit_create_subtree(&next, &mut mutations),
            Some(previous_root) => diff_node(previous_root, &next, &mut mutations),
        }

        self.tap_handlers = collect_tap_handlers(&next);
        self.current = Some(next);
        self.force_full_resync = false;

        RenderBatch {
            protocol_version: ProtocolVersion::V1,
            mutations,
            layout: Vec::new(),
        }
    }

    pub fn dispatch_event(&self, event: UiEvent) {
        if let UiEvent::Tap { id } = event {
            if let Some(handler) = self.tap_handlers.get(&id) {
                handler();
            }
        }
    }

    pub fn request_full_resync(&mut self) {
        self.force_full_resync = true;
    }

    fn build_root(&mut self, previous: Option<&CanonicalNode>, view: &View) -> Option<CanonicalNode> {
        if is_fragment(view) {
            let children = flatten_children(view.children());
            let first = children.first()?;
            return Some(self.build_node(previous, first));
        }

        Some(self.build_node(previous, view))
    }

    fn build_node(&mut self, previous: Option<&CanonicalNode>, view: &View) -> CanonicalNode {
        let descriptor = NodeDescriptor::from_view(view);
        let id = if previous
            .map(|node| node.descriptor == descriptor)
            .unwrap_or(false)
        {
            previous.unwrap().id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };

        let previous_children = previous.map(|node| node.children.as_slice()).unwrap_or(&[]);
        let mut children = Vec::new();
        let flat_children = flatten_children(view.children());
        for (index, child_view) in flat_children.into_iter().enumerate() {
            let prior = previous_children.get(index).filter(|child| {
                child.descriptor == NodeDescriptor::from_view(child_view)
            });
            children.push(self.build_node(prior, child_view));
        }

        canonicalize_view(id, view, children)
    }
}

#[derive(Clone)]
struct CanonicalNode {
    id: UiNodeId,
    descriptor: NodeDescriptor,
    props: Vec<(PropKey, PropValue)>,
    text: Option<String>,
    tap_handler: Option<ButtonAction>,
    children: Vec<CanonicalNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeDescriptor {
    Text,
    Element(ElementKind),
}

impl NodeDescriptor {
    fn from_view(view: &View) -> Self {
        if view.element().as_any().is::<TextView>() {
            Self::Text
        } else if view.element().as_any().is::<ButtonView>() {
            Self::Element(ElementKind::Button)
        } else if view.element().as_any().is::<ImageView>() {
            Self::Element(ElementKind::Image)
        } else if view.element().as_any().is::<StackElement>() {
            Self::Element(ElementKind::Stack)
        } else if view.element().name() == "List" {
            Self::Element(ElementKind::List)
        } else {
            Self::Element(ElementKind::Stack)
        }
    }
}

fn is_fragment(view: &View) -> bool {
    view.element().as_any().is::<Fragment>()
}

fn flatten_children(children: &[View]) -> Vec<&View> {
    let mut flat = Vec::new();
    for child in children {
        if is_fragment(child) {
            flat.extend(flatten_children(child.children()));
        } else {
            flat.push(child);
        }
    }
    flat
}

fn canonicalize_view(id: UiNodeId, view: &View, children: Vec<CanonicalNode>) -> CanonicalNode {
    if let Some(text) = view.element().as_any().downcast_ref::<TextView>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Text,
            props: text_props(text),
            text: Some(text.content().to_string()),
            tap_handler: None,
            children,
        };
    }

    if let Some(button) = view.element().as_any().downcast_ref::<ButtonView>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Button),
            props: Vec::new(),
            text: Some(button.label().to_string()),
            tap_handler: button.action().map(Arc::clone),
            children,
        };
    }

    if let Some(image) = view.element().as_any().downcast_ref::<ImageView>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Image),
            props: image_props(image),
            text: None,
            tap_handler: None,
            children,
        };
    }

    if let Some(stack) = view.element().as_any().downcast_ref::<StackElement>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Stack),
            props: stack_props(stack),
            text: None,
            tap_handler: None,
            children,
        };
    }

    if view.element().name() == "List" {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::List),
            props: Vec::new(),
            text: None,
            tap_handler: None,
            children,
        };
    }

    CanonicalNode {
        id,
        descriptor: NodeDescriptor::Element(ElementKind::Stack),
        props: Vec::new(),
        text: None,
        tap_handler: None,
        children,
    }
}

fn text_props(text: &TextView) -> Vec<(PropKey, PropValue)> {
    let mut props = Vec::new();
    if let Some(color) = text.color_value() {
        props.push((
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    if let Some(font) = text.font_value() {
        props.push((PropKey::FontSize, PropValue::Float(font.size)));
        props.push((
            PropKey::FontWeight,
            PropValue::FontWeight(match font.weight {
                mf_widgets::FontWeight::Regular => FontWeight::Regular,
                mf_widgets::FontWeight::SemiBold => FontWeight::SemiBold,
                mf_widgets::FontWeight::Bold => FontWeight::Bold,
            }),
        ));
    }
    props
}

fn image_props(image: &ImageView) -> Vec<(PropKey, PropValue)> {
    let mut props = vec![(
        PropKey::Source,
        PropValue::String(image.source().to_string()),
    )];
    let (width, height) = image.dimensions();
    if let Some(width) = width {
        props.push((PropKey::Width, PropValue::Dimension(native_schema::DimensionValue::Points(width))));
    }
    if let Some(height) = height {
        props.push((
            PropKey::Height,
            PropValue::Dimension(native_schema::DimensionValue::Points(height)),
        ));
    }
    if let Some(radius) = image.corner_radius_value() {
        props.push((PropKey::CornerRadius, PropValue::Float(radius)));
    }
    props
}

fn stack_props(stack: &StackElement) -> Vec<(PropKey, PropValue)> {
    vec![
        (
            PropKey::Axis,
            PropValue::Axis(match stack.axis() {
                WidgetAxis::Horizontal => Axis::Horizontal,
                WidgetAxis::Vertical => Axis::Vertical,
            }),
        ),
        (PropKey::Spacing, PropValue::Float(stack.spacing())),
        (
            PropKey::Padding,
            PropValue::Insets(EdgeInsets::all(stack.padding())),
        ),
        (
            PropKey::Alignment,
            PropValue::Alignment(match stack.alignment() {
                WidgetAlignment::Leading => Alignment::Leading,
                WidgetAlignment::Center => Alignment::Center,
                WidgetAlignment::Trailing => Alignment::Trailing,
            }),
        ),
    ]
}

fn collect_tap_handlers(root: &CanonicalNode) -> HashMap<UiNodeId, ButtonAction> {
    let mut handlers = HashMap::new();
    collect_handlers_recursive(root, &mut handlers);
    handlers
}

fn collect_handlers_recursive(node: &CanonicalNode, handlers: &mut HashMap<UiNodeId, ButtonAction>) {
    if let Some(handler) = &node.tap_handler {
        handlers.insert(node.id, Arc::clone(handler));
    }
    for child in &node.children {
        collect_handlers_recursive(child, handlers);
    }
}

fn emit_create_subtree(node: &CanonicalNode, mutations: &mut Vec<Mutation>) {
    match node.descriptor {
        NodeDescriptor::Text => mutations.push(Mutation::CreateTextNode {
            id: node.id,
            text: node.text.clone().unwrap_or_default(),
        }),
        NodeDescriptor::Element(kind) => mutations.push(Mutation::CreateNode { id: node.id, kind }),
    }

    if let Some(text) = &node.text {
        if matches!(node.descriptor, NodeDescriptor::Element(_)) {
            mutations.push(Mutation::SetText {
                id: node.id,
                text: text.clone(),
            });
        }
    }

    for (key, value) in &node.props {
        mutations.push(Mutation::SetProp {
            id: node.id,
            key: *key,
            value: value.clone(),
        });
    }

    if node.tap_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::Tap,
        });
    }

    for (index, child) in node.children.iter().enumerate() {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: node.id,
            child: child.id,
            index: index as u32,
        });
    }
}

fn diff_node(previous: &CanonicalNode, next: &CanonicalNode, mutations: &mut Vec<Mutation>) {
    if previous.descriptor != next.descriptor {
        mutations.push(replace_mutation(previous.id, next));
        emit_replace_payload(next, mutations);
        return;
    }

    if previous.tap_handler.is_some() != next.tap_handler.is_some() || props_removed(previous, next) {
        mutations.push(replace_mutation(previous.id, next));
        emit_replace_payload(next, mutations);
        return;
    }

    if previous.text != next.text {
        if let Some(text) = &next.text {
            mutations.push(Mutation::SetText {
                id: next.id,
                text: text.clone(),
            });
        }
    }

    for (key, value) in &next.props {
        if prop_value(previous, *key) != Some(value) {
            mutations.push(Mutation::SetProp {
                id: next.id,
                key: *key,
                value: value.clone(),
            });
        }
    }

    let shared_len = previous.children.len().min(next.children.len());
    for index in 0..shared_len {
        diff_node(&previous.children[index], &next.children[index], mutations);
    }

    for child in previous.children.iter().skip(shared_len) {
        mutations.push(Mutation::RemoveNode { id: child.id });
    }

    for (index, child) in next.children.iter().enumerate().skip(shared_len) {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: next.id,
            child: child.id,
            index: index as u32,
        });
    }
}

fn replace_mutation(old: UiNodeId, next: &CanonicalNode) -> Mutation {
    match next.descriptor {
        NodeDescriptor::Text => Mutation::ReplaceNode {
            old,
            new_id: next.id,
            kind: ElementKind::Text,
        },
        NodeDescriptor::Element(kind) => Mutation::ReplaceNode {
            old,
            new_id: next.id,
            kind,
        },
    }
}

fn emit_replace_payload(node: &CanonicalNode, mutations: &mut Vec<Mutation>) {
    if matches!(node.descriptor, NodeDescriptor::Text) {
        if let Some(text) = &node.text {
            mutations.push(Mutation::SetText {
                id: node.id,
                text: text.clone(),
            });
        }
    } else if let Some(text) = &node.text {
        mutations.push(Mutation::SetText {
            id: node.id,
            text: text.clone(),
        });
    }

    for (key, value) in &node.props {
        mutations.push(Mutation::SetProp {
            id: node.id,
            key: *key,
            value: value.clone(),
        });
    }

    if node.tap_handler.is_some() {
        mutations.push(Mutation::AttachEventListener {
            id: node.id,
            event: EventKind::Tap,
        });
    }

    for (index, child) in node.children.iter().enumerate() {
        emit_create_subtree(child, mutations);
        mutations.push(Mutation::InsertChild {
            parent: node.id,
            child: child.id,
            index: index as u32,
        });
    }
}

fn props_removed(previous: &CanonicalNode, next: &CanonicalNode) -> bool {
    previous
        .props
        .iter()
        .any(|(key, _)| prop_value(next, *key).is_none())
}

fn prop_value(node: &CanonicalNode, key: PropKey) -> Option<&PropValue> {
    node.props
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use mf_core::{IntoView, View, WithChildren};
    use mf_widgets::prelude::*;

    fn render(runtime: &mut VdomRuntime, view: View) -> RenderBatch {
        runtime.render(&view)
    }

    #[test]
    fn initial_mount_emits_create_and_insert_sequence() {
        let mut runtime = VdomRuntime::new();
        let batch = render(
            &mut runtime,
            VStack()
                .spacing(12.0)
                .padding(16.0)
                .with_children(vec![Text("Hello").into_view(), Button("Tap").into_view()]),
        );

        assert!(matches!(
            batch.mutations.first(),
            Some(Mutation::CreateNode {
                kind: ElementKind::Stack,
                ..
            })
        ));
        assert!(batch
            .mutations
            .iter()
            .any(|mutation| matches!(mutation, Mutation::CreateTextNode { .. })));
        assert!(batch
            .mutations
            .iter()
            .any(|mutation| matches!(mutation, Mutation::InsertChild { .. })));
    }

    #[test]
    fn text_update_emits_only_set_text() {
        let mut runtime = VdomRuntime::new();
        let first = Text("Count: 0").into_view();
        let second = Text("Count: 1").into_view();

        let _ = runtime.render(&first);
        let batch = runtime.render(&second);

        assert_eq!(batch.mutations.len(), 1);
        assert!(matches!(batch.mutations[0], Mutation::SetText { .. }));
    }

    #[test]
    fn removed_props_force_replace() {
        let mut runtime = VdomRuntime::new();
        let first = Text("Hello").font(mf_widgets::Font::bold(24.0)).into_view();
        let second = Text("Hello").into_view();

        let _ = runtime.render(&first);
        let batch = runtime.render(&second);

        assert!(matches!(batch.mutations[0], Mutation::ReplaceNode { .. }));
    }

    #[test]
    fn button_tap_dispatch_invokes_handler() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = Arc::clone(&calls);
        let view = Button("Tap")
            .on_click(move || {
                calls_for_handler.fetch_add(1, Ordering::Relaxed);
            })
            .into_view();

        let mut runtime = VdomRuntime::new();
        let batch = runtime.render(&view);
        let button_id = batch
            .mutations
            .iter()
            .find_map(|mutation| match mutation {
                Mutation::AttachEventListener { id, event: EventKind::Tap } => Some(*id),
                _ => None,
            })
            .expect("button id");

        runtime.dispatch_event(UiEvent::Tap { id: button_id });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
