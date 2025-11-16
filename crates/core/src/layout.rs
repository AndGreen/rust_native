use taffy::prelude::*;

/// Wrapper over a Taffy style node used by widgets to describe layout constraints.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    style: Style,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self { style }
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
}

/// Widgets implement this trait to expose their layout requirements to the engine.
pub trait LayoutSpec {
    fn create_layout(&self) -> LayoutNode;
}

#[derive(Debug, Clone, Copy)]
pub struct ComputedLayout {
    pub width: f32,
    pub height: f32,
}

impl Default for ComputedLayout {
    fn default() -> Self {
        Self { width: 0.0, height: 0.0 }
    }
}

/// Convenience helper for computing layout for a node tree. The MVP implementation hooks
/// directly into Taffy and returns the resolved size for the provided node.
pub fn compute_layout(root: &LayoutNode, size: Size<AvailableSpace>) -> ComputedLayout {
    let mut taffy = Taffy::new();
    let node = taffy.new_leaf(root.style().clone()).unwrap_or_else(|_| taffy.new_leaf(Style::DEFAULT).unwrap());
    let _ = taffy.compute_layout(node, size);
    taffy
        .layout(node)
        .map(|layout| ComputedLayout {
            width: layout.size.width,
            height: layout.size.height,
        })
        .unwrap_or_default()
}
