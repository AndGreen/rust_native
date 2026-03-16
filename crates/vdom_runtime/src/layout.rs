use std::collections::HashSet;

use native_schema::{
    Alignment, Axis, EdgeInsets, ElementKind, JustifyContent, LayoutFrame, PropKey, PropValue,
    SafeAreaEdges,
};
use taffy::prelude::*;

use crate::tree::{dimension_points, float_prop, prop_value, CanonicalNode, NodeDescriptor};
use crate::types::HostSize;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const TEXT_WIDTH_FACTOR: f32 = 0.6;
const TEXT_HEIGHT_FACTOR: f32 = 1.2;
const BUTTON_HORIZONTAL_PADDING: f32 = 16.0;
const BUTTON_VERTICAL_PADDING: f32 = 10.0;
const BUTTON_MIN_HEIGHT: f32 = 32.0;
const INPUT_VERTICAL_PADDING: f32 = 12.0;
const INPUT_MIN_HEIGHT: f32 = 44.0;
const FALLBACK_IMAGE_SIZE: f32 = 44.0;

#[derive(Debug, Clone, Copy)]
struct LayoutProps {
    axis: Axis,
    spacing: f32,
    padding: EdgeInsets,
    alignment: Alignment,
    justify_content: JustifyContent,
    safe_area_edges: Option<SafeAreaEdges>,
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
            NodeDescriptor::Element(ElementKind::List)
            | NodeDescriptor::Element(ElementKind::Container) => Self {
                axis: Axis::Vertical,
                spacing: 0.0,
                padding: EdgeInsets::all(0.0),
                alignment: Alignment::Leading,
                justify_content: JustifyContent::Start,
                safe_area_edges: None,
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
                justify_content: JustifyContent::Start,
                safe_area_edges: None,
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
        if let Some(PropValue::JustifyContent(justify_content)) =
            prop_value(node, PropKey::JustifyContent)
        {
            props.justify_content = *justify_content;
        }
        if let Some(PropValue::SafeAreaEdges(edges)) = prop_value(node, PropKey::SafeAreaEdges) {
            props.safe_area_edges = Some(*edges);
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

#[derive(Debug, Clone, Copy)]
struct ParentLayoutContext {
    axis: Axis,
    alignment: Alignment,
    justify_content: JustifyContent,
}

pub(crate) fn compute_layout_frames(root: &CanonicalNode, host_size: HostSize) -> Vec<LayoutFrame> {
    let mut taffy = Taffy::new();
    let root_node = build_taffy_tree(&mut taffy, root, host_size, true, None);
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
    parent_context: Option<ParentLayoutContext>,
) -> Node {
    let props = LayoutProps::for_node(node);
    let next_parent_context = match node.descriptor {
        NodeDescriptor::Element(ElementKind::Stack)
        | NodeDescriptor::Element(ElementKind::Container)
        | NodeDescriptor::Element(ElementKind::SafeArea)
        | NodeDescriptor::Element(ElementKind::List) => Some(ParentLayoutContext {
            axis: props.axis,
            alignment: props.alignment,
            justify_content: props.justify_content,
        }),
        _ => None,
    };
    let children: Vec<Node> = node
        .children
        .iter()
        .map(|child| build_taffy_tree(taffy, child, host_size, false, next_parent_context))
        .collect();
    let style = style_for_node(node, &props, host_size, is_root, parent_context);

    if children.is_empty() {
        taffy.new_leaf(style).expect("leaf node should be created")
    } else {
        taffy
            .new_with_children(style, &children)
            .expect("container node should be created")
    }
}

fn style_for_node(
    node: &CanonicalNode,
    props: &LayoutProps,
    host_size: HostSize,
    is_root: bool,
    parent_context: Option<ParentLayoutContext>,
) -> Style {
    let mut style = Style::DEFAULT.clone();

    if is_root {
        style.size = Size {
            width: points(host_size.width),
            height: points(host_size.height),
        };
    }

    let resolved_padding = if matches!(
        node.descriptor,
        NodeDescriptor::Element(ElementKind::SafeArea)
    ) {
        props
            .safe_area_edges
            .unwrap_or(SafeAreaEdges::TopBottom)
            .apply_to(host_size.safe_area)
    } else {
        props.padding
    };

    style.padding = Rect {
        left: points(resolved_padding.left),
        right: points(resolved_padding.right),
        top: points(resolved_padding.top),
        bottom: points(resolved_padding.bottom),
    };
    style.align_items = Some(map_alignment(props.alignment));
    style.justify_content = Some(map_justify_content(props.justify_content));

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
        | NodeDescriptor::Element(ElementKind::Container)
        | NodeDescriptor::Element(ElementKind::SafeArea)
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
            apply_main_axis_stretch_if_needed(&mut style, props, parent_context);
        }
        NodeDescriptor::Text => {
            let (width, height) = intrinsic_text_size(node);
            if is_root {
                style.size = Size {
                    width: points(host_size.width),
                    height: points(host_size.height),
                };
            } else {
                if props.width.is_none() && !text_should_fill_parent_width(parent_context) {
                    style.size.width = points(width);
                }
                if props.height.is_none() {
                    style.size.height = points(height);
                }
            }
        }
        NodeDescriptor::Element(ElementKind::Button) => {
            let (width, height) = intrinsic_button_size(node);
            if is_root {
                style.size = Size {
                    width: points(host_size.width),
                    height: points(host_size.height),
                };
            } else {
                if props.width.is_none() {
                    style.size.width = points(width);
                }
                if props.height.is_none() {
                    style.size.height = points(height);
                }
            }
        }
        NodeDescriptor::Element(ElementKind::Image) => {
            let (width, height) = intrinsic_image_size(props);
            if is_root {
                style.size = Size {
                    width: points(host_size.width),
                    height: points(host_size.height),
                };
            } else {
                if props.width.is_none() {
                    style.size.width = points(width);
                }
                if props.height.is_none() {
                    style.size.height = points(height);
                }
            }
        }
        NodeDescriptor::Element(ElementKind::Input) => {
            let height = intrinsic_input_height(node, props);
            if is_root {
                style.size.width = points(host_size.width);
            } else if let Some(width) = props.width {
                style.size.width = points(width);
            } else {
                // Inputs should fill the parent content box and never size to text length.
                style.size.width = percent(1.0);
            }
            style.size.height = points(if is_root { host_size.height } else { height });
        }
        NodeDescriptor::Element(_) => {}
    }

    style
}

fn text_should_fill_parent_width(parent_context: Option<ParentLayoutContext>) -> bool {
    matches!(
        parent_context,
        Some(ParentLayoutContext {
            axis: Axis::Vertical,
            alignment: Alignment::Stretch,
            ..
        })
    )
}

fn apply_main_axis_stretch_if_needed(
    style: &mut Style,
    props: &LayoutProps,
    parent_context: Option<ParentLayoutContext>,
) {
    let should_stretch = matches!(
        parent_context,
        Some(ParentLayoutContext {
            axis: Axis::Vertical,
            justify_content: JustifyContent::Stretch,
            ..
        }) if props.height.is_none()
    ) || matches!(
        parent_context,
        Some(ParentLayoutContext {
            axis: Axis::Horizontal,
            justify_content: JustifyContent::Stretch,
            ..
        }) if props.width.is_none()
    );

    if !should_stretch {
        return;
    }

    match parent_context.expect("checked above").axis {
        Axis::Vertical => style.size.height = percent(1.0),
        Axis::Horizontal => style.size.width = percent(1.0),
    }
}

fn map_alignment(alignment: Alignment) -> AlignItems {
    match alignment {
        Alignment::Leading => AlignItems::Start,
        Alignment::Center => AlignItems::Center,
        Alignment::Trailing => AlignItems::End,
        Alignment::Stretch => AlignItems::Stretch,
    }
}

fn map_justify_content(justify_content: JustifyContent) -> taffy::style::JustifyContent {
    match justify_content {
        JustifyContent::Start => taffy::style::JustifyContent::Start,
        JustifyContent::Center => taffy::style::JustifyContent::Center,
        JustifyContent::End => taffy::style::JustifyContent::End,
        JustifyContent::Stretch => taffy::style::JustifyContent::Stretch,
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

fn intrinsic_input_height(node: &CanonicalNode, props: &LayoutProps) -> f32 {
    let (_, text_height) = intrinsic_text_size(node);
    props
        .height
        .unwrap_or((text_height + INPUT_VERTICAL_PADDING * 2.0).max(INPUT_MIN_HEIGHT))
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
    use mf_core::{IntoView, WithChildren};
    use mf_widgets::{
        Alignment, Container, EdgeInsets, HStack, Image, Input, JustifyContent, SafeArea, Text,
        VStack,
    };
    use native_schema::{ElementKind, LayoutFrame};

    use super::{compute_layout_frames, validate_layout_frames};
    use crate::tree::{CanonicalNode, NodeDescriptor};
    use crate::types::HostSize;

    #[test]
    fn layout_frame_validation_rejects_duplicates() {
        let node = CanonicalNode {
            id: 1,
            descriptor: NodeDescriptor::Element(ElementKind::Stack),
            props: crate::tree::PropMap::new(),
            text: None,
            tap_handler: None,
            input_handler: None,
            focus_change_handler: None,
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

    #[test]
    fn input_layout_stays_within_parent_content_box() {
        let view = VStack()
            .padding(24.0)
            .alignment(Alignment::Leading)
            .with_children(vec![Input("hello").into_view()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(
                2,
                view.children().first().expect("input child"),
                Vec::new(),
            )],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("parent frame");
        let input = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("input frame");

        assert_eq!(input.x, 24.0);
        assert_eq!(input.width, parent.width - 48.0);
        assert!(input.height >= 44.0);
    }

    #[test]
    fn vstack_default_stretches_child_stack_to_content_width() {
        let child = HStack().with_children(Vec::new()).into_view();
        let view = VStack().padding(24.0).with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("parent frame");
        let child = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("child frame");

        assert_eq!(child.x, 24.0);
        assert_eq!(child.width, parent.width - 48.0);
    }

    #[test]
    fn hstack_default_alignment_does_not_stretch_children_vertically() {
        let child = VStack().with_children(Vec::new()).into_view();
        let view = HStack().padding(24.0).with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("parent frame");
        let child = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("child frame");

        assert_eq!(child.y, (parent.height - 48.0 - child.height) / 2.0 + 24.0);
        assert_eq!(child.height, 0.0);
    }

    #[test]
    fn text_in_default_vstack_fills_parent_content_width() {
        let child = Text("Name").into_view();
        let view = VStack().padding(24.0).with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("parent frame");
        let text = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("text frame");

        assert_eq!(text.x, 24.0);
        assert_eq!(text.width, parent.width - 48.0);
        assert!(text.height > 0.0);
    }

    #[test]
    fn text_in_leading_vstack_keeps_intrinsic_width() {
        let child = Text("Name").into_view();
        let view = VStack()
            .padding(24.0)
            .alignment(Alignment::Leading)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let text = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("text frame");

        assert_eq!(text.x, 24.0);
        assert!(text.width > 0.0);
        assert!(text.width < 100.0);
    }

    #[test]
    fn text_in_hstack_keeps_intrinsic_width() {
        let child = Text("Name").into_view();
        let view = HStack().padding(24.0).with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("parent frame");
        let text = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("text frame");

        assert!(text.y > 24.0);
        assert!(text.y < parent.height - 24.0);
        assert!(text.width > 0.0);
        assert!(text.width < 100.0);
    }

    #[test]
    fn safe_area_can_center_child_within_host_frame() {
        let child = Image("cover.png").size(40.0, 48.0).into_view();
        let view = SafeArea()
            .alignment(Alignment::Center)
            .justify_content(JustifyContent::Center)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let safe_area = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("safe area frame");
        let child = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("child frame");

        assert_eq!(safe_area.height, 844.0);
        assert_eq!(child.x, (390.0 - child.width) / 2.0);
        assert_eq!(child.y, (844.0 - child.height) / 2.0);
    }

    #[test]
    fn safe_area_can_stretch_child_along_main_axis() {
        let child = VStack().with_children(Vec::new()).into_view();
        let view = SafeArea()
            .justify_content(JustifyContent::Stretch)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let child = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("child frame");

        assert_eq!(child.y, 0.0);
        assert_eq!(child.height, 844.0);
    }

    #[test]
    fn container_can_center_child_within_padded_content_box() {
        let child = Text("Preview").into_view();
        let view = Container::new()
            .width(120.0)
            .height(80.0)
            .padding_insets(EdgeInsets::new(8.0, 12.0, 8.0, 12.0))
            .alignment(Alignment::Center)
            .justify_content(JustifyContent::Center)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let container = frames
            .iter()
            .find(|frame| frame.id == 1)
            .expect("container frame");
        let child = frames
            .iter()
            .find(|frame| frame.id == 2)
            .expect("child frame");

        let content_width = container.width - 24.0;
        let content_height = container.height - 16.0;

        assert_eq!(child.x, 12.0 + (content_width - child.width) / 2.0);
        assert_eq!(child.y, 8.0 + (content_height - child.height) / 2.0);
    }
}
