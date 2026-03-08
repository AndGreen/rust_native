use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use mf_core::{Fragment, View};
use mf_widgets::button::ButtonAction;
use mf_widgets::button::ButtonView;
use mf_widgets::image::ImageView;
use mf_widgets::layout::{Alignment as WidgetAlignment, Axis as WidgetAxis, StackElement};
use mf_widgets::text::TextView;
use native_schema::{
    Alignment, Axis, ColorValue, DimensionValue, EdgeInsets, ElementKind, EventKind, FontWeight,
    LayoutFrame, Mutation, PropKey, PropValue, ProtocolVersion, UiEvent, UiNodeId,
};
use taffy::prelude::*;

const DEFAULT_HOST_WIDTH: f32 = 390.0;
const DEFAULT_HOST_HEIGHT: f32 = 844.0;
const DEFAULT_FONT_SIZE: f32 = 14.0;
const TEXT_WIDTH_FACTOR: f32 = 0.6;
const TEXT_HEIGHT_FACTOR: f32 = 1.2;
const BUTTON_HORIZONTAL_PADDING: f32 = 16.0;
const BUTTON_VERTICAL_PADDING: f32 = 10.0;
const BUTTON_MIN_HEIGHT: f32 = 32.0;
const FALLBACK_IMAGE_SIZE: f32 = 44.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostSize {
    pub width: f32,
    pub height: f32,
}

impl HostSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for HostSize {
    fn default() -> Self {
        Self::new(DEFAULT_HOST_WIDTH, DEFAULT_HOST_HEIGHT)
    }
}

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
    current_layout: Vec<LayoutFrame>,
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
            current_layout: Vec::new(),
            next_id: 1,
            tap_handlers: HashMap::new(),
            force_full_resync: false,
        }
    }

    pub fn render(&mut self, view: &View, host_size: HostSize) -> RenderBatch {
        let previous = if self.force_full_resync {
            None
        } else {
            self.current.clone()
        };

        let Some(next) = self.build_root(previous.as_ref(), view) else {
            self.current = None;
            self.current_layout.clear();
            self.tap_handlers.clear();
            self.force_full_resync = false;
            return RenderBatch::default();
        };

        let mut mutations = Vec::new();
        match previous.as_ref() {
            None => emit_create_subtree(&next, &mut mutations),
            Some(previous_root) => diff_node(previous_root, &next, &mut mutations),
        }

        let next_layout = compute_layout_frames(&next, host_size);
        let layout_changed = self.force_full_resync || self.current_layout != next_layout;

        self.tap_handlers = collect_tap_handlers(&next);
        self.current = Some(next);
        self.current_layout = next_layout.clone();
        self.force_full_resync = false;

        RenderBatch {
            protocol_version: ProtocolVersion::V1,
            mutations,
            layout: if layout_changed {
                next_layout
            } else {
                Vec::new()
            },
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

    fn build_root(
        &mut self,
        previous: Option<&CanonicalNode>,
        view: &View,
    ) -> Option<CanonicalNode> {
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
            previous.expect("previous node").id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };

        let previous_children = previous.map(|node| node.children.as_slice()).unwrap_or(&[]);
        let mut children = Vec::new();
        let flat_children = flatten_children(view.children());
        for (index, child_view) in flat_children.into_iter().enumerate() {
            let prior = previous_children
                .get(index)
                .filter(|child| child.descriptor == NodeDescriptor::from_view(child_view));
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

#[derive(Debug, Clone, Copy)]
struct LayoutProps {
    axis: Axis,
    spacing: f32,
    padding: EdgeInsets,
    alignment: Alignment,
    width: Option<f32>,
    height: Option<f32>,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    flex_grow: Option<f32>,
    flex_shrink: Option<f32>,
}

impl LayoutProps {
    fn for_node(node: &CanonicalNode) -> Self {
        let mut props = match node.descriptor {
            NodeDescriptor::Element(ElementKind::List) => Self {
                axis: Axis::Vertical,
                spacing: 0.0,
                padding: EdgeInsets::all(0.0),
                alignment: Alignment::Leading,
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
                flex_grow: None,
                flex_shrink: None,
            },
            _ => Self {
                axis: Axis::Vertical,
                spacing: 0.0,
                padding: EdgeInsets::all(0.0),
                alignment: Alignment::Leading,
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
                flex_grow: None,
                flex_shrink: None,
            },
        };

        if let Some(PropValue::Axis(axis)) = prop_value(node, PropKey::Axis) {
            props.axis = *axis;
        }
        if let Some(PropValue::Float(spacing)) = prop_value(node, PropKey::Spacing) {
            props.spacing = *spacing;
        }
        if let Some(PropValue::Insets(padding)) = prop_value(node, PropKey::Padding) {
            props.padding = *padding;
        }
        if let Some(PropValue::Alignment(alignment)) = prop_value(node, PropKey::Alignment) {
            props.alignment = *alignment;
        }

        props.width = dimension_points(node, PropKey::Width);
        props.height = dimension_points(node, PropKey::Height);
        props.min_width = dimension_points(node, PropKey::MinWidth);
        props.min_height = dimension_points(node, PropKey::MinHeight);
        props.max_width = dimension_points(node, PropKey::MaxWidth);
        props.max_height = dimension_points(node, PropKey::MaxHeight);
        props.flex_grow = float_prop(node, PropKey::FlexGrow);
        props.flex_shrink = float_prop(node, PropKey::FlexShrink);

        props
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
            props: button_props(),
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
            props: list_props(),
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

fn button_props() -> Vec<(PropKey, PropValue)> {
    Vec::new()
}

fn image_props(image: &ImageView) -> Vec<(PropKey, PropValue)> {
    let mut props = vec![(
        PropKey::Source,
        PropValue::String(image.source().to_string()),
    )];
    let (width, height) = image.dimensions();
    if let Some(width) = width {
        props.push((
            PropKey::Width,
            PropValue::Dimension(DimensionValue::Points(width)),
        ));
    }
    if let Some(height) = height {
        props.push((
            PropKey::Height,
            PropValue::Dimension(DimensionValue::Points(height)),
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

fn list_props() -> Vec<(PropKey, PropValue)> {
    vec![
        (PropKey::Axis, PropValue::Axis(Axis::Vertical)),
        (PropKey::Spacing, PropValue::Float(0.0)),
        (PropKey::Padding, PropValue::Insets(EdgeInsets::all(0.0))),
        (PropKey::Alignment, PropValue::Alignment(Alignment::Leading)),
    ]
}

fn collect_tap_handlers(root: &CanonicalNode) -> HashMap<UiNodeId, ButtonAction> {
    let mut handlers = HashMap::new();
    collect_handlers_recursive(root, &mut handlers);
    handlers
}

fn collect_handlers_recursive(
    node: &CanonicalNode,
    handlers: &mut HashMap<UiNodeId, ButtonAction>,
) {
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

    if previous.tap_handler.is_some() != next.tap_handler.is_some() || props_removed(previous, next)
    {
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
    if let Some(text) = &node.text {
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

fn float_prop(node: &CanonicalNode, key: PropKey) -> Option<f32> {
    match prop_value(node, key) {
        Some(PropValue::Float(value)) => Some(*value),
        _ => None,
    }
}

fn dimension_points(node: &CanonicalNode, key: PropKey) -> Option<f32> {
    match prop_value(node, key) {
        Some(PropValue::Dimension(DimensionValue::Points(value))) => Some(*value),
        Some(PropValue::Dimension(DimensionValue::Auto)) | None => None,
        _ => None,
    }
}

fn compute_layout_frames(root: &CanonicalNode, host_size: HostSize) -> Vec<LayoutFrame> {
    let mut taffy = Taffy::new();
    let root_node = build_taffy_tree(&mut taffy, root, host_size, true);
    taffy
        .compute_layout(
            root_node,
            Size {
                width: AvailableSpace::Definite(host_size.width),
                height: AvailableSpace::Definite(host_size.height),
            },
        )
        .expect("taffy layout should compute");

    let mut frames = Vec::new();
    collect_layout_frames(&taffy, root, root_node, &mut frames);
    validate_layout_frames(root, &frames);
    frames
}

fn build_taffy_tree(
    taffy: &mut Taffy,
    node: &CanonicalNode,
    host_size: HostSize,
    is_root: bool,
) -> Node {
    let children: Vec<Node> = node
        .children
        .iter()
        .map(|child| build_taffy_tree(taffy, child, host_size, false))
        .collect();
    let style = style_for_node(node, host_size, is_root);

    if children.is_empty() {
        taffy.new_leaf(style).expect("leaf node should be created")
    } else {
        taffy
            .new_with_children(style, &children)
            .expect("container node should be created")
    }
}

fn style_for_node(node: &CanonicalNode, host_size: HostSize, is_root: bool) -> Style {
    let props = LayoutProps::for_node(node);
    let mut style = Style::DEFAULT.clone();

    if is_root {
        style.size = Size {
            width: points(host_size.width),
            height: points(host_size.height),
        };
    }

    style.padding = Rect {
        left: points(props.padding.left),
        right: points(props.padding.right),
        top: points(props.padding.top),
        bottom: points(props.padding.bottom),
    };
    style.align_items = Some(map_alignment(props.alignment));

    if let Some(value) = props.width {
        style.size.width = points(value);
    }
    if let Some(value) = props.height {
        style.size.height = points(value);
    }
    if let Some(value) = props.min_width {
        style.min_size.width = points(value);
    }
    if let Some(value) = props.min_height {
        style.min_size.height = points(value);
    }
    if let Some(value) = props.max_width {
        style.max_size.width = points(value);
    }
    if let Some(value) = props.max_height {
        style.max_size.height = points(value);
    }
    if let Some(value) = props.flex_grow {
        style.flex_grow = value;
    }
    if let Some(value) = props.flex_shrink {
        style.flex_shrink = value;
    }

    match node.descriptor {
        NodeDescriptor::Element(ElementKind::Stack)
        | NodeDescriptor::Element(ElementKind::List) => {
            style.flex_direction = match props.axis {
                Axis::Horizontal => FlexDirection::Row,
                Axis::Vertical => FlexDirection::Column,
            };
            style.gap = match props.axis {
                Axis::Horizontal => Size {
                    width: points(props.spacing),
                    height: zero(),
                },
                Axis::Vertical => Size {
                    width: zero(),
                    height: points(props.spacing),
                },
            };
        }
        NodeDescriptor::Text => {
            let (width, height) = intrinsic_text_size(node);
            style.size = Size {
                width: points(if is_root { host_size.width } else { width }),
                height: points(if is_root { host_size.height } else { height }),
            };
        }
        NodeDescriptor::Element(ElementKind::Button) => {
            let (width, height) = intrinsic_button_size(node);
            style.size = Size {
                width: points(if is_root { host_size.width } else { width }),
                height: points(if is_root { host_size.height } else { height }),
            };
        }
        NodeDescriptor::Element(ElementKind::Image) => {
            let (width, height) = intrinsic_image_size(&props);
            style.size = Size {
                width: points(if is_root { host_size.width } else { width }),
                height: points(if is_root { host_size.height } else { height }),
            };
        }
        NodeDescriptor::Element(_) => {}
    }

    style
}

fn map_alignment(alignment: Alignment) -> AlignItems {
    match alignment {
        Alignment::Leading => AlignItems::Start,
        Alignment::Center => AlignItems::Center,
        Alignment::Trailing => AlignItems::End,
    }
}

fn intrinsic_text_size(node: &CanonicalNode) -> (f32, f32) {
    let font_size = float_prop(node, PropKey::FontSize).unwrap_or(DEFAULT_FONT_SIZE);
    let chars = node
        .text
        .as_deref()
        .map(|text| text.chars().count().max(1) as f32)
        .unwrap_or(1.0);
    (
        chars * font_size * TEXT_WIDTH_FACTOR,
        font_size * TEXT_HEIGHT_FACTOR,
    )
}

fn intrinsic_button_size(node: &CanonicalNode) -> (f32, f32) {
    let (label_width, label_height) = intrinsic_text_size(node);
    (
        label_width + BUTTON_HORIZONTAL_PADDING * 2.0,
        (label_height + BUTTON_VERTICAL_PADDING * 2.0).max(BUTTON_MIN_HEIGHT),
    )
}

fn intrinsic_image_size(props: &LayoutProps) -> (f32, f32) {
    (
        props.width.unwrap_or(FALLBACK_IMAGE_SIZE),
        props.height.unwrap_or(FALLBACK_IMAGE_SIZE),
    )
}

fn collect_layout_frames(
    taffy: &Taffy,
    node: &CanonicalNode,
    taffy_node: Node,
    frames: &mut Vec<LayoutFrame>,
) {
    let layout = taffy.layout(taffy_node).expect("computed layout");
    frames.push(LayoutFrame {
        id: node.id,
        x: layout.location.x,
        y: layout.location.y,
        width: layout.size.width,
        height: layout.size.height,
    });

    for (child, child_taffy) in node
        .children
        .iter()
        .zip(taffy.children(taffy_node).unwrap_or_default())
    {
        collect_layout_frames(taffy, child, child_taffy, frames);
    }
}

fn validate_layout_frames(root: &CanonicalNode, frames: &[LayoutFrame]) {
    let expected_count = count_nodes(root);
    assert_eq!(
        frames.len(),
        expected_count,
        "layout frame count must match rendered node count"
    );

    let mut ids = HashSet::new();
    for frame in frames {
        assert!(
            ids.insert(frame.id),
            "duplicate layout frame id {}",
            frame.id
        );
        frame.validate().expect("layout frame must be valid");
    }
}

fn count_nodes(node: &CanonicalNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use mf_core::{IntoView, View, WithChildren};
    use mf_widgets::prelude::*;

    const TEST_HOST: HostSize = HostSize {
        width: 390.0,
        height: 844.0,
    };

    fn render(runtime: &mut VdomRuntime, view: View) -> RenderBatch {
        runtime.render(&view, TEST_HOST)
    }

    #[test]
    fn initial_mount_emits_create_insert_and_layout() {
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
        assert_eq!(batch.layout.len(), 3);
        assert_eq!(batch.layout[0].width, TEST_HOST.width);
        assert_eq!(batch.layout[0].height, TEST_HOST.height);
    }

    #[test]
    fn text_update_emits_only_set_text_and_no_layout_delta_when_frames_match() {
        let mut runtime = VdomRuntime::new();
        let first = Text("Count: 0").into_view();
        let second = Text("Count: 1").into_view();

        let _ = runtime.render(&first, TEST_HOST);
        let batch = runtime.render(&second, TEST_HOST);

        assert_eq!(batch.mutations.len(), 1);
        assert!(matches!(batch.mutations[0], Mutation::SetText { .. }));
        assert!(batch.layout.is_empty());
    }

    #[test]
    fn removed_props_force_replace() {
        let mut runtime = VdomRuntime::new();
        let first = VStack().with_children(vec![Text("Hello")
            .font(mf_widgets::Font::bold(24.0))
            .into_view()]);
        let second = VStack().with_children(vec![Text("Hello").into_view()]);

        let _ = runtime.render(&first, TEST_HOST);
        let batch = runtime.render(&second, TEST_HOST);

        assert!(matches!(batch.mutations[0], Mutation::ReplaceNode { .. }));
        assert!(!batch.layout.is_empty());
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
        let batch = runtime.render(&view, TEST_HOST);
        let button_id = batch
            .mutations
            .iter()
            .find_map(|mutation| match mutation {
                Mutation::AttachEventListener {
                    id,
                    event: EventKind::Tap,
                } => Some(*id),
                _ => None,
            })
            .expect("button id");

        runtime.dispatch_event(UiEvent::Tap { id: button_id });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn nested_layout_is_parent_first_and_covers_every_node() {
        let mut runtime = VdomRuntime::new();
        let batch = render(
            &mut runtime,
            VStack().spacing(16.0).padding(24.0).with_children(vec![
                Text("Albums").font(Font::bold(32.0)).into_view(),
                HStack().spacing(12.0).padding(8.0).with_children(vec![
                    Image("cover.jpg").size(60.0, 60.0).into_view(),
                    VStack()
                        .alignment(mf_widgets::Alignment::Leading)
                        .with_children(vec![
                            Text("Explorations").font(Font::semibold(18.0)).into_view(),
                            Text("Nova Collective").into_view(),
                        ]),
                    Button("Like").into_view(),
                ]),
            ]),
        );

        assert_eq!(batch.layout.len(), 8);
        assert_eq!(batch.layout[0].x, 0.0);
        assert_eq!(batch.layout[0].y, 0.0);
        assert!(batch.layout.windows(2).all(|pair| pair[0].id != pair[1].id));
    }

    #[test]
    fn layout_frame_validation_rejects_duplicates() {
        let node = CanonicalNode {
            id: 1,
            descriptor: NodeDescriptor::Element(ElementKind::Stack),
            props: Vec::new(),
            text: None,
            tap_handler: None,
            children: Vec::new(),
        };

        let frames = vec![
            LayoutFrame {
                id: 1,
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
            LayoutFrame {
                id: 1,
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        ];

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            validate_layout_frames(&node, &frames)
        }));
        assert!(result.is_err());
    }
}
