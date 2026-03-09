pub type UiNodeId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProtocolVersion {
    #[default]
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Stack,
    SafeArea,
    Text,
    Button,
    Image,
    List,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    SemiBold,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Tap,
    TextInput,
    Scroll,
    Appear,
    Disappear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropKey {
    Axis,
    Spacing,
    Padding,
    Alignment,
    SafeAreaEdges,
    Color,
    BackgroundColor,
    FontSize,
    FontWeight,
    CornerRadius,
    Source,
    Enabled,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    FlexGrow,
    FlexShrink,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    String(String),
    Bool(bool),
    Float(f32),
    Color(ColorValue),
    Axis(crate::Axis),
    Alignment(crate::Alignment),
    SafeAreaEdges(crate::SafeAreaEdges),
    FontWeight(crate::FontWeight),
    Insets(crate::EdgeInsets),
    Dimension(crate::DimensionValue),
}

#[derive(Debug, Clone, PartialEq)]
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
            PropKey::SafeAreaEdges,
            PropKey::Color,
            PropKey::BackgroundColor,
            PropKey::FontSize,
            PropKey::FontWeight,
            PropKey::CornerRadius,
            PropKey::Source,
            PropKey::Enabled,
            PropKey::Width,
            PropKey::Height,
            PropKey::MinWidth,
            PropKey::MinHeight,
            PropKey::MaxWidth,
            PropKey::MaxHeight,
            PropKey::FlexGrow,
            PropKey::FlexShrink,
        ];

        assert_eq!(keys.len(), 20);
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
}
