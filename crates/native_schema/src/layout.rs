use crate::UiNodeId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DimensionValue {
    Auto,
    Points(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
