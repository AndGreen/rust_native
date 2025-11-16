# Rust-Native UI MVP

This workspace hosts an experimental, fully Rust-native UI framework inspired by SwiftUI/React Native but without any JavaScript runtime or reflection. Everything – DSL parsing, reactive state, layout, widgets, runtime scheduler, and sample apps – lives in Rust crates inside the `mf_*` namespace.

## Workspace Layout

```
mf/
 ├─ crates/
 │   ├─ core/          # Signals, diffing, layout bridge, View representation
 │   ├─ macros/        # `ui!` procedural macro and DSL parser
 │   ├─ backend_api/   # Backend trait + debug helpers
 │   ├─ backend_native/# Logging stub that mimics UIKit/Android bindings
 │   ├─ backend_wgpu/  # Logging stub for GPU renderer
 │   ├─ widgets/       # Text, Button, Image, VStack, HStack, List primitives
 │   └─ runtime/       # App scheduler, signal watching, repaint loop
 └─ examples/
     ├─ counter/       # Signal-driven counter demo
     └─ album_list/    # List/feed demo with nested stacks
```

## Getting Started

1. **Install Rust** (stable toolchain) with `rustup`.
2. Fetch dependencies and verify everything builds:
   ```bash
   cargo check
   ```
3. Run the examples – each prints the rendered view tree through the logging backend:
   ```bash
   cargo run -p counter
   cargo run -p album_list
   ```

## Key Concepts

- **`mf_core`**: Provides the `View` type, widget traits, signal primitives (`signal`, `Signal`, `Setter`), a tiny diffing engine, and a `taffy`-based layout shim.
- **`mf_macros::ui!`**: Compiles SwiftUI-like syntax into pure Rust by chaining `IntoView`/`WithChildren` implementations. Supports positional and named args plus modifier chains.
- **`mf_widgets`**: Supplies basic widgets (`Text`, `Button`, `Image`, `VStack`, `HStack`, `List`) with builder-style modifiers for fonts, colors, spacing, etc.
- **`mf_runtime`**: Hosts the `App` type that owns a backend and rebuilds the tree whenever watched signals emit updates.
- **Backends**: `backend_native` and `backend_wgpu` currently log the diffed tree; they are designed to be swapped for actual UIKit/Android or wgpu/vello integrations later.

## Roadmap Snapshot

See `MF_framework_plan.md` for the full vision. Immediate priorities after this MVP:

1. Replace logging backends with real UIKit/Android/wgpu renderers.
2. Expand widgets (ScrollView, gestures, environment values) and runtime services (focus, async events).
3. Harden the macro diagnostics and add tests around signals/diffing to keep ergonomics high as features grow.

## Contributing

This repository is in rapid iteration mode. Feel free to experiment inside the examples or add new crates, but keep code formatted (`cargo fmt`) and linted (`cargo clippy`) before submitting patches. Open issues/PRs for discussion on architecture changes or new widget APIs. All contributions should maintain the “Rust-only, no JS bridge” principle laid out in the plan.
