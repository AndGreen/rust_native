use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};
use native_schema::{EdgeInsets, JustifyContent};

use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Leading,
    Center,
    Trailing,
    Stretch,
}

#[derive(Clone)]
pub struct VStack {
    spacing: f32,
    padding: EdgeInsets,
    alignment: Alignment,
    justify_content: JustifyContent,
    background: Option<Color>,
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl VStack {
    pub fn new() -> Self {
        Self {
            spacing: 8.0,
            padding: EdgeInsets::all(0.0),
            alignment: Alignment::Stretch,
            justify_content: JustifyContent::Start,
            background: None,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = EdgeInsets::all(padding);
        self
    }

    pub fn padding_insets(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl WithChildren for VStack {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(
            StackElement {
                axis: Axis::Vertical,
                spacing: self.spacing,
                padding: self.padding,
                alignment: self.alignment,
                justify_content: self.justify_content,
                background: self.background,
            },
            children,
        )
    }
}

#[derive(Clone)]
pub struct HStack {
    spacing: f32,
    padding: EdgeInsets,
    alignment: Alignment,
    justify_content: JustifyContent,
    background: Option<Color>,
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl HStack {
    pub fn new() -> Self {
        Self {
            spacing: 8.0,
            padding: EdgeInsets::all(0.0),
            alignment: Alignment::Center,
            justify_content: JustifyContent::Start,
            background: None,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = EdgeInsets::all(padding);
        self
    }

    pub fn padding_insets(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl WithChildren for HStack {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(
            StackElement {
                axis: Axis::Horizontal,
                spacing: self.spacing,
                padding: self.padding,
                alignment: self.alignment,
                justify_content: self.justify_content,
                background: self.background,
            },
            children,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

pub struct StackElement {
    axis: Axis,
    spacing: f32,
    padding: EdgeInsets,
    alignment: Alignment,
    justify_content: JustifyContent,
    background: Option<Color>,
}

impl StackElement {
    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn spacing(&self) -> f32 {
        self.spacing
    }

    pub fn padding_value(&self) -> EdgeInsets {
        self.padding
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub fn justify_content(&self) -> JustifyContent {
        self.justify_content
    }

    pub fn background_value(&self) -> Option<&Color> {
        self.background.as_ref()
    }
}

impl WidgetElement for StackElement {
    fn name(&self) -> &'static str {
        match self.axis {
            Axis::Horizontal => "HStack",
            Axis::Vertical => "VStack",
        }
    }

    fn describe(&self) -> String {
        format!(
            "{}(spacing: {}, padding: {:?}, alignment: {:?}, justify_content: {:?}, background: {:?})",
            self.name(),
            self.spacing,
            self.padding,
            self.alignment,
            self.justify_content,
            self.background
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vstack_defaults_to_stretch_alignment() {
        let view = VStack::new().with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert!(matches!(stack.alignment(), Alignment::Stretch));
        assert_eq!(stack.justify_content(), JustifyContent::Start);
    }

    #[test]
    fn hstack_keeps_center_alignment_by_default() {
        let view = HStack::new().with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert!(matches!(stack.alignment(), Alignment::Center));
        assert_eq!(stack.justify_content(), JustifyContent::Start);
    }

    #[test]
    fn vstack_background_builder_preserves_color() {
        let color = Color::new(0.2, 0.3, 0.4).with_alpha(0.8);
        let view = VStack::new().background(color).with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert_eq!(stack.background_value(), Some(&color));
    }

    #[test]
    fn stack_padding_insets_preserve_each_side() {
        let view = HStack::new()
            .padding_insets(EdgeInsets::new(4.0, 8.0, 12.0, 16.0))
            .with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert_eq!(stack.padding_value(), EdgeInsets::new(4.0, 8.0, 12.0, 16.0));
    }

    #[test]
    fn stack_justify_content_builder_preserves_value() {
        let view = VStack::new()
            .justify_content(JustifyContent::Center)
            .with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert_eq!(stack.justify_content(), JustifyContent::Center);
    }

    #[test]
    fn stack_justify_content_supports_stretch() {
        let view = HStack::new()
            .justify_content(JustifyContent::Stretch)
            .with_children(Vec::new());
        let stack = view
            .element()
            .as_any()
            .downcast_ref::<StackElement>()
            .expect("stack element");

        assert_eq!(stack.justify_content(), JustifyContent::Stretch);
    }
}
