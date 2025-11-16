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
