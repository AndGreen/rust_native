use std::sync::Arc;

use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

use crate::color::Color;

pub type ButtonAction = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct ButtonView {
    label: String,
    background: Option<Color>,
    color: Option<Color>,
    corner_radius: Option<f32>,
    enabled: bool,
    on_click: Option<ButtonAction>,
}

impl ButtonView {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            background: None,
            color: None,
            corner_radius: None,
            enabled: true,
            on_click: None,
        }
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn foreground(self, color: Color) -> Self {
        self.color(color)
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(handler));
        self
    }

    pub fn trigger(&self) {
        if let Some(action) = &self.on_click {
            action();
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn background_value(&self) -> Option<&Color> {
        self.background.as_ref()
    }

    pub fn color_value(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    pub fn corner_radius_value(&self) -> Option<f32> {
        self.corner_radius
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn action(&self) -> Option<&ButtonAction> {
        self.on_click.as_ref()
    }
}

impl WidgetElement for ButtonView {
    fn name(&self) -> &'static str {
        "Button"
    }

    fn describe(&self) -> String {
        let mut description = format!("Button(\"{}\")", self.label);
        if let Some(color) = &self.background {
            description.push_str(&format!(
                "[background: {:.2},{:.2},{:.2}]",
                color.r, color.g, color.b
            ));
        }
        if let Some(color) = &self.color {
            description.push_str(&format!(
                "[color: {:.2},{:.2},{:.2}]",
                color.r, color.g, color.b
            ));
        }
        if let Some(radius) = self.corner_radius {
            description.push_str(&format!("[corner_radius: {:.1}]", radius));
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

impl IntoView for ButtonView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn button_builder_sets_visual_style() {
        let background = Color::new(0.2, 0.3, 0.4).with_alpha(0.9);
        let foreground = Color::new(0.9, 0.8, 0.7);
        let button = ButtonView::new("Save")
            .background(background)
            .foreground(foreground)
            .corner_radius(12.0)
            .enabled(false);

        assert_eq!(button.background_value().copied().map(|c| c.r), Some(0.2));
        assert_eq!(button.background_value().copied().map(|c| c.a), Some(0.9));
        assert_eq!(button.color_value().copied().map(|c| c.g), Some(0.8));
        assert_eq!(button.corner_radius_value(), Some(12.0));
        assert!(!button.is_enabled());
    }

    #[test]
    fn trigger_invokes_registered_click_handler() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let button = ButtonView::new("Like").on_click(move || {
            calls_for_handler.fetch_add(1, Ordering::Relaxed);
        });

        button.trigger();
        button.trigger();

        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
