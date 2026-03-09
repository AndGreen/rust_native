use std::collections::HashMap;
use std::sync::Arc;

use mf_core::{Fragment, View};
use mf_widgets::button::ButtonAction;
use mf_widgets::button::ButtonView;
use mf_widgets::image::ImageView;
use mf_widgets::layout::{Alignment as WidgetAlignment, Axis as WidgetAxis, StackElement};
use mf_widgets::text::TextView;
use native_schema::{
    Alignment, Axis, ColorValue, DimensionValue, EdgeInsets, ElementKind, FontWeight, PropKey,
    PropValue, UiNodeId,
};

#[derive(Clone)]
pub(crate) struct CanonicalNode {
    pub(crate) id: UiNodeId,
    pub(crate) descriptor: NodeDescriptor,
    pub(crate) props: Vec<(PropKey, PropValue)>,
    pub(crate) text: Option<String>,
    pub(crate) tap_handler: Option<ButtonAction>,
    pub(crate) children: Vec<CanonicalNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeDescriptor {
    Text,
    Element(ElementKind),
}

impl NodeDescriptor {
    pub(crate) fn from_view(view: &View) -> Self {
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

pub(crate) fn is_fragment(view: &View) -> bool {
    view.element().as_any().is::<Fragment>()
}

pub(crate) fn flatten_children<'a>(children: &'a [View]) -> Vec<&'a View> {
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

pub(crate) fn canonicalize_view(
    id: UiNodeId,
    view: &View,
    children: Vec<CanonicalNode>,
) -> CanonicalNode {
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

pub(crate) fn collect_tap_handlers(root: &CanonicalNode) -> HashMap<UiNodeId, ButtonAction> {
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

pub(crate) fn prop_value(node: &CanonicalNode, key: PropKey) -> Option<&PropValue> {
    node.props
        .iter()
        .find_map(|(candidate, value)| (*candidate == key).then_some(value))
}

pub(crate) fn float_prop(node: &CanonicalNode, key: PropKey) -> Option<f32> {
    match prop_value(node, key) {
        Some(PropValue::Float(value)) => Some(*value),
        _ => None,
    }
}

pub(crate) fn dimension_points(node: &CanonicalNode, key: PropKey) -> Option<f32> {
    match prop_value(node, key) {
        Some(PropValue::Dimension(DimensionValue::Points(value))) => Some(*value),
        Some(PropValue::Dimension(DimensionValue::Auto)) | None => None,
        _ => None,
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
