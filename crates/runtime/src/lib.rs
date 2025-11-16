use std::sync::{Arc, Mutex};

use backend_api::Backend;
use mf_core::diff::DiffEngine;
use mf_core::signal::{signal, Setter, Signal};
use mf_core::View;

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
            }),
        }
    }

    pub fn repaint(&self) {
        let next = (self.inner.builder)();
        let mut backend = self.inner.backend.lock().unwrap();
        let mut current = self.inner.current.lock().unwrap();
        let patches = self.inner.diff.diff(current.as_ref(), &next);
        if current.is_none() {
            backend.mount(&next);
        } else if !patches.is_empty() {
            backend.update(&next);
        }
        *current = Some(next);
    }

    pub fn watch_signal<T>(&self, signal: &Signal<T>) -> mf_core::signal::SignalSubscription
    where
        T: Send + Sync + Clone + 'static,
    {
        let app = self.clone();
        signal.subscribe(move || {
            app.repaint();
        })
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
