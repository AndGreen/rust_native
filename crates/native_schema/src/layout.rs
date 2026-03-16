use crate::UiNodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeAreaEdges {
    Top,
    TopBottom,
    All,
}

impl SafeAreaEdges {
    pub fn apply_to(self, insets: EdgeInsets) -> EdgeInsets {
        match self {
            Self::Top => EdgeInsets::new(insets.top, 0.0, 0.0, 0.0),
            Self::TopBottom => EdgeInsets::new(insets.top, 0.0, insets.bottom, 0.0),
            Self::All => insets,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionValue {
    Auto,
    Points(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PointValue {
    pub x: f32,
    pub y: f32,
}

impl PointValue {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutFrame {
    pub id: UiNodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutFrame {
    pub fn validate(&self) -> Result<(), LayoutFrameValidationError> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(LayoutFrameValidationError::NonFinitePosition);
        }
        if !self.width.is_finite() || !self.height.is_finite() {
            return Err(LayoutFrameValidationError::NonFiniteSize);
        }
        if self.width < 0.0 || self.height < 0.0 {
            return Err(LayoutFrameValidationError::NegativeSize);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutFrameValidationError {
    NonFinitePosition,
    NonFiniteSize,
    NegativeSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_insets_all_sets_each_side() {
        let insets = EdgeInsets::all(12.0);

        assert_eq!(insets.top, 12.0);
        assert_eq!(insets.right, 12.0);
        assert_eq!(insets.bottom, 12.0);
        assert_eq!(insets.left, 12.0);
    }

    #[test]
    fn point_value_preserves_components() {
        let point = PointValue::new(4.0, -8.0);

        assert_eq!(point.x, 4.0);
        assert_eq!(point.y, -8.0);
    }

    #[test]
    fn corner_radii_preserve_each_corner() {
        let radii = CornerRadii::new(4.0, 8.0, 12.0, 16.0);

        assert_eq!(radii.top_left, 4.0);
        assert_eq!(radii.top_right, 8.0);
        assert_eq!(radii.bottom_right, 12.0);
        assert_eq!(radii.bottom_left, 16.0);
    }

    #[test]
    fn safe_area_edges_top_bottom_only_keeps_vertical_insets() {
        let insets = EdgeInsets::new(59.0, 12.0, 34.0, 8.0);

        assert_eq!(
            SafeAreaEdges::TopBottom.apply_to(insets),
            EdgeInsets::new(59.0, 0.0, 34.0, 0.0)
        );
    }

    #[test]
    fn layout_frame_accepts_valid_values() {
        let frame = LayoutFrame {
            id: 1,
            x: 0.0,
            y: 4.0,
            width: 100.0,
            height: 20.0,
        };

        assert_eq!(frame.validate(), Ok(()));
    }

    #[test]
    fn layout_frame_rejects_negative_size() {
        let frame = LayoutFrame {
            id: 1,
            x: 0.0,
            y: 0.0,
            width: -1.0,
            height: 20.0,
        };

        assert_eq!(
            frame.validate(),
            Err(LayoutFrameValidationError::NegativeSize)
        );
    }

    #[test]
    fn layout_frame_rejects_non_finite_values() {
        let frame = LayoutFrame {
            id: 1,
            x: f32::NAN,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };

        assert_eq!(
            frame.validate(),
            Err(LayoutFrameValidationError::NonFinitePosition)
        );
    }
}
