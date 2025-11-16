use crate::view::View;

/// Converts a widget or builder into a concrete [`View`].
pub trait IntoView {
    fn into_view(self) -> View;
}

impl IntoView for View {
    fn into_view(self) -> View {
        self
    }
}

/// Trait implemented by container widgets that may host child views.
pub trait WithChildren: Sized {
    fn with_children(self, children: Vec<View>) -> View;
}

impl<T> IntoView for T
where
    T: WithChildren,
{
    fn into_view(self) -> View {
        self.with_children(Vec::new())
    }
}
