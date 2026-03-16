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
     ├─ counter/       # Signal-driven counter demo + local iOS app host
     └─ album_list/    # List/feed demo + local iOS app host
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
4. Build the iOS static libraries for the local example hosts:
   ```bash
   cargo build -p counter --target aarch64-apple-ios-sim
   cargo build -p album_list --target aarch64-apple-ios-sim
   cargo build -p form_demo --target aarch64-apple-ios-sim
   ```
5. Open the per-example iOS project when needed:
   - `examples/counter/ios/App/App.xcodeproj`
   - `examples/album_list/ios/App/App.xcodeproj`
   - `examples/form_demo/ios/App/App.xcodeproj`

## Remote Dev Server

The examples can run through the TCP dev server instead of embedding the Rust app directly into the host process. The current supported host flow in this repository is the local iOS Simulator setup used by the example Xcode projects.

1. Start the dev server for the app you want to run:
   ```bash
   cargo run -p dev_cli -- --app form_demo --host 127.0.0.1 --port 4488
   ```
   Replace `form_demo` with `counter` or `album_list` as needed.
2. Keep the server running in a terminal. It will:
   - build the selected Rust example;
   - spawn the remote worker process;
   - watch the workspace for file changes;
   - rebuild and reconnect the UI on change.
3. Open the matching Xcode project and run the `App` scheme in the iOS Simulator.

### Environment Variables For The Simulator

When the native host starts in remote-dev mode, it reads these environment variables:

- `MF_DEV_SERVER_HOST`: host name or IP address for `dev_cli`.
- `MF_DEV_SERVER_PORT`: TCP port for `dev_cli`. Defaults to `4488` if omitted by the host.

For the local iOS Simulator flow on the same Mac, use:

```text
MF_DEV_SERVER_HOST=127.0.0.1
MF_DEV_SERVER_PORT=4488
```

These values are already configured in the shared Xcode schemes shipped with the examples. If you wire up a custom native host or emulator, make sure its launch environment uses the same host and port as the `dev_cli` command line.

Do not pass these worker-only variables into the simulator manually:

- `MF_DEV_REMOTE_WORKER`
- `MF_DEV_HOST_METRICS`

`dev_cli` injects them into the spawned Rust worker process automatically. The simulator/native host only needs `MF_DEV_SERVER_HOST` and `MF_DEV_SERVER_PORT`.

## Solid-like Reactive Helpers

- `create_signal` / `Setter` mirror Solid's `createSignal`.
- `batch_updates(|| { ... })` coalesces multiple setter calls before notifying subscribers.
- `Scope` + `on_cleanup` let you register cleanup callbacks (e.g., to stop timers).
- `start_interval(Duration, f)` spawns a cancelable interval; pair it with `on_cleanup`.
- The `counter` example demonstrates the pattern: interval-driven increments, batched button updates, and cleanup.

## Key Concepts

- **`mf_core`**: Provides the `View` type, widget traits, signal primitives (`signal`, `Signal`, `Setter`), a tiny diffing engine, and a `taffy`-based layout shim.
- **`mf_macros::ui!`**: Compiles SwiftUI-like syntax into pure Rust by chaining `IntoView`/`WithChildren` implementations. Supports positional and named args plus modifier chains.
- **`mf_widgets`**: Supplies basic widgets (`Text`, `Button`, `Image`, `Container`, `VStack`, `HStack`, `List`) with builder-style modifiers for fonts, colors, spacing, and the first visual foundation props.
- **`mf_runtime`**: Hosts the `App` type that owns a backend and rebuilds the tree whenever watched signals emit updates.
- **Backends**: `backend_native` renders to UIKit when built for iOS (bootstraps a `UIWindow`/`UIViewController` and maps `Text`, `Button`, `Image`, `HStack`/`VStack`, `List` into native views). On non-iOS targets it stays a logging stub. Each example now ships with its own Rust `staticlib` entrypoints plus a colocated Xcode host in `examples/*/ios/App`, so simulator/device startup stays local to that example. `backend_wgpu` remains a logging stub for now.

## Roadmap Snapshot

See `MF_framework_plan.md` for the full vision. Immediate priorities after this MVP:

1. Replace logging backends with real UIKit/Android/wgpu renderers.
2. Expand widgets (ScrollView, gestures, environment values) and runtime services (focus, async events).
3. Harden the macro diagnostics and add tests around signals/diffing to keep ergonomics high as features grow.

## VDOM-First Docs

Current execution documentation for the VDOM-first path:

1. `docs/remediation_plan.md` - 12-week implementation and stabilization plan.
2. `docs/architecture_vdom.md` - target runtime and renderer architecture.
3. `docs/mutation_protocol.md` - canonical mutation contract and invariants.
4. `docs/layout_contract.md` - Rust-driven layout contract and frame rules.
5. `docs/testing_strategy.md` - test layers, CI gates, and acceptance scenarios.
6. `docs/roadmap_vdom_native.md` - post-remediation milestones.

## Contributing

This repository is in rapid iteration mode. Feel free to experiment inside the examples or add new crates, but keep code formatted (`cargo fmt`) and linted (`cargo clippy`) before submitting patches. Open issues/PRs for discussion on architecture changes or new widget APIs. All contributions should maintain the “Rust-only, no JS bridge” principle laid out in the plan.
