# Mutation Protocol v1

## 1. Purpose
Define a stable, cross-platform command stream from Rust VDOM runtime to native renderers.

## 2. Versioning
1. Current version: `v1`.
2. Backward-compatible changes: additive only.
3. Breaking changes: major version increment.

## 3. Core Types (Conceptual)
```rust
pub type UiNodeId = u32;

pub enum Mutation {
    CreateNode { id: UiNodeId, kind: ElementKind },
    CreateTextNode { id: UiNodeId, text: String },
    SetText { id: UiNodeId, text: String },
    SetProp { id: UiNodeId, key: PropKey, value: PropValue },
    InsertChild { parent: UiNodeId, child: UiNodeId, index: u32 },
    MoveNode { id: UiNodeId, new_parent: UiNodeId, index: u32 },
    ReplaceNode { old: UiNodeId, new_id: UiNodeId, kind: ElementKind },
    RemoveNode { id: UiNodeId },
    AttachEventListener { id: UiNodeId, event: EventKind },
}
```

## 4. Ordering Rules
1. `CreateNode` or `CreateTextNode` must happen before any mutation referencing that node.
2. `InsertChild` requires both parent and child to exist.
3. `SetProp` and `SetText` can occur before or after `InsertChild` but after creation.
4. `RemoveNode` invalidates the removed node and descendants immediately.
5. `ReplaceNode` is atomic at protocol level.
6. `LayoutFrame` application always occurs after mutation batch completion.

## 5. Invariants
1. Node IDs are unique within a frame.
2. A node has at most one parent.
3. Root node cannot be removed without replacement in same frame.
4. Cycles are invalid and must be rejected.

## 6. Element Kinds
1. `Stack`
2. `Text`
3. `Button`
4. `Image`
5. `List`
6. `Input`

## 7. Property Keys
1. `Axis`
2. `Spacing`
3. `Padding`
4. `Alignment`
5. `Color`
6. `FontSize`
7. `FontWeight`
8. `CornerRadius`
9. `Source`
10. `Enabled`

## 8. Event Contract (Native -> Rust)
```rust
pub enum UiEvent {
    Tap { id: UiNodeId },
    TextInput { id: UiNodeId, value: String },
    Scroll { id: UiNodeId, dx: f32, dy: f32 },
    Appear { id: UiNodeId },
    Disappear { id: UiNodeId },
}
```

## 9. Example: Initial Mount (Counter)
1. `CreateNode(Stack#1)`
2. `SetProp(#1, Axis, Vertical)`
3. `CreateTextNode(#2, "Count: 0")`
4. `CreateNode(Button#3)`
5. `SetText(#3, "+")`
6. `InsertChild(#1, #2, 0)`
7. `InsertChild(#1, #3, 1)`
8. `AttachEventListener(#3, Tap)`

## 10. Example: Increment Update
1. `SetText(#2, "Count: 1")`

No other structural mutations are expected.

## 11. Error Policy
1. Soft error:
   - unknown prop key.
   - unknown optional event.
Action: ignore + warn.
2. Hard error:
   - invalid ID references.
   - parent-child invariant violation.
Action: reject batch and trigger reconciliation recovery.

## 12. Recovery Strategy
1. Renderer reports `BatchRejected`.
2. Runtime schedules a full subtree re-sync for affected root.
3. Renderer drops stale handles not present after re-sync.

