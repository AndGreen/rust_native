use std::sync::{Arc, Mutex};
use std::thread;

use backend_api::{Backend, BackendError};
use mf_core::view::WidgetElement;
use mf_core::{IntoView, View, WithChildren};
use mf_widgets::{Button, SafeArea, Text};
use native_schema::UiEvent;

use crate::{create_signal, App, HostSize};

#[derive(Default, Clone)]
struct Counts {
    apply_mutations: usize,
    apply_layout: usize,
    flushes: usize,
    last_mutation_count: usize,
    last_layout_count: usize,
}

#[derive(Clone)]
struct TestBackend {
    counts: Arc<Mutex<Counts>>,
    pending_events: Arc<Mutex<Vec<UiEvent>>>,
    reject_once: Arc<Mutex<bool>>,
    emit_tap_once: Arc<Mutex<bool>>,
}

impl TestBackend {
    fn new() -> (Self, Arc<Mutex<Counts>>) {
        let counts = Arc::new(Mutex::new(Counts::default()));
        (
            Self {
                counts: Arc::clone(&counts),
                pending_events: Arc::new(Mutex::new(Vec::new())),
                reject_once: Arc::new(Mutex::new(false)),
                emit_tap_once: Arc::new(Mutex::new(false)),
            },
            counts,
        )
    }
}

impl Backend for TestBackend {
    fn apply_mutations(
        &mut self,
        mutations: &[native_schema::Mutation],
    ) -> Result<(), BackendError> {
        self.counts.lock().unwrap().apply_mutations += 1;
        self.counts.lock().unwrap().last_mutation_count = mutations.len();

        let mut emit_tap_once = self.emit_tap_once.lock().unwrap();
        if *emit_tap_once {
            if let Some(id) = mutations.iter().find_map(|mutation| match mutation {
                native_schema::Mutation::AttachEventListener {
                    id,
                    event: native_schema::EventKind::Tap,
                } => Some(*id),
                _ => None,
            }) {
                self.pending_events
                    .lock()
                    .unwrap()
                    .push(UiEvent::Tap { id });
                *emit_tap_once = false;
            }
        }

        let mut reject_once = self.reject_once.lock().unwrap();
        if *reject_once {
            *reject_once = false;
            return Err(BackendError::BatchRejected("retry".into()));
        }

        Ok(())
    }

    fn apply_layout(&mut self, frames: &[native_schema::LayoutFrame]) -> Result<(), BackendError> {
        self.counts.lock().unwrap().apply_layout += 1;
        self.counts.lock().unwrap().last_layout_count = frames.len();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        self.counts.lock().unwrap().flushes += 1;
        Ok(())
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        self.pending_events.lock().unwrap().drain(..).collect()
    }
}

struct TestElement(&'static str);

impl WidgetElement for TestElement {
    fn name(&self) -> &'static str {
        self.0
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn node(name: &'static str) -> View {
    View::new(TestElement(name), Vec::new())
}

#[test]
fn first_repaint_emits_mutations_and_flushes() {
    let (backend, counts) = TestBackend::new();
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || node("Root"));

    app.repaint();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 1);
    assert_eq!(snapshot.apply_layout, 1);
    assert_eq!(snapshot.last_layout_count, 1);
    assert_eq!(snapshot.flushes, 1);
}

#[test]
fn repaint_without_tree_changes_skips_backend_calls() {
    let (backend, counts) = TestBackend::new();
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || node("Root"));

    app.repaint();
    app.repaint();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 1);
    assert_eq!(snapshot.flushes, 1);
}

#[test]
fn full_resync_forces_new_batch_without_tree_changes() {
    let (backend, counts) = TestBackend::new();
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || node("Root"));

    app.repaint();
    app.request_full_resync();
    app.tick();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
    assert_eq!(snapshot.apply_layout, 2);
    assert_eq!(snapshot.flushes, 2);
    assert_eq!(snapshot.last_layout_count, 1);
}

#[test]
fn signal_change_that_alters_tree_shape_triggers_batch_update() {
    let (backend, counts) = TestBackend::new();
    let (state, set_state) = create_signal(false);

    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), move || {
        if state.get() {
            Button("Tap").into_view()
        } else {
            Text("Value").into_view()
        }
    });

    app.repaint();
    set_state.set(true);
    app.tick();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
    assert_eq!(snapshot.flushes, 2);
}

#[test]
fn batch_rejection_triggers_full_resync_retry() {
    let (backend, counts) = TestBackend::new();
    *backend.reject_once.lock().unwrap() = true;
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || node("Root"));

    app.repaint();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
    assert_eq!(snapshot.flushes, 1);
}

#[test]
fn drained_events_are_dispatched() {
    let (backend, _counts) = TestBackend::new();
    let (value, set_value) = create_signal(0usize);
    *backend.emit_tap_once.lock().unwrap() = true;

    let app = App::new_with_host_size(backend.clone(), HostSize::new(390.0, 844.0), move || {
        let setter = set_value.clone();
        Button("Tap")
            .on_click(move || setter.update(|current| *current += 1))
            .into_view()
    });

    app.repaint();
    assert_eq!(value.get(), 1);
}

#[test]
fn signal_change_marks_app_dirty_until_host_ticks() {
    let (backend, counts) = TestBackend::new();
    let (state, set_state) = create_signal(false);
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), move || {
        if state.get() {
            Text("Dirty").into_view()
        } else {
            Text("Clean").into_view()
        }
    });

    app.repaint();
    let worker = thread::spawn(move || set_state.set(true));
    worker.join().unwrap();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 1);

    app.tick();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
}

#[test]
fn host_resize_triggers_layout_refresh() {
    let (backend, counts) = TestBackend::new();
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || node("Root"));

    app.repaint();
    app.set_host_size(HostSize::new(844.0, 390.0));
    app.tick();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
    assert_eq!(snapshot.apply_layout, 2);
    assert_eq!(snapshot.flushes, 2);
    assert_eq!(snapshot.last_mutation_count, 0);
    assert_eq!(snapshot.last_layout_count, 1);
}

#[test]
fn safe_area_change_triggers_layout_refresh() {
    let (backend, counts) = TestBackend::new();
    let app = App::new_with_host_size(backend, HostSize::new(390.0, 844.0), || {
        SafeArea().with_children(vec![node("Root").into_view()])
    });

    app.repaint();
    app.set_host_size(HostSize::with_safe_area(
        390.0,
        844.0,
        native_schema::EdgeInsets::new(59.0, 0.0, 34.0, 0.0),
    ));
    app.tick();

    let snapshot = counts.lock().unwrap().clone();
    assert_eq!(snapshot.apply_mutations, 2);
    assert_eq!(snapshot.apply_layout, 2);
    assert_eq!(snapshot.flushes, 2);
    assert_eq!(snapshot.last_mutation_count, 0);
    assert_eq!(snapshot.last_layout_count, 2);
}
