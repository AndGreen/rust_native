# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace centered on native UI rendering. Core libraries live in `crates/`: `core`, `macros`, `widgets`, `runtime`, `vdom_runtime`, and backend crates such as `backend_native` and `backend_wgpu`. Developer tooling is in `crates/dev_cli`, `crates/dev_protocol`, and `crates/dev_support`. Runnable examples live in `examples/counter`, `examples/album_list`, and `examples/form_demo`; each example keeps Rust sources in `src/` and an iOS host app in `ios/`. Design and rollout docs are under `docs/`.

## Build, Test, and Development Commands
Use workspace-level Cargo commands from the repository root:

- `cargo check --workspace` — fast validation for all crates.
- `cargo fmt --check` — enforce formatting before review.
- `cargo clippy --workspace --all-targets -- -D warnings` — CI-level lint gate.
- `cargo test --workspace` — run unit, integration, and snapshot tests.
- `cargo run -p counter` or `cargo run -p album_list` — run example apps with the logging backend.
- `cargo run -p dev_cli -- --app counter` — start the local dev loop for a specific example.
- `cargo build -p counter --target aarch64-apple-ios-sim` — build iOS simulator static libraries; repeat for other examples as needed.

## Coding Style & Naming Conventions
Follow standard Rust 2021 style with default `rustfmt` output: 4-space indentation, trailing commas where formatter adds them, and no manual formatting fights. Use `snake_case` for functions, modules, and test names; `PascalCase` for types and traits; keep crate and package names lowercase with underscores. Favor small modules with explicit imports over glob-heavy files, except for UI prelude usage already established in examples.

## Testing Guidelines
Tests are split between inline module tests such as `crates/runtime/src/tests.rs` and integration tests in `crates/vdom_runtime/tests/`. Snapshot fixtures live in `crates/vdom_runtime/tests/fixtures/*.snap`; update them only when the rendered mutation protocol intentionally changes. Keep test names descriptive, for example `counter_update_snapshot_matches_fixture`. Run `cargo test --workspace` locally before opening a PR; perf coverage uses `cargo test -p vdom_runtime --test perf_harness -- --ignored --nocapture`.

## Commit & Pull Request Guidelines
Recent history uses short imperative commit subjects like `Fix first dev run` and `Refactor runtime and native backend internals`. Keep commits focused and descriptive, ideally under 72 characters, and avoid mixing refactors with behavior changes. PRs should include a concise summary, affected crates/examples, linked issues or design docs, and screenshots or simulator notes for UI-visible changes. If you touch rendering, layout, or protocol behavior, mention test and snapshot updates explicitly.
