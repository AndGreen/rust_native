use std::collections::HashSet;

use native_schema::{
    Alignment, Axis, EdgeInsets, ElementKind, LayoutFrame, PropKey, PropValue, SafeAreaEdges,
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
            NodeDescriptor::Element(ElementKind::List) => Self {
                axis: Axis::Vertical,
                spacing: 0.0,
                padding: EdgeInsets::all(0.0),
                alignment: Alignment::Leading,
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

pub(crate) fn compute_layout_frames(root: &CanonicalNode, host_size: HostSize) -> Vec<LayoutFrame> {
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
    if !matches!(
        node.descriptor,
        NodeDescriptor::Element(ElementKind::SafeArea)
    ) {
        style.align_items = Some(map_alignment(props.alignment));
    }

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
        NodeDescriptor::Element(ElementKind::Input) => {
            let height = intrinsic_input_height(node, &props);
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

fn map_alignment(alignment: Alignment) -> AlignItems {
    match alignment {
        Alignment::Leading => AlignItems::Start,
        Alignment::Center => AlignItems::Center,
        Alignment::Trailing => AlignItems::End,
        Alignment::Stretch => AlignItems::Stretch,
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
    use mf_widgets::{Alignment, HStack, Input, VStack};
    use native_schema::{ElementKind, LayoutFrame};

    use super::{compute_layout_frames, validate_layout_frames};
    use crate::tree::{CanonicalNode, NodeDescriptor};
    use crate::types::HostSize;

    #[test]
    fn layout_frame_validation_rejects_duplicates() {
        let node = CanonicalNode {
            id: 1,
            descriptor: NodeDescriptor::Element(ElementKind::Stack),
            props: Vec::new(),
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
        let root = crate::tree::canonicalize_view(1, &view, vec![crate::tree::canonicalize_view(
            2,
            view.children().first().expect("input child"),
            Vec::new(),
        )]);

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames.iter().find(|frame| frame.id == 1).expect("parent frame");
        let input = frames.iter().find(|frame| frame.id == 2).expect("input frame");

        assert_eq!(input.x, 24.0);
        assert_eq!(input.width, parent.width - 48.0);
        assert!(input.height >= 44.0);
    }

    #[test]
    fn vstack_default_stretches_child_stack_to_content_width() {
        let child = HStack().with_children(Vec::new()).into_view();
        let view = VStack()
            .padding(24.0)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames.iter().find(|frame| frame.id == 1).expect("parent frame");
        let child = frames.iter().find(|frame| frame.id == 2).expect("child frame");

        assert_eq!(child.x, 24.0);
        assert_eq!(child.width, parent.width - 48.0);
    }

    #[test]
    fn hstack_default_alignment_does_not_stretch_children_vertically() {
        let child = VStack().with_children(Vec::new()).into_view();
        let view = HStack()
            .padding(24.0)
            .with_children(vec![child.clone()]);
        let root = crate::tree::canonicalize_view(
            1,
            &view,
            vec![crate::tree::canonicalize_view(2, &child, Vec::new())],
        );

        let frames = compute_layout_frames(&root, HostSize::new(390.0, 844.0));
        let parent = frames.iter().find(|frame| frame.id == 1).expect("parent frame");
        let child = frames.iter().find(|frame| frame.id == 2).expect("child frame");

        assert_eq!(child.y, (parent.height - 48.0 - child.height) / 2.0 + 24.0);
        assert_eq!(child.height, 0.0);
    }
}
