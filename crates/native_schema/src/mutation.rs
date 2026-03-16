use serde::{Deserialize, Serialize};

pub type UiNodeId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProtocolVersion {
    #[default]
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Stack,
    Container,
    SafeArea,
    Text,
    Button,
    Image,
    List,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Leading,
    Center,
    Trailing,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Regular,
    SemiBold,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorValue {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorValue {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineStyle {
    pub width: f32,
    pub color: ColorValue,
}

impl LineStyle {
    pub const fn new(width: f32, color: ColorValue) -> Self {
        Self { width, color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub color: ColorValue,
    pub radius: f32,
    pub offset: crate::PointValue,
}

impl ShadowStyle {
    pub const fn new(color: ColorValue, radius: f32, offset: crate::PointValue) -> Self {
        Self {
            color,
            radius,
            offset,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Tap,
    TextInput,
    FocusChanged,
    Scroll,
    Appear,
    Disappear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PropKey {
    Axis,
    Spacing,
    Padding,
    Alignment,
    JustifyContent,
    SafeAreaEdges,
    Color,
    BackgroundColor,
    Opacity,
    FontSize,
    FontWeight,
    CornerRadius,
    CornerRadii,
    FullRound,
    Border,
    Stroke,
    Shadow,
    Offset,
    Source,
    Enabled,
    Focused,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    FlexGrow,
    FlexShrink,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropValue {
    String(String),
    Bool(bool),
    Float(f32),
    Color(ColorValue),
    Axis(crate::Axis),
    Alignment(crate::Alignment),
    JustifyContent(crate::JustifyContent),
    SafeAreaEdges(crate::SafeAreaEdges),
    FontWeight(crate::FontWeight),
    Insets(crate::EdgeInsets),
    Dimension(crate::DimensionValue),
    CornerRadii(crate::CornerRadii),
    LineStyle(crate::LineStyle),
    Shadow(crate::ShadowStyle),
    Point(crate::PointValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mutation {
    CreateNode {
        id: UiNodeId,
        kind: ElementKind,
    },
    CreateTextNode {
        id: UiNodeId,
        text: String,
    },
    SetText {
        id: UiNodeId,
        text: String,
    },
    SetProp {
        id: UiNodeId,
        key: PropKey,
        value: PropValue,
    },
    UnsetProp {
        id: UiNodeId,
        key: PropKey,
    },
    InsertChild {
        parent: UiNodeId,
        child: UiNodeId,
        index: u32,
    },
    MoveNode {
        id: UiNodeId,
        new_parent: UiNodeId,
        index: u32,
    },
    ReplaceNode {
        old: UiNodeId,
        new_id: UiNodeId,
        kind: ElementKind,
    },
    RemoveNode {
        id: UiNodeId,
    },
    AttachEventListener {
        id: UiNodeId,
        event: EventKind,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prop_keys_cover_layout_and_visual_vocabulary() {
        let keys = [
            PropKey::Axis,
            PropKey::Spacing,
            PropKey::Padding,
            PropKey::Alignment,
            PropKey::JustifyContent,
            PropKey::SafeAreaEdges,
            PropKey::Color,
            PropKey::BackgroundColor,
            PropKey::Opacity,
            PropKey::FontSize,
            PropKey::FontWeight,
            PropKey::CornerRadius,
            PropKey::CornerRadii,
            PropKey::FullRound,
            PropKey::Border,
            PropKey::Stroke,
            PropKey::Shadow,
            PropKey::Offset,
            PropKey::Source,
            PropKey::Enabled,
            PropKey::Focused,
            PropKey::Width,
            PropKey::Height,
            PropKey::MinWidth,
            PropKey::MinHeight,
            PropKey::MaxWidth,
            PropKey::MaxHeight,
            PropKey::FlexGrow,
            PropKey::FlexShrink,
        ];

        assert_eq!(keys.len(), 29);
    }

    #[test]
    fn create_text_node_preserves_payload() {
        let mutation = Mutation::CreateTextNode {
            id: 7,
            text: "Count: 1".to_string(),
        };

        match mutation {
            Mutation::CreateTextNode { id, text } => {
                assert_eq!(id, 7);
                assert_eq!(text, "Count: 1");
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn set_prop_preserves_typed_value() {
        let mutation = Mutation::SetProp {
            id: 3,
            key: PropKey::Padding,
            value: PropValue::Insets(crate::EdgeInsets::all(16.0)),
        };

        match mutation {
            Mutation::SetProp { id, key, value } => {
                assert_eq!(id, 3);
                assert_eq!(key, PropKey::Padding);
                assert_eq!(value, PropValue::Insets(crate::EdgeInsets::all(16.0)));
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn unset_prop_preserves_target_key() {
        let mutation = Mutation::UnsetProp {
            id: 9,
            key: PropKey::Shadow,
        };

        match mutation {
            Mutation::UnsetProp { id, key } => {
                assert_eq!(id, 9);
                assert_eq!(key, PropKey::Shadow);
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }
}
