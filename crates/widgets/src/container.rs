use mf_core::dsl::WithChildren;
use mf_core::view::{View, WidgetElement};
use native_schema::{CornerRadii, EdgeInsets, JustifyContent, LineStyle, PointValue, ShadowStyle};

use crate::color::Color;
use crate::layout::Alignment;

#[derive(Clone)]
pub struct Container {
    padding: EdgeInsets,
    alignment: Alignment,
    justify_content: JustifyContent,
    width: Option<f32>,
    height: Option<f32>,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    background: Option<Color>,
    opacity: Option<f32>,
    border: Option<LineStyle>,
    stroke: Option<LineStyle>,
    corner_radius: Option<f32>,
    corner_radii: Option<CornerRadii>,
    full_round: bool,
    shadow: Option<ShadowStyle>,
    offset: Option<PointValue>,
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            padding: EdgeInsets::all(0.0),
            alignment: Alignment::Leading,
            justify_content: JustifyContent::Start,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            background: None,
            opacity: None,
            border: None,
            stroke: None,
            corner_radius: None,
            corner_radii: None,
            full_round: false,
            shadow: None,
            offset: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = EdgeInsets::all(padding);
        self
    }

    pub fn padding_insets(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
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

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(LineStyle::new(width, color.into()));
        self
    }

    pub fn stroke(mut self, width: f32, color: Color) -> Self {
        self.stroke = Some(LineStyle::new(width, color.into()));
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn corner_radius_per_corner(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.corner_radii = Some(CornerRadii::new(
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        ));
        self
    }

    pub fn full_round(mut self, full_round: bool) -> Self {
        self.full_round = full_round;
        self
    }

    pub fn shadow(mut self, color: Color, radius: f32, x: f32, y: f32) -> Self {
        self.shadow = Some(ShadowStyle::new(
            color.into(),
            radius,
            PointValue::new(x, y),
        ));
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Some(PointValue::new(x, y));
        self
    }

    pub fn padding_value(&self) -> EdgeInsets {
        self.padding
    }

    pub fn alignment_value(&self) -> Alignment {
        self.alignment
    }

    pub fn justify_content_value(&self) -> JustifyContent {
        self.justify_content
    }

    pub fn width_value(&self) -> Option<f32> {
        self.width
    }

    pub fn height_value(&self) -> Option<f32> {
        self.height
    }

    pub fn min_width_value(&self) -> Option<f32> {
        self.min_width
    }

    pub fn min_height_value(&self) -> Option<f32> {
        self.min_height
    }

    pub fn max_width_value(&self) -> Option<f32> {
        self.max_width
    }

    pub fn max_height_value(&self) -> Option<f32> {
        self.max_height
    }

    pub fn background_value(&self) -> Option<&Color> {
        self.background.as_ref()
    }

    pub fn opacity_value(&self) -> Option<f32> {
        self.opacity
    }

    pub fn border_value(&self) -> Option<LineStyle> {
        self.border
    }

    pub fn stroke_value(&self) -> Option<LineStyle> {
        self.stroke
    }

    pub fn corner_radius_value(&self) -> Option<f32> {
        self.corner_radius
    }

    pub fn corner_radii_value(&self) -> Option<CornerRadii> {
        self.corner_radii
    }

    pub fn full_round_value(&self) -> bool {
        self.full_round
    }

    pub fn shadow_value(&self) -> Option<ShadowStyle> {
        self.shadow
    }

    pub fn offset_value(&self) -> Option<PointValue> {
        self.offset
    }
}

impl WidgetElement for Container {
    fn name(&self) -> &'static str {
        "Container"
    }

    fn describe(&self) -> String {
        format!(
            "Container(padding: {:?}, alignment: {:?}, justify_content: {:?}, width: {:?}, height: {:?}, background: {:?}, opacity: {:?}, full_round: {})",
            self.padding,
            self.alignment,
            self.justify_content,
            self.width,
            self.height,
            self.background,
            self.opacity,
            self.full_round
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl WithChildren for Container {
    fn with_children(self, children: Vec<View>) -> View {
        View::new(self, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_defaults_to_leading_start_alignment() {
        let container = Container::new();

        assert_eq!(container.alignment_value(), Alignment::Leading);
        assert_eq!(container.justify_content_value(), JustifyContent::Start);
    }

    #[test]
    fn container_builder_sets_visual_and_layout_props() {
        let container = Container::new()
            .width(120.0)
            .height(44.0)
            .min_width(80.0)
            .max_height(60.0)
            .padding_insets(EdgeInsets::new(4.0, 8.0, 12.0, 16.0))
            .alignment(Alignment::Center)
            .justify_content(JustifyContent::Stretch)
            .background(Color::new(0.1, 0.2, 0.3).with_alpha(0.9))
            .opacity(0.75)
            .border(2.0, Color::new(0.9, 0.8, 0.7))
            .stroke(1.0, Color::new(0.4, 0.5, 0.6))
            .corner_radius(14.0)
            .corner_radius_per_corner(4.0, 6.0, 8.0, 10.0)
            .full_round(true)
            .shadow(Color::new(0.0, 0.0, 0.0).with_alpha(0.4), 12.0, 2.0, 6.0)
            .offset(3.0, -2.0);

        assert_eq!(container.width_value(), Some(120.0));
        assert_eq!(container.height_value(), Some(44.0));
        assert_eq!(container.min_width_value(), Some(80.0));
        assert_eq!(container.max_height_value(), Some(60.0));
        assert_eq!(
            container.padding_value(),
            EdgeInsets::new(4.0, 8.0, 12.0, 16.0)
        );
        assert_eq!(container.alignment_value(), Alignment::Center);
        assert_eq!(container.justify_content_value(), JustifyContent::Stretch);
        assert_eq!(container.opacity_value(), Some(0.75));
        assert_eq!(container.corner_radius_value(), Some(14.0));
        assert_eq!(
            container.corner_radii_value(),
            Some(CornerRadii::new(4.0, 6.0, 8.0, 10.0))
        );
        assert!(container.full_round_value());
        assert_eq!(container.offset_value(), Some(PointValue::new(3.0, -2.0)));
        assert_eq!(
            container.border_value(),
            Some(LineStyle::new(2.0, Color::new(0.9, 0.8, 0.7).into()))
        );
        assert_eq!(
            container.stroke_value(),
            Some(LineStyle::new(1.0, Color::new(0.4, 0.5, 0.6).into()))
        );
    }
}
