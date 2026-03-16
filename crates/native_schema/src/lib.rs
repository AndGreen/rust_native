//! Canonical schema types shared by the Rust VDOM runtime and native renderers.
//!
//! This crate is intentionally backend-agnostic and does not depend on
//! `mf_core`, `mf_widgets`, or any platform-specific code.

mod events;
mod layout;
mod mutation;

pub use events::UiEvent;
pub use layout::{
    CornerRadii, DimensionValue, EdgeInsets, LayoutFrame, LayoutFrameValidationError, PointValue,
    SafeAreaEdges,
};
pub use mutation::{
    Alignment, Axis, ColorValue, ElementKind, EventKind, FontWeight, JustifyContent, LineStyle,
    Mutation, PropKey, PropValue, ProtocolVersion, ShadowStyle, UiNodeId,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_protocol_version_v1() {
        assert_eq!(ProtocolVersion::V1, ProtocolVersion::default());
    }
}
