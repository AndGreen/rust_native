use std::sync::Arc;

use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

use crate::color::Color;
use crate::font::Font;

pub type InputAction = Arc<dyn Fn(String) + Send + Sync>;
pub type FocusChangeAction = Arc<dyn Fn(bool) + Send + Sync>;

#[derive(Clone)]
pub struct InputView {
    value: String,
    font: Option<Font>,
    color: Option<Color>,
    background: Option<Color>,
    corner_radius: Option<f32>,
    enabled: bool,
    focused: bool,
    on_input: Option<InputAction>,
    on_focus_change: Option<FocusChangeAction>,
}

impl InputView {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            font: None,
            color: None,
            background: None,
            corner_radius: None,
            enabled: true,
            focused: false,
            on_input: None,
            on_focus_change: None,
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

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_input<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_input = Some(Arc::new(handler));
        self
    }

    pub fn on_focus_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_focus_change = Some(Arc::new(handler));
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn font_value(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    pub fn color_value(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    pub fn background_value(&self) -> Option<&Color> {
        self.background.as_ref()
    }

    pub fn corner_radius_value(&self) -> Option<f32> {
        self.corner_radius
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn input_action(&self) -> Option<&InputAction> {
        self.on_input.as_ref()
    }

    pub fn focus_change_action(&self) -> Option<&FocusChangeAction> {
        self.on_focus_change.as_ref()
    }
}

impl WidgetElement for InputView {
    fn name(&self) -> &'static str {
        "Input"
    }

    fn describe(&self) -> String {
        let mut description = format!("Input(\"{}\")", self.value);
        if let Some(font) = &self.font {
            description.push_str(&format!("[size: {} weight: {:?}]", font.size, font.weight));
        }
        if let Some(color) = &self.color {
            description.push_str(&format!(
                "[color: {:.2},{:.2},{:.2}]",
                color.r, color.g, color.b
            ));
        }
        if let Some(color) = &self.background {
            description.push_str(&format!(
                "[background: {:.2},{:.2},{:.2}]",
                color.r, color.g, color.b
            ));
        }
        if let Some(radius) = self.corner_radius {
            description.push_str(&format!("[corner_radius: {:.1}]", radius));
        }
        if self.focused {
            description.push_str("[focused]");
        }
        if !self.enabled {
            description.push_str("[disabled]");
        }
        description
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntoView for InputView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontWeight;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn input_builder_sets_visual_state_and_callbacks() {
        let input_calls = Arc::new(AtomicUsize::new(0));
        let input_calls_for_handler = Arc::clone(&input_calls);
        let focus_calls = Arc::new(AtomicUsize::new(0));
        let focus_calls_for_handler = Arc::clone(&focus_calls);

        let input = InputView::new("alex")
            .font(Font::bold(18.0))
            .foreground(Color::new(0.1, 0.2, 0.3))
            .background(Color::new(0.9, 0.8, 0.7))
            .corner_radius(10.0)
            .focused(true)
            .enabled(false)
            .on_input(move |_| {
                input_calls_for_handler.fetch_add(1, Ordering::Relaxed);
            })
            .on_focus_change(move |_| {
                focus_calls_for_handler.fetch_add(1, Ordering::Relaxed);
            });

        assert_eq!(input.value(), "alex");
        assert_eq!(input.corner_radius_value(), Some(10.0));
        assert!(input.is_focused());
        assert!(!input.is_enabled());
        assert!(matches!(
            input.font_value().map(|font| font.weight.clone()),
            Some(FontWeight::Bold)
        ));

        input.input_action().expect("input callback should be set")("next".to_string());
        input
            .focus_change_action()
            .expect("focus callback should be set")(true);

        assert_eq!(input_calls.load(Ordering::Relaxed), 1);
        assert_eq!(focus_calls.load(Ordering::Relaxed), 1);
    }
}
