use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};
use native_schema::SafeAreaEdges;

#[derive(Clone)]
pub struct SafeArea {
    edges: SafeAreaEdges,
}

impl Default for SafeArea {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeArea {
    pub fn new() -> Self {
        Self {
            edges: SafeAreaEdges::TopBottom,
        }
    }

    pub fn edges(mut self, edges: SafeAreaEdges) -> Self {
        self.edges = edges;
        self
    }

    pub fn edges_value(&self) -> SafeAreaEdges {
        self.edges
    }
}

impl WithChildren for SafeArea {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(self, children)
    }
}

impl WidgetElement for SafeArea {
    fn name(&self) -> &'static str {
        "SafeArea"
    }

    fn describe(&self) -> String {
        format!("SafeArea(edges: {:?})", self.edges)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::SafeArea;
    use native_schema::SafeAreaEdges;

    #[test]
    fn safe_area_defaults_to_top_bottom_edges() {
        assert_eq!(SafeArea::new().edges_value(), SafeAreaEdges::TopBottom);
    }

    #[test]
    fn safe_area_allows_edge_override() {
        assert_eq!(
            SafeArea::new().edges(SafeAreaEdges::All).edges_value(),
            SafeAreaEdges::All
        );
    }
}
