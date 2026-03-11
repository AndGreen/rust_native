use std::time::Instant;

use mf_core::{IntoView, View, WithChildren};
use mf_widgets::prelude::*;
use vdom_runtime::{HostSize, VdomRuntime};

const TEST_HOST: HostSize = HostSize::new(390.0, 844.0);
const LARGE_LIST_SIZE: usize = 250;
const CHURN_ITERATIONS: usize = 200;

#[test]
#[ignore = "perf"]
fn perf_large_list_update_reports_mutation_volume() {
    let mut runtime = VdomRuntime::new();
    let initial = list_view(LARGE_LIST_SIZE);
    let updated = list_view(LARGE_LIST_SIZE + 1);

    let mount_started = Instant::now();
    let first_batch = runtime.render(&initial, TEST_HOST);
    let mount_elapsed = mount_started.elapsed();

    let update_started = Instant::now();
    let second_batch = runtime.render(&updated, TEST_HOST);
    let update_elapsed = update_started.elapsed();

    println!(
        "perf_large_list_update mount_ms={} update_ms={} mount_mutations={} update_mutations={} update_layout_frames={}",
        mount_elapsed.as_millis(),
        update_elapsed.as_millis(),
        first_batch.mutations.len(),
        second_batch.mutations.len(),
        second_batch.layout.len()
    );

    assert!(!second_batch.mutations.is_empty());
}

#[test]
#[ignore = "perf"]
fn perf_create_remove_churn_reports_cycle_cost() {
    let mut runtime = VdomRuntime::new();
    let started = Instant::now();
    let mut total_mutations = 0usize;

    for iteration in 0..CHURN_ITERATIONS {
        let batch = if iteration % 2 == 0 {
            runtime.render(&list_view(24), TEST_HOST)
        } else {
            runtime.render(&empty_view(), TEST_HOST)
        };
        total_mutations += batch.mutations.len();
    }

    let elapsed = started.elapsed();
    println!(
        "perf_create_remove_churn iterations={} elapsed_ms={} total_mutations={}",
        CHURN_ITERATIONS,
        elapsed.as_millis(),
        total_mutations
    );

    assert!(total_mutations > 0);
}

fn list_view(items: usize) -> View {
    SafeArea().with_children(vec![VStack()
        .spacing(12.0)
        .padding(16.0)
        .with_children(vec![
            Text("Large List").font(Font::bold(24.0)).into_view(),
            List((0..items).collect::<Vec<_>>().into_iter(), |index| {
                HStack()
                    .spacing(8.0)
                    .padding(8.0)
                    .with_children(vec![
                        Text(format!("Row {index}")).into_view(),
                        Button("Select").on_click(|| {}).into_view(),
                    ])
                    .into_view()
            })
            .into_view(),
        ])
        .into_view()])
}

fn empty_view() -> View {
    SafeArea().with_children(vec![VStack()
        .spacing(12.0)
        .padding(16.0)
        .with_children(vec![Text("Empty").into_view()])
        .into_view()])
}
