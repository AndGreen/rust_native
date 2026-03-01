# VDOM-First Remediation Plan (12 Weeks)

## 1. Goal
Bring the repository from MVP/prototype status to a stable VDOM-first native-view foundation where Rust owns the UI tree and emits incremental mutations consumed by iOS and Android renderers.

## 2. Success Criteria
1. `cargo check`, `cargo test`, and `cargo clippy -- -D warnings` pass in CI.
2. Rust emits granular mutations (no full-tree replace as the default update path).
3. iOS renderer applies incremental updates to native views.
4. Android renderer applies incremental updates to native views.
5. `counter` and `album_list` demos run through the VDOM pipeline on both platforms.
6. Mutation and layout contracts are documented and versioned.

## 3. Scope
### In Scope
1. Compile stabilization and baseline tests.
2. VDOM runtime crate and mutation protocol.
3. Canonical layout contract on top of Taffy.
4. iOS and Android mutation executors.
5. CI quality gates and acceptance tests.

### Out of Scope
1. Full production UX polish (advanced animations, full accessibility parity, rich gestures).
2. Broad widget catalog expansion.
3. App Store / Play Store release automation.

## 4. Risks and Controls
1. Risk: model mismatch between Dioxus internals and native schema.
Control: strict `native_schema` vocabulary and translation tests.
2. Risk: JNI/Objective-C lifecycle bugs.
Control: ownership policy, stress tests for create/remove cycles, explicit weak/strong reference strategy.
3. Risk: layout parity drift between iOS and Android.
Control: canonical Rust layout frames + snapshot parity tests.
4. Risk: performance regressions from excessive mutation volume.
Control: mutation coalescing, no-op filtering, perf budget checks in CI.

## 5. Workstreams
1. Stabilization and cleanup.
2. VDOM runtime and mutation translation.
3. Layout contract and frame delivery.
4. iOS renderer.
5. Android renderer.
6. CI, testing, and documentation.

## 6. Delivery Plan by Week
### Week 1: Stabilization Baseline
1. Fix duplicate API names in widgets (`font`, `color`, `size` conflicts).
2. Add minimum tests for widgets/core/runtime smoke behavior.
3. Ensure workspace builds on default host target.
4. Freeze baseline metrics (build time, binary size, mutation throughput placeholder benchmark).

### Weeks 2-3: VDOM Runtime Core
1. Add `crates/vdom_runtime`.
2. Introduce canonical mutation types and node IDs.
3. Build translation layer from VDOM diff output to canonical `Mutation`.
4. Add deterministic tests for mutation sequences.
5. Add crate-level docs and examples.

### Weeks 4-5: Canonical Layout
1. Add `docs/layout_contract.md` implementation source-of-truth rules into code comments/tests.
2. Build layout tree from canonical node props.
3. Compute frames in Rust and publish `LayoutFrame` per node.
4. Add fallback strategy for unsupported props.
5. Add parity snapshots for expected frame trees.

### Weeks 6-7: iOS Incremental Renderer
1. Replace rebuild-all flow with mutation executor.
2. Implement create/insert/remove/replace/set-prop/set-text operations.
3. Add event bridge (tap, input, appear/disappear minimal set).
4. Enforce main-thread UI mutation path.
5. Add iOS smoke test script and manual verification checklist.

### Weeks 8-9: Android Incremental Renderer
1. Add Android bridge crate/module and JNI scaffolding.
2. Implement mutation executor mirroring iOS semantics.
3. Add event bridge with stable event payload contract.
4. Enforce UI-thread execution for all view mutations.
5. Add Android smoke test script and parity checklist.

### Weeks 10-11: Reliability and Parity
1. Add cross-platform parity tests for mutation and layout snapshots.
2. Add failure-mode coverage: invalid IDs, missing parents, unsupported props.
3. Add no-op mutation filtering and batch flushing.
4. Add perf regression checks for core scenarios.
5. Align docs and implementation behavior.

### Week 12: Hardening and Release Candidate Baseline
1. Finalize docs and architecture references.
2. Lock mutation protocol version `v1`.
3. Produce RC checklist and known issues.
4. Tag a milestone release candidate in git.

## 7. Definition of Done
1. Mutation contract `v1` is implemented and documented.
2. Incremental updates are default behavior on iOS and Android.
3. Layout is computed once in Rust and consumed consistently by both renderers.
4. Core demos function through VDOM pipeline with no full subtree remount as regular update mechanism.
5. CI enforces formatting, linting, tests, and core smoke checks.

## 8. Implementation Order (Strict)
1. Stabilize compile and tests.
2. Introduce canonical mutation schema.
3. Wire runtime mutation emission.
4. Move iOS renderer to incremental mutation executor.
5. Build Android parity path.
6. Harden reliability and performance.

## 9. Ownership Template
Use this section as an execution tracker.

| Workstream | Owner | Backup | Start | End | Status |
|---|---|---|---|---|---|
| Stabilization | TBD | TBD | TBD | TBD | Planned |
| VDOM Runtime | TBD | TBD | TBD | TBD | Planned |
| Layout | TBD | TBD | TBD | TBD | Planned |
| iOS Renderer | TBD | TBD | TBD | TBD | Planned |
| Android Renderer | TBD | TBD | TBD | TBD | Planned |
| CI and Testing | TBD | TBD | TBD | TBD | Planned |

