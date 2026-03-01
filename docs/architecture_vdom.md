# VDOM-First Architecture

## 1. Architectural Principle
Rust owns the UI tree, reconciliation, layout computation, and event routing. Native platforms only materialize and mutate view hierarchies based on Rust-issued commands.

## 2. High-Level Layers
1. `mf_widgets`: declarative widget primitives and modifiers.
2. `mf_macros`: `ui!` DSL expansion into Rust view construction.
3. `vdom_runtime`: VDOM lifecycle, diff, mutation emission, event intake.
4. `native_schema`: canonical element and property vocabulary.
5. `backend_native`:
   - iOS mutation executor.
   - Android mutation executor.
6. `mf_runtime`: scheduler, subscriptions, redraw triggers.

## 3. Canonical Data Flow
1. App state changes.
2. `mf_runtime` schedules render.
3. `vdom_runtime` computes tree changes.
4. Runtime emits canonical `Mutation[]`.
5. Runtime computes `LayoutFrame[]`.
6. Native backend applies `Mutation[]`, then applies `LayoutFrame[]`.
7. Native backend captures platform events and sends `UiEvent` back.
8. Loop continues with incremental updates.

## 4. Threading Model
1. Rust render/diff/layout can run on a non-UI thread.
2. Native view creation and mutation must run on UI main thread.
3. Event callbacks arrive on UI thread and are marshaled back to Rust runtime queue.
4. A frame boundary ensures deterministic ordering:
   - `mutations` first.
   - `layout` second.
   - `flush` third.

## 5. Node Identity and Registry
1. Each UI node has stable `UiNodeId`.
2. Runtime keeps node graph and parent-child relation map.
3. Backends keep `UiNodeId -> NativeHandle` map.
4. ID reuse is forbidden within one frame.
5. Removing a node removes all descendants unless explicitly reparented in the same frame.

## 6. Native Schema Boundary
This layer avoids leaking HTML/CSS semantics into mobile native renderers.

### Elements
1. `stack`
2. `text`
3. `button`
4. `image`
5. `list`
6. `input`

### Core Props
1. `axis`
2. `spacing`
3. `padding`
4. `alignment`
5. `text`
6. `font_size`
7. `font_weight`
8. `color`
9. `src`
10. `corner_radius`
11. `enabled`

## 7. Renderer Responsibilities
### iOS Renderer
1. Map schema elements to UIKit classes.
2. Execute mutations in order.
3. Apply frames after mutation pass.
4. Route touch/input events to runtime.

### Android Renderer
1. Map schema elements to `View` subclasses.
2. Execute mutations in order.
3. Apply frames after mutation pass.
4. Route click/input/scroll events to runtime.

## 8. Error Handling
1. Unknown element: create fallback placeholder node and log warning.
2. Unknown prop: ignore safely and log once per prop key.
3. Missing parent on insert: reject mutation batch and request full sync.
4. Invalid node ID: treat as protocol violation and recover with subtree rebuild.

## 9. Performance Baseline Rules
1. Skip no-op `SetProp` and `SetText`.
2. Batch multiple mutations per frame.
3. Avoid full rebuild on normal state updates.
4. Keep node lookup O(1) by ID.

## 10. Compatibility Strategy
1. Mutation protocol is versioned.
2. New fields must be additive.
3. Removed fields require major protocol version bump.
4. Native backends must reject unsupported major versions early.

