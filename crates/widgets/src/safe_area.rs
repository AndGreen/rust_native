use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};
use native_schema::{JustifyContent, SafeAreaEdges};

use crate::color::Color;
use crate::layout::Alignment;

#[derive(Clone)]
pub struct SafeArea {
    edges: SafeAreaEdges,
    alignment: Alignment,
    justify_content: JustifyContent,
    background: Option<Color>,
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
            alignment: Alignment::Stretch,
            justify_content: JustifyContent::Start,
            background: None,
        }
    }

    pub fn edges(mut self, edges: SafeAreaEdges) -> Self {
        self.edges = edges;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.justify_content = justify_content;
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    pub fn edges_value(&self) -> SafeAreaEdges {
        self.edges
    }

    pub fn alignment_value(&self) -> Alignment {
        self.alignment
    }

    pub fn justify_content_value(&self) -> JustifyContent {
        self.justify_content
    }

    pub fn background_value(&self) -> Option<&Color> {
        self.background.as_ref()
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
        format!(
            "SafeArea(edges: {:?}, alignment: {:?}, justify_content: {:?}, background: {:?})",
            self.edges, self.alignment, self.justify_content, self.background
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::SafeArea;
    use crate::color::Color;
    use crate::layout::Alignment;
    use native_schema::{JustifyContent, SafeAreaEdges};

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

    #[test]
    fn safe_area_supports_layout_and_background_props() {
        let color = Color::new(0.1, 0.2, 0.3).with_alpha(0.8);
        let safe_area = SafeArea::new()
            .alignment(Alignment::Center)
            .justify_content(JustifyContent::Stretch)
            .background(color);

        assert_eq!(safe_area.alignment_value(), Alignment::Center);
        assert_eq!(safe_area.justify_content_value(), JustifyContent::Stretch);
        assert_eq!(safe_area.background_value(), Some(&color));
    }
}
