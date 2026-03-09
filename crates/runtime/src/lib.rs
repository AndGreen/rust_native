use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use backend_api::{Backend, BackendError};
use mf_core::signal::{batch, collect_reads, signal, Setter, Signal};
use mf_core::View;
use native_schema::UiEvent;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use vdom_runtime::VdomRuntime;

type Cleanup = Box<dyn FnOnce() + Send>;
type CleanupFrame = Vec<Cleanup>;

pub struct App<B>
where
    B: Backend + Send + 'static,
{
    inner: Arc<AppInner<B>>,
}

struct AppInner<B>
where
    B: Backend + Send + 'static,
{
    backend: Mutex<B>,
    host_size: HostSize,
    builder: Arc<dyn Fn() -> View + Send + Sync>,
    vdom: Mutex<VdomRuntime>,
    subscriptions: Mutex<Vec<mf_core::signal::SignalSubscription>>,
    dirty: AtomicBool,
}

impl<B> App<B>
where
    B: Backend + Send + 'static,
{
    pub fn new<F>(backend: B, builder: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        Self::new_with_host_size(backend, HostSize::default(), builder)
    }

    pub fn new_with_host_size<F>(backend: B, host_size: HostSize, builder: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(AppInner {
                backend: Mutex::new(backend),
                host_size,
                builder: Arc::new(builder),
                vdom: Mutex::new(VdomRuntime::new()),
                subscriptions: Mutex::new(Vec::new()),
                dirty: AtomicBool::new(true),
            }),
        }
    }

    pub fn repaint(&self) {
        self.request_repaint();
        self.tick();
    }

    pub fn request_repaint(&self) {
        self.inner.dirty.store(true, Ordering::Relaxed);
    }

    pub fn tick(&self) {
        if self.inner.dirty.swap(false, Ordering::Relaxed) {
            self.render();
        } else {
            self.drain_events();
        }
    }

    fn render(&self) {
        let (next, reads) = collect_reads(|| (self.inner.builder)());
        // Re-subscribe to signals read during render.
        let mut subscriptions = self.inner.subscriptions.lock().unwrap();
        subscriptions.clear();
        let mut seen = HashSet::new();
        for handle in reads {
            if seen.insert(handle.id()) {
                let app = self.clone();
                let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    app.request_repaint();
                });
                let sub = handle.subscribe_callback(callback);
                subscriptions.push(sub);
            }
        }

        let mut vdom = self.inner.vdom.lock().unwrap();
        let batch = vdom.render(&next, self.inner.host_size);
        let mut backend = self.inner.backend.lock().unwrap();

        if !batch.mutations.is_empty() || !batch.layout.is_empty() {
            let result = backend
                .apply_mutations(&batch.mutations)
                .and_then(|_| backend.apply_layout(&batch.layout))
                .and_then(|_| backend.flush());

            match result {
                Ok(()) => {}
                Err(BackendError::BatchRejected(_)) => {
                    vdom.request_full_resync();
                    let retry = vdom.render(&next, self.inner.host_size);
                    match backend
                        .apply_mutations(&retry.mutations)
                        .and_then(|_| backend.apply_layout(&retry.layout))
                        .and_then(|_| backend.flush())
                    {
                        Ok(()) => {}
                        Err(BackendError::BatchRejected(_)) => {
                            self.request_repaint();
                            return;
                        }
                    }
                }
            }
        }

        drop(backend);
        drop(vdom);
        self.drain_events();
    }

    fn drain_events(&self) {
        let mut backend = self.inner.backend.lock().unwrap();
        let events = backend.drain_events();
        drop(backend);
        if events.is_empty() {
            return;
        }
        let vdom = self.inner.vdom.lock().unwrap();
        for event in events {
            dispatch_event(&vdom, event);
        }
    }

    /// Convenience helper: repaint immediately, then block for `duration` to allow async tasks/intervals.
    pub fn run_for(&self, duration: Duration) {
        let started = std::time::Instant::now();
        self.request_repaint();
        while started.elapsed() < duration {
            self.tick();
            thread::sleep(Duration::from_millis(16));
        }
    }

    /// Long-running loop that keeps repainting when any watched signal changes.
    /// Intended for demo/headless mode; exits only on Ctrl+C/termination.
    pub fn run(&self) {
        self.request_repaint();
        loop {
            self.tick();
            thread::sleep(Duration::from_millis(16));
        }
    }
}

impl<B> Clone for App<B>
where
    B: Backend + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub fn use_signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    signal(value)
}

pub use mf_core::signal::{Setter as SignalSetter, Signal as RuntimeSignal, SignalSubscription};
pub use vdom_runtime::HostSize;

/// Alias mirroring SolidJS nomenclature.
pub fn create_signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    use_signal(value)
}

/// Runs multiple setter calls but defers subscriptions until after `f` completes.
pub fn batch_updates<F>(f: F)
where
    F: FnOnce(),
{
    batch(f)
}

thread_local! {
    static CLEANUP_STACK: RefCell<Vec<CleanupFrame>> = RefCell::new(Vec::new());
}

/// Registers a cleanup to run when the current scope ends.
pub fn on_cleanup<F>(cleanup: F)
where
    F: FnOnce() + Send + 'static,
{
    CLEANUP_STACK.with(|stack| {
        if let Some(inner) = stack.borrow_mut().last_mut() {
            inner.push(Box::new(cleanup));
        }
    });
}

/// RAII scope used to collect `on_cleanup` callbacks similar to SolidJS.
pub struct Scope {
    cleanups: CleanupFrame,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            cleanups: Vec::new(),
        }
    }

    /// Runs `f` with a cleanup frame; any `on_cleanup` calls inside will attach to this scope.
    pub fn run<R>(&mut self, f: impl FnOnce() -> R) -> R {
        CLEANUP_STACK.with(|stack| stack.borrow_mut().push(Vec::new()));
        let result = f();
        let frame = CLEANUP_STACK
            .with(|stack| stack.borrow_mut().pop())
            .unwrap_or_default();
        self.cleanups.extend(frame);
        result
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        while let Some(cleanup) = self.cleanups.pop() {
            cleanup();
        }
    }
}

/// Starts a background interval that calls `f` every `duration`; returns a cancellation handle.
pub struct IntervalHandle {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl IntervalHandle {
    pub fn cancel(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for IntervalHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

/// Starts a background interval that calls `f` every `duration`; returns a cancellation handle.
pub fn start_interval<F>(duration: Duration, mut f: F) -> IntervalHandle
where
    F: FnMut() + Send + 'static,
{
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let join = thread::spawn(move || loop {
        thread::sleep(duration);
        if thread_stop.load(Ordering::Relaxed) {
            break;
        }
        f();
    });
    IntervalHandle {
        stop,
        join: Some(join),
    }
}

fn dispatch_event(vdom: &VdomRuntime, event: UiEvent) {
    vdom.dispatch_event(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use backend_api::Backend;
    use mf_core::view::WidgetElement;
    use mf_core::IntoView;
    use mf_core::View;
    use mf_widgets::{Button, Text};

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
                    counts: counts.clone(),
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

        fn apply_layout(
            &mut self,
            frames: &[native_schema::LayoutFrame],
        ) -> Result<(), BackendError> {
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

        let app =
            App::new_with_host_size(backend.clone(), HostSize::new(390.0, 844.0), move || {
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
}
