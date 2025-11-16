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
}

impl WidgetElement for ImageView {
    fn name(&self) -> &'static str {
        "Image"
    }

    fn describe(&self) -> String {
        format!("Image({})", self.source)
    }
}

impl IntoView for ImageView {
    fn into_view(self) -> View {
        View::new(self, Vec::new())
    }
}
