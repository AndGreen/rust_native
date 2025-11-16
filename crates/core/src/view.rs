use std::fmt;
use std::sync::Arc;

/// Describes a widget node and captures sufficient information for diffing/rendering.
pub trait WidgetElement: Send + Sync {
    fn name(&self) -> &'static str;

    fn describe(&self) -> String {
        self.name().to_string()
    }
}

#[derive(Clone)]
pub struct View {
    element: Arc<dyn WidgetElement>,
    children: Vec<View>,
}

impl View {
    pub fn new<E>(element: E, children: Vec<View>) -> Self
    where
        E: WidgetElement + 'static,
    {
        Self {
            element: Arc::new(element),
            children,
        }
    }

    pub fn element(&self) -> &dyn WidgetElement {
        self.element.as_ref()
    }

    pub fn children(&self) -> &[View] {
        &self.children
    }

    pub fn into_children(self) -> Vec<View> {
        self.children
    }

    pub fn fragment(children: Vec<View>) -> Self {
        Self::new(Fragment, children)
    }
}

pub struct Fragment;

impl WidgetElement for Fragment {
    fn name(&self) -> &'static str {
        "Fragment"
    }
}

impl fmt::Debug for View {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("View")
            .field("element", &self.element().name())
            .field("children", &self.children)
            .finish()
    }
}
