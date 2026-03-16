use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use backend_api::{Backend, BackendError};
use mf_core::signal::{batch, collect_reads, signal, Setter, Signal};
use mf_core::View;
use native_schema::UiEvent;
use vdom_runtime::{HostSize, VdomRuntime};

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
    host_size: Mutex<HostSize>,
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
                host_size: Mutex::new(host_size),
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

    pub fn request_full_resync(&self) {
        self.inner.vdom.lock().unwrap().request_full_resync();
        self.request_repaint();
    }

    pub fn set_host_size(&self, host_size: HostSize) {
        let mut current = self.inner.host_size.lock().unwrap();
        if *current != host_size {
            *current = host_size;
            self.request_repaint();
        }
    }

    pub fn tick(&self) {
        if self.inner.dirty.swap(false, Ordering::Relaxed) {
            self.render();
        } else {
            self.drain_events();
        }
    }

    pub fn run_for(&self, duration: Duration) {
        let started = std::time::Instant::now();
        self.request_repaint();
        while started.elapsed() < duration {
            self.tick();
            thread::sleep(Duration::from_millis(16));
        }
    }

    pub fn run(&self) {
        self.request_repaint();
        loop {
            self.tick();
            thread::sleep(Duration::from_millis(16));
        }
    }

    fn render(&self) {
        let (next, reads) = collect_reads(|| (self.inner.builder)());
        self.refresh_subscriptions(reads);

        let host_size = *self.inner.host_size.lock().unwrap();
        let mut vdom = self.inner.vdom.lock().unwrap();
        let batch = vdom.render(&next, host_size);
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
                    let retry = vdom.render(&next, host_size);
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

    fn refresh_subscriptions(&self, reads: Vec<mf_core::signal::ErasedSignalHandle>) {
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
}

impl<B> Clone for App<B>
where
    B: Backend + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub fn use_signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    signal(value)
}

pub fn create_signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    use_signal(value)
}

pub fn batch_updates<F>(f: F)
where
    F: FnOnce(),
{
    batch(f)
}

fn dispatch_event(vdom: &VdomRuntime, event: UiEvent) {
    vdom.dispatch_event(event);
}
