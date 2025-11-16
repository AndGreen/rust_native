use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};

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
}

impl VStack {
    pub fn new() -> Self {
        Self {
            spacing: 8.0,
            padding: 0.0,
            alignment: Alignment::Center,
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
}

impl WithChildren for VStack {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(StackElement {
            axis: Axis::Vertical,
            spacing: self.spacing,
            padding: self.padding,
            alignment: self.alignment,
        }, children)
    }
}

#[derive(Clone)]
pub struct HStack {
    spacing: f32,
    padding: f32,
    alignment: Alignment,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            spacing: 8.0,
            padding: 0.0,
            alignment: Alignment::Center,
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
}

impl WithChildren for HStack {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(StackElement {
            axis: Axis::Horizontal,
            spacing: self.spacing,
            padding: self.padding,
            alignment: self.alignment,
        }, children)
    }
}

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

struct StackElement {
    axis: Axis,
    spacing: f32,
    padding: f32,
    alignment: Alignment,
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
            "{}(spacing: {}, padding: {}, alignment: {:?})",
            self.name(),
            self.spacing,
            self.padding,
            self.alignment
        )
    }
}
