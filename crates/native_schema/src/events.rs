use crate::UiNodeId;

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    Tap { id: UiNodeId },
    TextInput { id: UiNodeId, value: String },
    Scroll { id: UiNodeId, dx: f32, dy: f32 },
    Appear { id: UiNodeId },
    Disappear { id: UiNodeId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_preserves_value() {
        let event = UiEvent::TextInput {
            id: 9,
            value: "hello".to_string(),
        };

        match event {
            UiEvent::TextInput { id, value } => {
                assert_eq!(id, 9);
                assert_eq!(value, "hello");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn scroll_preserves_offsets() {
        let event = UiEvent::Scroll {
            id: 5,
            dx: 4.5,
            dy: -8.0,
        };

        match event {
            UiEvent::Scroll { id, dx, dy } => {
                assert_eq!(id, 5);
                assert_eq!(dx, 4.5);
                assert_eq!(dy, -8.0);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
