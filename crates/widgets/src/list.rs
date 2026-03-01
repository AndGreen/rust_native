use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

pub struct ListView {
    children: Vec<View>,
}

impl ListView {
    pub fn from_iterator<I, F, Item>(items: I, builder: F) -> Self
    where
        I: IntoIterator<Item = Item>,
        F: Fn(Item) -> View,
    {
        let children = items.into_iter().map(builder).collect();
        Self { children }
    }
}

struct ListElement {
    len: usize,
}

impl WidgetElement for ListElement {
    fn name(&self) -> &'static str {
        "List"
    }

    fn describe(&self) -> String {
        format!("List(len: {})", self.len)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntoView for ListView {
    fn into_view(self) -> View {
        let len = self.children.len();
        View::new(ListElement { len }, self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ItemElement;

    impl WidgetElement for ItemElement {
        fn name(&self) -> &'static str {
            "Item"
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn list_from_iterator_builds_list_view_with_children() {
        let list = ListView::from_iterator([1, 2, 3], |_| View::new(ItemElement, Vec::new()));
        let view = list.into_view();

        assert_eq!(view.element().name(), "List");
        assert_eq!(view.children().len(), 3);
        assert!(view
            .children()
            .iter()
            .all(|child| child.element().name() == "Item"));
    }
}
