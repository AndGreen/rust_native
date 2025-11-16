use std::sync::Arc;

use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

pub type ButtonAction = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct ButtonView {
    label: String,
    on_click: Option<ButtonAction>,
}

impl ButtonView {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
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
}

impl WidgetElement for ButtonView {
    fn name(&self) -> &'static str {
        "Button"
    }

    fn describe(&self) -> String {
        format!("Button(\"{}\")", self.label)
    }
}

impl IntoView for ButtonView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}
