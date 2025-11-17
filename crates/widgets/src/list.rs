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
