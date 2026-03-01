use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Describes a widget node and captures sufficient information for diffing/rendering.
pub trait WidgetElement: Send + Sync {
    fn name(&self) -> &'static str;

    fn describe(&self) -> String {
        self.name().to_string()
    }

    /// Enables backend-specific downcasting to concrete widget types.
    fn as_any(&self) -> &dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestElement(&'static str);

    impl WidgetElement for TestElement {
        fn name(&self) -> &'static str {
            self.0
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn view_new_exposes_element_and_children() {
        let child = View::new(TestElement("Child"), Vec::new());
        let root = View::new(TestElement("Root"), vec![child]);

        assert_eq!(root.element().name(), "Root");
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].element().name(), "Child");
    }

    #[test]
    fn fragment_wraps_children_with_fragment_element() {
        let child = View::new(TestElement("Child"), Vec::new());
        let fragment = View::fragment(vec![child]);

        assert_eq!(fragment.element().name(), "Fragment");
        assert_eq!(fragment.children().len(), 1);
    }
}
