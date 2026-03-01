# VDOM Native Roadmap

## 1. Purpose
Define post-remediation milestones to reach production-grade native-view framework quality.

## 2. Milestone M1: Stable VDOM Core
Target: first stable internal release.

1. Mutation protocol `v1` frozen.
2. Canonical layout path enabled by default.
3. Incremental iOS and Android renderers for core widgets.
4. Basic event roundtrip and deterministic updates.

Exit Criteria:
1. Core demos run on both platforms.
2. No regular full-tree remount in update path.

## 3. Milestone M2: Interaction and Input
1. Text input focus management.
2. Scroll event normalization.
3. Gesture baseline: tap, long press, simple drag.
4. Keyboard visibility and safe area integration.

Exit Criteria:
1. Editable form demo.
2. Gesture demo with cross-platform parity.

## 4. Milestone M3: Visual Fidelity
1. Image loading pipeline with caching.
2. Typography controls and text truncation rules.
3. Better color, shadow, corner, and clipping parity.
4. Theming baseline.

Exit Criteria:
1. Gallery demo with scrolling list and images.
2. Visual regression snapshots accepted for iOS and Android.

## 5. Milestone M4: Reliability and Tooling
1. Crash-safe recovery paths for protocol errors.
2. Dev diagnostics panel (mutation trace and node tree).
3. Better debug logging and feature flags.
4. Performance dashboards for frame time and mutation volume.

Exit Criteria:
1. Stable nightly runs.
2. Reproducible profiling workflow documented.

## 6. Milestone M5: Accessibility and Semantics
1. Semantic role mapping.
2. Label and hint propagation.
3. Focus traversal policies.
4. Screen reader baseline support.

Exit Criteria:
1. Accessibility demo checklist passed on both platforms.

## 7. Milestone M6: Public Beta
1. API freeze proposal.
2. Migration notes from pre-beta APIs.
3. Starter templates and examples.
4. Beta feedback loop process.

Exit Criteria:
1. Public beta tag with versioned docs and known issues list.

## 8. Suggested Timeline
1. M1-M2: 2-3 months.
2. M3-M4: 2 months.
3. M5-M6: 2 months.

Total: ~6-7 months depending on team size and platform expertise.

