use native_schema::{LayoutFrame, Mutation, ProtocolVersion};

const DEFAULT_HOST_WIDTH: f32 = 390.0;
const DEFAULT_HOST_HEIGHT: f32 = 844.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostSize {
    pub width: f32,
    pub height: f32,
}

impl HostSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for HostSize {
    fn default() -> Self {
        Self::new(DEFAULT_HOST_WIDTH, DEFAULT_HOST_HEIGHT)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderBatch {
    pub protocol_version: ProtocolVersion,
    pub mutations: Vec<Mutation>,
    pub layout: Vec<LayoutFrame>,
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self {
            protocol_version: ProtocolVersion::V1,
            mutations: Vec::new(),
            layout: Vec::new(),
        }
    }
}
