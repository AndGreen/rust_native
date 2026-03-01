use mf_core::dsl::IntoView;
use mf_core::view::{View, WidgetElement};

#[derive(Clone)]
pub struct ImageView {
    source: String,
    width: Option<f32>,
    height: Option<f32>,
    corner_radius: Option<f32>,
}

impl ImageView {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            width: None,
            height: None,
            corner_radius: None,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn dimensions(&self) -> (Option<f32>, Option<f32>) {
        (self.width, self.height)
    }

    pub fn corner_radius_value(&self) -> Option<f32> {
        self.corner_radius
    }
}

impl WidgetElement for ImageView {
    fn name(&self) -> &'static str {
        "Image"
    }

    fn describe(&self) -> String {
        format!("Image({})", self.source)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntoView for ImageView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_builder_sets_dimensions_and_corner_radius() {
        let view = ImageView::new("cover.jpg")
            .size(60.0, 80.0)
            .corner_radius(8.0);

        assert_eq!(view.source(), "cover.jpg");
        assert_eq!(view.dimensions(), (Some(60.0), Some(80.0)));
        assert_eq!(view.corner_radius_value(), Some(8.0));
    }
}
