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

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn font_value(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    pub fn color_value(&self) -> Option<&Color> {
        self.color.as_ref()
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
            description.push_str(&format!(
                "[color: {:.2},{:.2},{:.2}]",
                color.r, color.g, color.b
            ));
        }
        description
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntoView for TextView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontWeight;

    #[test]
    fn text_builder_sets_content_font_and_color() {
        let color = Color::new(0.2, 0.3, 0.4).with_alpha(0.9);
        let view = TextView::new("hello")
            .font(Font::semibold(18.0))
            .color(color);

        assert_eq!(view.content(), "hello");

        let font = view.font_value().expect("font should be set");
        assert_eq!(font.size, 18.0);
        assert!(matches!(font.weight, FontWeight::SemiBold));

        let actual_color = view.color_value().expect("color should be set");
        assert_eq!(actual_color.r, 0.2);
        assert_eq!(actual_color.g, 0.3);
        assert_eq!(actual_color.b, 0.4);
        assert_eq!(actual_color.a, 0.9);
    }
}
