use crate::view::View;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch {
    Replace,
}

#[derive(Default)]
pub struct DiffEngine;

impl DiffEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn diff(&self, previous: Option<&View>, next: &View) -> Vec<Patch> {
        if previous.is_none() {
            vec![Patch::Replace]
        } else if Self::equals(previous.unwrap(), next) {
            Vec::new()
        } else {
            vec![Patch::Replace]
        }
    }

    fn equals(a: &View, b: &View) -> bool {
        if a.element().name() != b.element().name() {
            return false;
        }
        let a_children = a.children();
        let b_children = b.children();
        if a_children.len() != b_children.len() {
            return false;
        }
        a_children.iter().zip(b_children.iter()).all(|(x, y)| Self::equals(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::WidgetElement;

    struct Element(&'static str);

    impl WidgetElement for Element {
        fn name(&self) -> &'static str {
            self.0
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn node(name: &'static str, children: Vec<View>) -> View {
        View::new(Element(name), children)
    }

    #[test]
    fn returns_replace_patch_on_initial_mount() {
        let diff = DiffEngine::new();
        let next = node("Root", Vec::new());

        assert_eq!(diff.diff(None, &next), vec![Patch::Replace]);
    }

    #[test]
    fn returns_no_patches_for_equal_tree_shape() {
        let diff = DiffEngine::new();
        let previous = node("Root", vec![node("Child", Vec::new())]);
        let next = node("Root", vec![node("Child", Vec::new())]);

        assert!(diff.diff(Some(&previous), &next).is_empty());
    }

    #[test]
    fn returns_replace_when_tree_shape_changes() {
        let diff = DiffEngine::new();
        let previous = node("Root", vec![node("Child", Vec::new())]);
        let next = node("Root", vec![node("AnotherChild", Vec::new())]);

        assert_eq!(diff.diff(Some(&previous), &next), vec![Patch::Replace]);
    }
}
