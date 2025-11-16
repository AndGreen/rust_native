# Rust-Native Multi-Platform UI Framework Plan

This document outlines a fully Rust-native UI framework inspired by SwiftUI and React Native. The entire stack is implemented in Rust, compiles to static code (no runtime reflection or JavaScript bridge), and targets both native widget sets and GPU rendering backends.

## 1. Vision and Design Goals

- **Rust-first declarative UI:** Express every UI tree as pure Rust values generated at compile time via macros.
- **Safety over dynamism:** No runtime type inspection, no JS runtime; leverage Rust's type system, pattern matching, and ownership.
- **Predictable performance:** Minimize allocations, avoid GC, batch state updates, and diff only what changes.
- **Composable and testable:** Widgets are deterministic functions of state; signals enable snapshot testing and logic reuse.
- **Multi-backend parity:** Same widget tree can hydrate native UIKit/Android controls or render through wgpu/vello with identical behavior.
- **Extensible tooling:** Encourage third-party backends, custom widgets, and editor tooling by stabilizing ABI/API boundaries inside the workspace.

## 2. Cargo Workspace Architecture

```
mf/
 ├─ Cargo.toml                # Workspace definition
 ├─ crates/
 │   ├─ core/                 # mf_core runtime, diffing, layout bridge
 │   ├─ macros/               # mf_macros procedural macros + DSL parser
 │   ├─ backend_api/          # Common traits + FFI-safe interfaces
 │   ├─ backend_native/       # UIKit + Android View bindings via objc2/JNI
 │   ├─ backend_wgpu/         # GPU renderer (wgpu + vello)
 │   ├─ widgets/              # Standard widgets & modifiers
 │   └─ runtime/              # Scheduler, events, animation loop
 └─ examples/
     ├─ counter/
     └─ album_list/
```

### `mf_core`
- Signal primitives (`Signal<T>`, `Setter<T>`, derived selectors) implemented via lock-free cells or `Arc<RwLock<T>>` with change tokens.
- Diffing engine that compares widget trees produced by the DSL expansion and emits `Patch` structures scoped to nodes.
- Layout layer integrating the `taffy` crate for Flexbox/Grid; converts widget traits (`LayoutSpec`) into taffy nodes and returns resolved frames.
- Widget trait definitions (`Widget`, `RenderObject`, `Modifier`) and lifecycle hooks (`mount`, `unmount`, `update`).

### `mf_macros`
- Exposes `ui!` proc-macro that parses SwiftUI-like nested syntax.
- Expands `Type { ... }` blocks into `Type::new().child(...)` invocations, auto-inserting constructors if omitted.
- Emits compile-time diagnostics (span errors) for invalid attributes, duplicate modifiers, or lifetime issues.
- Generates deterministic IDs for diffing and associates captured closures with `'static` or borrowing lifetimes.

### `backend_api`
- Defines platform-agnostic traits for creating, updating, and disposing platform nodes (`BackendNode`, `BackendContext`).
- Provides serialization helpers for font, color, and image descriptors, plus feature probes (e.g., `supports_blur()`).

### `backend_native`
- Wraps UIKit controls (UILabel, UIButton, UIStackView, UIScrollView) via `objc2`, and Android Views via JNI.
- Manages thread affinity, ensuring updates run on the main UI threads for iOS and Android.
- Translates diff patches into native view tree mutations (insert/remove/update constraints).
- Bridges gestures, input events, accessibility tree, and focus states back to the Rust runtime.

### `backend_wgpu`
- Builds a retained scene graph and issues draw calls through `wgpu`.
- Uses `vello` (or Skia-equivalent) for text shaping, vector graphics, and image compositing.
- Shares layout results from `mf_core`, applying transforms and clipping before rendering.
- Handles GPU resource lifecycle, texture atlases, and asynchronous uploads from the runtime thread.

### `widgets`
- Implements standard widgets: `Text`, `Button`, `Image`, `VStack`, `HStack`, `ScrollView`, `List`, `Spacer`, `Toggle`, etc.
- Provides modifiers (`.padding()`, `.font()`, `.color()`, `.corner_radius()`, `.shadow()`) as composable structs.
- Re-exports widget DSL-friendly constructors used inside `ui!`.

### `runtime`
- Central scheduler that orchestrates signal updates, batching, and coalescing diff passes.
- Event loop hooks per platform (run loop observers on iOS, Choreographer on Android, winit on desktop).
- Animation timelines, spring drivers, async command queue, focus ring, and accessibility semantics.
- Integrates global contexts (e.g., `Environment<T>`) and portable asset loading.

## 3. SwiftUI-like DSL Syntax

```rust
let (count, set_count) = use_signal(0);

ui! {
    VStack(spacing = 12, padding = 16) {
        Text("Albums")
            .font(Font::bold(20))
            .color(Color::primary())

        List(albums, |album| {
            HStack(spacing = 8) {
                Image(album.cover)
                    .size(60, 60)
                    .corner_radius(8)

                VStack(alignment = .leading) {
                    Text(&album.title)
                        .font(Font::semibold(16))
                    Text(&album.artist)
                        .foreground(Color::secondary())
                }
            }
        })
    }
}
```

### Macro Behavior
- Parses nested `{}` blocks and produces explicit `.child(...)` method chains.
- Distinguishes positional arguments, named parameters, and builder modifiers.
- Supports closures with captured state (`List(items, |item| { ... })`), ensuring borrow checking remains valid.
- Emits strongly typed Rust code—no runtime reflection or string-based lookups.

## 4. Reactive Signal System

- `use_signal(T)` returns `(Signal<T>, Setter<T>)`.
- `Signal<T>::get()` provides copy/cloned snapshots; `Setter<T>::update(|value| { ... })` mutates with interior mutability.
- Signals track subscribers, enqueueing dirty nodes in the scheduler which triggers diffing only for affected subtrees.
- Bindings (`use_binding(&Signal<T>)`) propagate state down the tree, similar to SwiftUI's `@Binding`.
- Derived signals / memoization: `let doubled = derive(&count, |c| c * 2);`.
- Supports async updates by integrating with `runtime::Executor` (e.g., `spawn(async move { set_count.set(fetch().await); })`).

## 5. Rendering Backends & Layout

- Layout is delegated to `taffy`; widgets describe constraints via `impl LayoutSpec`.
- `mf_core` computes layout once per frame; results are consumed identically by both backends.
- **Native backend:** Maps widget nodes to UIKit/Android Views, syncing diff patches to native properties and constraints.
- **GPU backend:** Constructs a scene graph rendered via `wgpu`/`vello`, enabling advanced visuals and desktop targets.
- Backend selection occurs at compile time (`backend_native` or `backend_wgpu` feature flags) or runtime via trait objects.

## 6. Event Handling & Lifecycle

- Runtime normalizes events (`Event::Click`, `Event::Scroll`, `Event::Key`, `GestureEvent`) and dispatches them down the tree.
- Widgets expose closures such as `.on_click(|| println!("Liked"))` or `.on_scroll(|delta| { ... })`.
- Ensures callbacks run on the UI thread; cross-thread communication uses channels with `Send + 'static` requirements.
- Lifecycle hooks: `on_mount`, `on_unmount`, `on_appear`, `on_disappear`, and `on_animation_frame`.
- Accessibility: runtime synthesizes semantics tree for VoiceOver/TalkBack, drawing metadata from widget properties.

## 7. Build & Workspace Flow

- Workspace root `Cargo.toml` enables feature selection (e.g., `default = ["backend_native"]`, optional `wgpu`).
- Each crate has unit and integration tests; `examples/` provide runnable demos via `cargo run -p counter_demo`.
- Continuous integration should run `cargo fmt`, `cargo clippy`, `cargo test`, plus backend-specific smoke tests (simulator/emulator or headless wgpu).

## 8. Evolution Roadmap

| Phase | Features |
|-------|----------|
| MVP   | Text, Button, VStack, HStack, List, signal system, native backend skeleton |
| v0.2  | ScrollView, Image, full `taffy` layout integration, environment values |
| v0.3  | Gestures, keyboard/focus management, async event commands, derived signals |
| v0.4  | Animation/transition engine, timeline APIs, layout transitions |
| v1.0  | Production-ready wgpu renderer, Skia/vello parity, hot reload/dev inspector |

## 9. Guiding Philosophy

- **Compile-time guarantees:** Prefer macro expansions and traits over dynamic registries; catch errors during build.
- **Zero reflection:** All metadata is Rust types; no `Any`-based downcasts in public API.
- **Minimal runtime overhead:** Lean data structures, slab allocators for node storage, batched updates.
- **Unified reactive model:** Apply the same signal/diff engine to desktop, mobile, and GPU surfaces.
- **Extensibility:** Encourage custom widgets/backends via documented traits, macro hooks, and backend-agnostic modifiers.

## 10. Next Steps

1. Scaffold Cargo workspace with stub crates and minimal `ui!` macro.
2. Implement `Signal` primitives and diffing logic inside `mf_core`.
3. Build MVP widgets and connect them to the runtime scheduler.
4. Deliver the counter and album list examples to validate ergonomics.

This roadmap turns Rust into a native declarative UI language with predictable performance and strong compile-time guarantees.
