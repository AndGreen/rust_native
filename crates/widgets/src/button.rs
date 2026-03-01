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

    pub fn label(&self) -> &str {
        &self.label
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
        format!("Button(\"{}\")", self.label)
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
