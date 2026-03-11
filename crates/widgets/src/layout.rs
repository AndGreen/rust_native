use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};

use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Clone)]
pub struct VStack {
    spacing: f32,
    padding: f32,
    alignment: Alignment,
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
            padding: 0.0,
            alignment: Alignment::Center,
            background: None,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
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
                background: self.background,
            },
            children,
        )
    }
}

#[derive(Clone)]
pub struct HStack {
    spacing: f32,
    padding: f32,
    alignment: Alignment,
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
            padding: 0.0,
            alignment: Alignment::Center,
            background: None,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
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
    padding: f32,
    alignment: Alignment,
    background: Option<Color>,
}

impl StackElement {
    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn spacing(&self) -> f32 {
        self.spacing
    }

    pub fn padding(&self) -> f32 {
        self.padding
    }

    pub fn alignment(&self) -> Alignment {
        self.alignment
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
            "{}(spacing: {}, padding: {}, alignment: {:?}, background: {:?})",
            self.name(),
            self.spacing,
            self.padding,
            self.alignment,
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
}
