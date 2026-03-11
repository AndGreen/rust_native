use std::collections::HashMap;
use std::sync::Arc;

use mf_core::{Fragment, View};
use mf_widgets::button::ButtonAction;
use mf_widgets::button::ButtonView;
use mf_widgets::image::ImageView;
use mf_widgets::input::{FocusChangeAction, InputAction, InputView};
use mf_widgets::layout::{Alignment as WidgetAlignment, Axis as WidgetAxis, StackElement};
use mf_widgets::safe_area::SafeArea;
use mf_widgets::text::TextView;
use native_schema::{
    Alignment, Axis, ColorValue, DimensionValue, EdgeInsets, ElementKind, FontWeight, PropKey,
    PropValue, SafeAreaEdges, UiNodeId,
};

#[derive(Clone)]
pub(crate) struct CanonicalNode {
    pub(crate) id: UiNodeId,
    pub(crate) descriptor: NodeDescriptor,
    pub(crate) props: Vec<(PropKey, PropValue)>,
    pub(crate) text: Option<String>,
    pub(crate) tap_handler: Option<ButtonAction>,
    pub(crate) input_handler: Option<InputAction>,
    pub(crate) focus_change_handler: Option<FocusChangeAction>,
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
        } else if view.element().as_any().is::<InputView>() {
            Self::Element(ElementKind::Input)
        } else if view.element().as_any().is::<SafeArea>() {
            Self::Element(ElementKind::SafeArea)
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

pub(crate) fn flatten_children(children: &[View]) -> Vec<&View> {
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
            input_handler: None,
            focus_change_handler: None,
            children,
        };
    }

    if let Some(button) = view.element().as_any().downcast_ref::<ButtonView>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Button),
            props: button_props(button),
            text: Some(button.label().to_string()),
            tap_handler: button.action().map(Arc::clone),
            input_handler: None,
            focus_change_handler: None,
            children,
        };
    }

    if let Some(input) = view.element().as_any().downcast_ref::<InputView>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Input),
            props: input_props(input),
            text: Some(input.value().to_string()),
            tap_handler: None,
            input_handler: input.input_action().map(Arc::clone),
            focus_change_handler: input.focus_change_action().map(Arc::clone),
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
            input_handler: None,
            focus_change_handler: None,
            children,
        };
    }

    if let Some(safe_area) = view.element().as_any().downcast_ref::<SafeArea>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::SafeArea),
            props: safe_area_props(safe_area),
            text: None,
            tap_handler: None,
            input_handler: None,
            focus_change_handler: None,
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
            input_handler: None,
            focus_change_handler: None,
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
            input_handler: None,
            focus_change_handler: None,
            children,
        };
    }

    CanonicalNode {
        id,
        descriptor: NodeDescriptor::Element(ElementKind::Stack),
        props: Vec::new(),
        text: None,
        tap_handler: None,
        input_handler: None,
        focus_change_handler: None,
        children,
    }
}

pub(crate) fn collect_tap_handlers(root: &CanonicalNode) -> HashMap<UiNodeId, ButtonAction> {
    let mut handlers = HashMap::new();
    collect_tap_handlers_recursive(root, &mut handlers);
    handlers
}

pub(crate) fn collect_input_handlers(root: &CanonicalNode) -> HashMap<UiNodeId, InputAction> {
    let mut handlers = HashMap::new();
    collect_input_handlers_recursive(root, &mut handlers);
    handlers
}

pub(crate) fn collect_focus_change_handlers(
    root: &CanonicalNode,
) -> HashMap<UiNodeId, FocusChangeAction> {
    let mut handlers = HashMap::new();
    collect_focus_change_handlers_recursive(root, &mut handlers);
    handlers
}

fn collect_tap_handlers_recursive(
    node: &CanonicalNode,
    handlers: &mut HashMap<UiNodeId, ButtonAction>,
) {
    if let Some(handler) = &node.tap_handler {
        handlers.insert(node.id, Arc::clone(handler));
    }
    for child in &node.children {
        collect_tap_handlers_recursive(child, handlers);
    }
}

fn collect_input_handlers_recursive(
    node: &CanonicalNode,
    handlers: &mut HashMap<UiNodeId, InputAction>,
) {
    if let Some(handler) = &node.input_handler {
        handlers.insert(node.id, Arc::clone(handler));
    }
    for child in &node.children {
        collect_input_handlers_recursive(child, handlers);
    }
}

fn collect_focus_change_handlers_recursive(
    node: &CanonicalNode,
    handlers: &mut HashMap<UiNodeId, FocusChangeAction>,
) {
    if let Some(handler) = &node.focus_change_handler {
        handlers.insert(node.id, Arc::clone(handler));
    }
    for child in &node.children {
        collect_focus_change_handlers_recursive(child, handlers);
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

fn button_props(button: &ButtonView) -> Vec<(PropKey, PropValue)> {
    let mut props = Vec::new();
    if let Some(color) = button.color_value() {
        props.push((
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    if let Some(color) = button.background_value() {
        props.push((
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    if let Some(radius) = button.corner_radius_value() {
        props.push((PropKey::CornerRadius, PropValue::Float(radius)));
    }
    if !button.is_enabled() {
        props.push((PropKey::Enabled, PropValue::Bool(false)));
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

fn input_props(input: &InputView) -> Vec<(PropKey, PropValue)> {
    let mut props = Vec::new();
    if let Some(color) = input.color_value() {
        props.push((
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    if let Some(color) = input.background_value() {
        props.push((
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    if let Some(font) = input.font_value() {
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
    if let Some(radius) = input.corner_radius_value() {
        props.push((PropKey::CornerRadius, PropValue::Float(radius)));
    }
    if !input.is_enabled() {
        props.push((PropKey::Enabled, PropValue::Bool(false)));
    }
    if input.is_focused() {
        props.push((PropKey::Focused, PropValue::Bool(true)));
    }
    props
}

fn stack_props(stack: &StackElement) -> Vec<(PropKey, PropValue)> {
    let mut props = vec![
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
    ];
    if let Some(color) = stack.background_value() {
        props.push((
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        ));
    }
    props
}

fn safe_area_props(safe_area: &SafeArea) -> Vec<(PropKey, PropValue)> {
    vec![(
        PropKey::SafeAreaEdges,
        PropValue::SafeAreaEdges(match safe_area.edges_value() {
            SafeAreaEdges::Top => SafeAreaEdges::Top,
            SafeAreaEdges::TopBottom => SafeAreaEdges::TopBottom,
            SafeAreaEdges::All => SafeAreaEdges::All,
        }),
    )]
}

fn list_props() -> Vec<(PropKey, PropValue)> {
    vec![
        (PropKey::Axis, PropValue::Axis(Axis::Vertical)),
        (PropKey::Spacing, PropValue::Float(0.0)),
        (PropKey::Padding, PropValue::Insets(EdgeInsets::all(0.0))),
        (PropKey::Alignment, PropValue::Alignment(Alignment::Leading)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_core::WithChildren;
    use mf_widgets::{Button, Color, HStack, Input};

    #[test]
    fn button_props_include_visual_style_and_enabled_state() {
        let button = Button("Save")
            .background(Color::new(0.1, 0.2, 0.3))
            .foreground(Color::new(0.9, 0.8, 0.7).with_alpha(0.6))
            .corner_radius(10.0)
            .enabled(false);

        let props = button_props(&button);

        assert!(props.contains(&(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(0.1, 0.2, 0.3, 1.0))
        )));
        assert!(props.contains(&(
            PropKey::Color,
            PropValue::Color(ColorValue::new(0.9, 0.8, 0.7, 0.6))
        )));
        assert!(props.contains(&(PropKey::CornerRadius, PropValue::Float(10.0))));
        assert!(props.contains(&(PropKey::Enabled, PropValue::Bool(false))));
    }

    #[test]
    fn input_props_include_visual_state_focus_and_enabled() {
        let input = Input("alex")
            .foreground(Color::new(0.2, 0.3, 0.4))
            .background(Color::new(0.9, 0.8, 0.7))
            .corner_radius(12.0)
            .focused(true)
            .enabled(false);

        let props = input_props(&input);

        assert!(props.contains(&(PropKey::Focused, PropValue::Bool(true))));
        assert!(props.contains(&(PropKey::Enabled, PropValue::Bool(false))));
        assert!(props.contains(&(PropKey::CornerRadius, PropValue::Float(12.0))));
    }

    #[test]
    fn stack_props_include_background_color() {
        let view = HStack::new()
            .background(Color::new(0.3, 0.4, 0.5).with_alpha(0.7))
            .with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        let props = stack_props(stack);

        assert!(props.contains(&(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(0.3, 0.4, 0.5, 0.7)),
        )));
    }
}
