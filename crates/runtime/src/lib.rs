use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use backend_api::Backend;
use mf_core::diff::DiffEngine;
use mf_core::signal::{batch, collect_reads, signal, Setter, Signal};
use mf_core::View;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

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
    builder: Arc<dyn Fn() -> View + Send + Sync>,
    current: Mutex<Option<View>>,
    diff: DiffEngine,
    subscriptions: Mutex<Vec<mf_core::signal::SignalSubscription>>,
}

impl<B> App<B>
where
    B: Backend + Send + 'static,
{
    pub fn new<F>(backend: B, builder: F) -> Self
    where
        F: Fn() -> View + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(AppInner {
                backend: Mutex::new(backend),
                builder: Arc::new(builder),
                current: Mutex::new(None),
                diff: DiffEngine::new(),
                subscriptions: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn repaint(&self) {
        let (next, reads) = collect_reads(|| (self.inner.builder)());
        let mut backend = self.inner.backend.lock().unwrap();
        let mut current = self.inner.current.lock().unwrap();

        // Re-subscribe to signals read during render.
        let mut subscriptions = self.inner.subscriptions.lock().unwrap();
        subscriptions.clear();
        let mut seen = HashSet::new();
        for handle in reads {
            if seen.insert(handle.id()) {
                let app = self.clone();
                let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    app.repaint();
                });
                let sub = handle.subscribe_callback(callback);
                subscriptions.push(sub);
            }
        }

        let patches = self.inner.diff.diff(current.as_ref(), &next);
        if current.is_none() {
            backend.mount(&next);
        } else if !patches.is_empty() {
            backend.update(&next);
        }
        *current = Some(next);
    }

    /// Convenience helper: repaint immediately, then block for `duration` to allow async tasks/intervals.
    pub fn run_for(&self, duration: Duration) {
        self.repaint();
        thread::sleep(duration);
    }

    /// Long-running loop that keeps repainting when any watched signal changes.
    /// Intended for demo/headless mode; exits only on Ctrl+C/termination.
    pub fn run(&self) {
        self.repaint();
        loop {
            thread::sleep(Duration::from_secs(1));
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

pub use mf_core::signal::{
    Setter as SignalSetter, Signal as RuntimeSignal, SignalSubscription,
};

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
    static CLEANUP_STACK: RefCell<Vec<Vec<Box<dyn FnOnce() + Send>>>> = RefCell::new(Vec::new());
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
    cleanups: Vec<Box<dyn FnOnce() + Send>>,
}

impl Scope {
    pub fn new() -> Self {
        Self { cleanups: Vec::new() }
    }

    /// Runs `f` with a cleanup frame; any `on_cleanup` calls inside will attach to this scope.
    pub fn run<R>(&mut self, f: impl FnOnce() -> R) -> R {
        CLEANUP_STACK.with(|stack| stack.borrow_mut().push(Vec::new()));
        let result = f();
        let frame = CLEANUP_STACK.with(|stack| stack.borrow_mut().pop()).unwrap_or_default();
        self.cleanups.extend(frame);
        result
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
