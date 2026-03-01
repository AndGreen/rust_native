# Layout Contract (Taffy Canonical)

## 1. Objective
Use Rust as the single layout authority. Native renderers receive concrete frames and apply them directly.

## 2. Pipeline
1. Build layout tree from canonical nodes and props.
2. Resolve style with defaults.
3. Compute layout with Taffy.
4. Emit `LayoutFrame` list keyed by `UiNodeId`.
5. Apply frames on native side after mutation pass.

## 3. Core Data Type
```rust
pub struct LayoutFrame {
    pub id: UiNodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

## 4. Coordinate Rules
1. Coordinates are local to parent content area.
2. Root coordinates are relative to host container.
3. Units are logical pixels (`dp/pt` equivalent abstraction).
4. Negative width/height is invalid and rejected.

## 5. Supported Layout Props v1
1. `axis`: horizontal or vertical.
2. `spacing`: gap between direct children.
3. `padding`: uniform or directional.
4. `alignment`: leading, center, trailing.
5. `width` and `height`: fixed or auto.
6. `min_width`, `min_height`, `max_width`, `max_height`.
7. `flex_grow`, `flex_shrink`.

## 6. Defaults
1. `axis = vertical` for stacks/lists.
2. `spacing = 0`.
3. `padding = 0`.
4. `alignment = leading`.
5. `width/height = auto`.

## 7. Native Mapping
### iOS
1. Frame application via `setFrame`.
2. Layer properties for visual props when needed (`cornerRadius`, etc).
3. If Auto Layout is present in host shell, managed subtree uses manual frames only.

### Android
1. Apply position and size through layout params and explicit layout calls.
2. Keep managed subtree isolated from parent auto measurement conflicts.

## 8. Frame Application Order
1. Parent frames before child frames.
2. Stable order by tree traversal (preorder).
3. Apply all structural mutations first, then all layout frames.

## 9. Unsupported Cases in v1
1. Text intrinsic multi-line complexity beyond simple measurement.
2. Baseline alignment.
3. Advanced wrapping and overflow policies.

Fallback behavior:
1. Use intrinsic or minimum-safe size.
2. Emit warning diagnostics once per node kind.

## 10. Validation
1. Every node in rendered tree must have one frame.
2. No duplicate `LayoutFrame` IDs in same frame batch.
3. Parent-child containment is best-effort and logged if violated.

## 11. Test Cases
1. Fixed-size stack with known spacing and padding.
2. Nested stacks with mixed axis.
3. Dynamic list insert/remove and frame stability.
4. Text update with unchanged structure and expected frame changes.

