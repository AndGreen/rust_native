use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

use crate::color::Color;
use crate::font::Font;

#[derive(Clone)]
pub struct TextView {
    content: String,
    font: Option<Font>,
    color: Option<Color>,
}

impl TextView {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font: None,
            color: None,
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn foreground(self, color: Color) -> Self {
        self.color(color)
    }
}

impl WidgetElement for TextView {
    fn name(&self) -> &'static str {
        "Text"
    }

    fn describe(&self) -> String {
        let mut description = format!("Text(\"{}\")", self.content);
        if let Some(font) = &self.font {
            description.push_str(&format!("[size: {} weight: {:?}]", font.size, font.weight));
        }
        if let Some(color) = &self.color {
            description.push_str(&format!("[color: {:.2},{:.2},{:.2}]", color.r, color.g, color.b));
        }
        description
    }
}

impl IntoView for TextView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}
