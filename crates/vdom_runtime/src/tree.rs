use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use mf_core::{Fragment, View};
use mf_widgets::button::ButtonAction;
use mf_widgets::button::ButtonView;
use mf_widgets::container::Container;
use mf_widgets::image::ImageView;
use mf_widgets::input::{FocusChangeAction, InputAction, InputView};
use mf_widgets::layout::{Alignment as WidgetAlignment, Axis as WidgetAxis, StackElement};
use mf_widgets::safe_area::SafeArea;
use mf_widgets::text::TextView;
use native_schema::{
    Alignment, Axis, ColorValue, DimensionValue, EdgeInsets, ElementKind, FontWeight,
    JustifyContent, PropKey, PropValue, SafeAreaEdges, UiNodeId,
};

#[derive(Clone)]
pub(crate) struct CanonicalNode {
    pub(crate) id: UiNodeId,
    pub(crate) descriptor: NodeDescriptor,
    pub(crate) props: PropMap,
    pub(crate) text: Option<String>,
    pub(crate) tap_handler: Option<ButtonAction>,
    pub(crate) input_handler: Option<InputAction>,
    pub(crate) focus_change_handler: Option<FocusChangeAction>,
    pub(crate) children: Vec<CanonicalNode>,
}

pub(crate) type PropMap = IndexMap<PropKey, PropValue>;

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
        } else if view.element().as_any().is::<Container>() {
            Self::Element(ElementKind::Container)
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

    if let Some(container) = view.element().as_any().downcast_ref::<Container>() {
        return CanonicalNode {
            id,
            descriptor: NodeDescriptor::Element(ElementKind::Container),
            props: container_props(container),
            text: None,
            tap_handler: None,
            input_handler: None,
            focus_change_handler: None,
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
        props: PropMap::new(),
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
    node.props.get(&key)
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

fn text_props(text: &TextView) -> PropMap {
    let mut props = PropMap::new();
    if let Some(color) = text.color_value() {
        props.insert(
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    if let Some(font) = text.font_value() {
        props.insert(PropKey::FontSize, PropValue::Float(font.size));
        props.insert(
            PropKey::FontWeight,
            PropValue::FontWeight(match font.weight {
                mf_widgets::FontWeight::Regular => FontWeight::Regular,
                mf_widgets::FontWeight::SemiBold => FontWeight::SemiBold,
                mf_widgets::FontWeight::Bold => FontWeight::Bold,
            }),
        );
    }
    props
}

fn button_props(button: &ButtonView) -> PropMap {
    let mut props = PropMap::new();
    if let Some(color) = button.color_value() {
        props.insert(
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    if let Some(color) = button.background_value() {
        props.insert(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    if let Some(radius) = button.corner_radius_value() {
        props.insert(PropKey::CornerRadius, PropValue::Float(radius));
    }
    if !button.is_enabled() {
        props.insert(PropKey::Enabled, PropValue::Bool(false));
    }
    props
}

fn image_props(image: &ImageView) -> PropMap {
    let mut props = PropMap::new();
    props.insert(
        PropKey::Source,
        PropValue::String(image.source().to_string()),
    );
    let (width, height) = image.dimensions();
    if let Some(width) = width {
        props.insert(
            PropKey::Width,
            PropValue::Dimension(DimensionValue::Points(width)),
        );
    }
    if let Some(height) = height {
        props.insert(
            PropKey::Height,
            PropValue::Dimension(DimensionValue::Points(height)),
        );
    }
    if let Some(radius) = image.corner_radius_value() {
        props.insert(PropKey::CornerRadius, PropValue::Float(radius));
    }
    props
}

fn container_props(container: &Container) -> PropMap {
    let mut props = PropMap::new();
    props.insert(
        PropKey::Padding,
        PropValue::Insets(container.padding_value()),
    );
    if let Some(width) = container.width_value() {
        props.insert(
            PropKey::Width,
            PropValue::Dimension(DimensionValue::Points(width)),
        );
    }
    if let Some(height) = container.height_value() {
        props.insert(
            PropKey::Height,
            PropValue::Dimension(DimensionValue::Points(height)),
        );
    }
    if let Some(min_width) = container.min_width_value() {
        props.insert(
            PropKey::MinWidth,
            PropValue::Dimension(DimensionValue::Points(min_width)),
        );
    }
    if let Some(min_height) = container.min_height_value() {
        props.insert(
            PropKey::MinHeight,
            PropValue::Dimension(DimensionValue::Points(min_height)),
        );
    }
    if let Some(max_width) = container.max_width_value() {
        props.insert(
            PropKey::MaxWidth,
            PropValue::Dimension(DimensionValue::Points(max_width)),
        );
    }
    if let Some(max_height) = container.max_height_value() {
        props.insert(
            PropKey::MaxHeight,
            PropValue::Dimension(DimensionValue::Points(max_height)),
        );
    }
    if let Some(color) = container.background_value() {
        props.insert(PropKey::BackgroundColor, PropValue::Color((*color).into()));
    }
    if let Some(opacity) = container.opacity_value() {
        props.insert(PropKey::Opacity, PropValue::Float(opacity));
    }
    if let Some(border) = container.border_value() {
        props.insert(PropKey::Border, PropValue::LineStyle(border));
    }
    if let Some(stroke) = container.stroke_value() {
        props.insert(PropKey::Stroke, PropValue::LineStyle(stroke));
    }
    if let Some(radius) = container.corner_radius_value() {
        props.insert(PropKey::CornerRadius, PropValue::Float(radius));
    }
    if let Some(radii) = container.corner_radii_value() {
        props.insert(PropKey::CornerRadii, PropValue::CornerRadii(radii));
    }
    if container.full_round_value() {
        props.insert(PropKey::FullRound, PropValue::Bool(true));
    }
    if let Some(shadow) = container.shadow_value() {
        props.insert(PropKey::Shadow, PropValue::Shadow(shadow));
    }
    if let Some(offset) = container.offset_value() {
        props.insert(PropKey::Offset, PropValue::Point(offset));
    }
    props
}

fn input_props(input: &InputView) -> PropMap {
    let mut props = PropMap::new();
    if let Some(color) = input.color_value() {
        props.insert(
            PropKey::Color,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    if let Some(color) = input.background_value() {
        props.insert(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    if let Some(font) = input.font_value() {
        props.insert(PropKey::FontSize, PropValue::Float(font.size));
        props.insert(
            PropKey::FontWeight,
            PropValue::FontWeight(match font.weight {
                mf_widgets::FontWeight::Regular => FontWeight::Regular,
                mf_widgets::FontWeight::SemiBold => FontWeight::SemiBold,
                mf_widgets::FontWeight::Bold => FontWeight::Bold,
            }),
        );
    }
    if let Some(radius) = input.corner_radius_value() {
        props.insert(PropKey::CornerRadius, PropValue::Float(radius));
    }
    if !input.is_enabled() {
        props.insert(PropKey::Enabled, PropValue::Bool(false));
    }
    if input.is_focused() {
        props.insert(PropKey::Focused, PropValue::Bool(true));
    }
    props
}

fn stack_props(stack: &StackElement) -> PropMap {
    let mut props = PropMap::new();
    props.insert(
        PropKey::Axis,
        PropValue::Axis(match stack.axis() {
            WidgetAxis::Horizontal => Axis::Horizontal,
            WidgetAxis::Vertical => Axis::Vertical,
        }),
    );
    props.insert(PropKey::Spacing, PropValue::Float(stack.spacing()));
    props.insert(PropKey::Padding, PropValue::Insets(stack.padding_value()));
    props.insert(
        PropKey::Alignment,
        PropValue::Alignment(match stack.alignment() {
            WidgetAlignment::Leading => Alignment::Leading,
            WidgetAlignment::Center => Alignment::Center,
            WidgetAlignment::Trailing => Alignment::Trailing,
            WidgetAlignment::Stretch => Alignment::Stretch,
        }),
    );
    props.insert(
        PropKey::JustifyContent,
        PropValue::JustifyContent(stack.justify_content()),
    );
    if let Some(color) = stack.background_value() {
        props.insert(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    props
}

fn safe_area_props(safe_area: &SafeArea) -> PropMap {
    let mut props = PropMap::new();
    props.insert(
        PropKey::SafeAreaEdges,
        PropValue::SafeAreaEdges(match safe_area.edges_value() {
            SafeAreaEdges::Top => SafeAreaEdges::Top,
            SafeAreaEdges::TopBottom => SafeAreaEdges::TopBottom,
            SafeAreaEdges::All => SafeAreaEdges::All,
        }),
    );
    props.insert(
        PropKey::Alignment,
        PropValue::Alignment(match safe_area.alignment_value() {
            WidgetAlignment::Leading => Alignment::Leading,
            WidgetAlignment::Center => Alignment::Center,
            WidgetAlignment::Trailing => Alignment::Trailing,
            WidgetAlignment::Stretch => Alignment::Stretch,
        }),
    );
    props.insert(
        PropKey::JustifyContent,
        PropValue::JustifyContent(match safe_area.justify_content_value() {
            JustifyContent::Start => JustifyContent::Start,
            JustifyContent::Center => JustifyContent::Center,
            JustifyContent::End => JustifyContent::End,
            JustifyContent::Stretch => JustifyContent::Stretch,
        }),
    );
    props.insert(
        PropKey::Padding,
        PropValue::Insets(safe_area.padding_value()),
    );
    if let Some(color) = safe_area.background_value() {
        props.insert(
            PropKey::BackgroundColor,
            PropValue::Color(ColorValue::new(color.r, color.g, color.b, color.a)),
        );
    }
    props
}

fn list_props() -> PropMap {
    let mut props = PropMap::new();
    props.insert(PropKey::Axis, PropValue::Axis(Axis::Vertical));
    props.insert(PropKey::Spacing, PropValue::Float(0.0));
    props.insert(PropKey::Padding, PropValue::Insets(EdgeInsets::all(0.0)));
    props.insert(PropKey::Alignment, PropValue::Alignment(Alignment::Leading));
    props
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_core::WithChildren;
    use mf_widgets::{
        Button, Color, Container, EdgeInsets, HStack, Input, JustifyContent, SafeArea, VStack,
    };

    #[test]
    fn button_props_include_visual_style_and_enabled_state() {
        let button = Button("Save")
            .background(Color::new(0.1, 0.2, 0.3))
            .foreground(Color::new(0.9, 0.8, 0.7).with_alpha(0.6))
            .corner_radius(10.0)
            .enabled(false);

        let props = button_props(&button);

        assert_eq!(
            props.get(&PropKey::BackgroundColor),
            Some(&PropValue::Color(ColorValue::new(0.1, 0.2, 0.3, 1.0)))
        );
        assert_eq!(
            props.get(&PropKey::Color),
            Some(&PropValue::Color(ColorValue::new(0.9, 0.8, 0.7, 0.6)))
        );
        assert_eq!(
            props.get(&PropKey::CornerRadius),
            Some(&PropValue::Float(10.0))
        );
        assert_eq!(props.get(&PropKey::Enabled), Some(&PropValue::Bool(false)));
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

        assert_eq!(props.get(&PropKey::Focused), Some(&PropValue::Bool(true)));
        assert_eq!(props.get(&PropKey::Enabled), Some(&PropValue::Bool(false)));
        assert_eq!(
            props.get(&PropKey::CornerRadius),
            Some(&PropValue::Float(12.0))
        );
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

        assert_eq!(
            props.get(&PropKey::BackgroundColor),
            Some(&PropValue::Color(ColorValue::new(0.3, 0.4, 0.5, 0.7))),
        );
    }

    #[test]
    fn container_props_include_visual_and_layout_values() {
        let view = Container::new()
            .padding_insets(EdgeInsets::new(1.0, 2.0, 3.0, 4.0))
            .width(120.0)
            .height(44.0)
            .background(Color::new(0.2, 0.3, 0.4).with_alpha(0.7))
            .opacity(0.8)
            .border(2.0, Color::new(0.9, 0.8, 0.7))
            .stroke(1.0, Color::new(0.5, 0.6, 0.7))
            .corner_radius(12.0)
            .corner_radius_per_corner(4.0, 6.0, 8.0, 10.0)
            .full_round(true)
            .shadow(Color::new(0.0, 0.0, 0.0).with_alpha(0.3), 10.0, 2.0, 4.0)
            .offset(3.0, -1.0)
            .with_children(Vec::new());
        let container = view
            .element()
            .as_any()
            .downcast_ref::<Container>()
            .expect("container element");

        let props = container_props(container);

        assert_eq!(
            props.get(&PropKey::Padding),
            Some(&PropValue::Insets(EdgeInsets::new(1.0, 2.0, 3.0, 4.0)))
        );
        assert_eq!(
            props.get(&PropKey::CornerRadii),
            Some(&PropValue::CornerRadii(native_schema::CornerRadii::new(
                4.0, 6.0, 8.0, 10.0,
            )))
        );
        assert_eq!(props.get(&PropKey::FullRound), Some(&PropValue::Bool(true)));
        assert!(matches!(
            props.get(&PropKey::Border),
            Some(PropValue::LineStyle(_))
        ));
        assert!(matches!(
            props.get(&PropKey::Shadow),
            Some(PropValue::Shadow(_))
        ));
        assert!(matches!(
            props.get(&PropKey::Offset),
            Some(PropValue::Point(_))
        ));
    }

    #[test]
    fn vstack_default_alignment_serializes_as_stretch() {
        let view = VStack::new().with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        let props = stack_props(stack);

        assert_eq!(
            props.get(&PropKey::Alignment),
            Some(&PropValue::Alignment(Alignment::Stretch)),
        );
    }

    #[test]
    fn stack_props_include_justify_content() {
        let view = VStack::new()
            .justify_content(JustifyContent::Center)
            .with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        let props = stack_props(stack);

        assert_eq!(
            props.get(&PropKey::JustifyContent),
            Some(&PropValue::JustifyContent(JustifyContent::Center)),
        );
    }

    #[test]
    fn safe_area_props_include_layout_and_background_values() {
        let safe_area = SafeArea::new()
            .alignment(mf_widgets::Alignment::Center)
            .justify_content(JustifyContent::Center)
            .background(Color::new(0.3, 0.4, 0.5).with_alpha(0.7));

        let props = safe_area_props(&safe_area);

        assert_eq!(
            props.get(&PropKey::Alignment),
            Some(&PropValue::Alignment(Alignment::Center)),
        );
        assert_eq!(
            props.get(&PropKey::JustifyContent),
            Some(&PropValue::JustifyContent(JustifyContent::Center)),
        );
        assert_eq!(
            props.get(&PropKey::BackgroundColor),
            Some(&PropValue::Color(ColorValue::new(0.3, 0.4, 0.5, 0.7))),
        );
    }
}
