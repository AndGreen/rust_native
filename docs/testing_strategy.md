# Testing Strategy

## 1. Testing Goals
1. Prevent regressions in mutation semantics.
2. Guarantee cross-platform renderer parity.
3. Detect protocol/layout incompatibilities early.
4. Enforce minimum performance and reliability baselines.

## 2. Test Layers
### Unit Tests
1. Widget-to-schema mapping.
2. Mutation translator behavior.
3. Layout builder input normalization.
4. Event payload parsing and validation.

### Golden Snapshot Tests
1. Mutation sequence snapshots for canonical screens.
2. Layout frame snapshots for canonical screens.
3. Versioned fixtures per protocol version (`v1` directory).

### Integration Tests
1. Runtime render loop with synthetic events.
2. End-to-end event roundtrip:
   - emit native event.
   - process in Rust.
   - assert mutation delta.
3. Renderer contract tests using mock backend.

### Platform Smoke Tests
1. iOS build and basic launch path.
2. Android build and basic launch path.
3. Scenario scripts for `counter` and `album_list`.

### Performance Tests
1. Mutation throughput test for large list updates.
2. No-op update test ensuring low mutation count.
3. Memory churn test for repeated create/remove cycles.

## 3. CI Gates
1. `cargo fmt --check`
2. `cargo check --workspace`
3. `cargo clippy --workspace -- -D warnings`
4. `cargo test --workspace`
5. Mutation golden snapshot verification.
6. Layout snapshot verification.
7. Optional nightly perf checks with thresholds.

## 4. Acceptance Scenarios
1. Counter screen:
   - initial mount.
   - increment/decrement.
   - no full-tree remount on value changes.
2. Album list:
   - initial list render.
   - item insert/remove.
   - item reorder.
3. Error handling:
   - unknown prop.
   - invalid node ID.
   - parent missing on insert.

## 5. Quality Thresholds
1. Zero failing tests on main branch.
2. Zero clippy warnings in CI.
3. Snapshot changes require explicit review.
4. Performance regressions above threshold block merge.

## 6. Release Checklist
1. Mutation protocol docs updated.
2. Layout contract docs updated.
3. Snapshot fixtures regenerated and reviewed.
4. Smoke tests passed on iOS and Android.
5. Known limitations documented in release notes.

