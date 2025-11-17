use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static BATCH_QUEUE: RefCell<Option<Vec<Arc<dyn Fn() + Send + Sync>>>> = RefCell::new(None);
    static READ_TRACKER: RefCell<Option<Vec<ErasedSignalHandle>>> = RefCell::new(None);
}

/// Creates a new signal/value pair.
pub fn signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    let inner = Arc::new(SignalInner::new(value));
    (Signal { inner: inner.clone() }, Setter { inner })
}

/// Groups multiple signal updates and dispatches their notifications once after `f` completes.
pub fn batch<F>(f: F)
where
    F: FnOnce(),
{
    let already_batching = BATCH_QUEUE.with(|cell| cell.borrow().is_some());
    if already_batching {
        f();
        return;
    }

    BATCH_QUEUE.with(|cell| {
        *cell.borrow_mut() = Some(Vec::new());
    });

    f();

    let callbacks = BATCH_QUEUE.with(|cell| cell.borrow_mut().take().unwrap_or_default());
    for callback in callbacks {
        callback();
    }
}

pub struct Signal<T>
where
    T: Send + Sync + 'static,
{
    inner: Arc<SignalInner<T>>,
}

impl<T> Clone for Signal<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Signal<T>
where
    T: Send + Sync + 'static,
{
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        track_read(self.erase());
        self.inner.value.read().unwrap().clone()
    }

    pub fn subscribe<F>(&self, callback: F) -> SignalSubscription
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(callback);
        self.inner
            .observers
            .lock()
            .unwrap()
            .push(Arc::downgrade(&callback));
        SignalSubscription {
            _callback: callback,
            inner_id: self.inner.id,
        }
    }
}

fn track_read(handle: ErasedSignalHandle) {
    READ_TRACKER.with(|tracker| {
        if let Some(list) = tracker.borrow_mut().as_mut() {
            // Dedup by id to keep the list small.
            let exists = list.iter().any(|h| h.id() == handle.id());
            if !exists {
                list.push(handle);
            }
        }
    });
}

/// Runs `f`, collecting any signals read via `get()`. Returns the result and the unique set of signals read.
pub fn collect_reads<R>(f: impl FnOnce() -> R) -> (R, Vec<ErasedSignalHandle>) {
    READ_TRACKER.with(|tracker| tracker.replace(Some(Vec::new())));
    let result = f();
    let reads = READ_TRACKER.with(|tracker| tracker.replace(None).unwrap_or_default());
    // Defensive dedup in case of nested tracking scopes.
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for handle in reads {
        if seen.insert(handle.id()) {
            unique.push(handle);
        }
    }
    (result, unique)
}

pub trait ErasedSignal: Send + Sync {
    fn id(&self) -> u64;
    fn subscribe_callback(&self, callback: Arc<dyn Fn() + Send + Sync>) -> SignalSubscription;
}

#[derive(Clone)]
pub struct ErasedSignalHandle {
    inner: Arc<dyn ErasedSignal>,
}

impl ErasedSignalHandle {
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    pub fn subscribe_callback(&self, callback: Arc<dyn Fn() + Send + Sync>) -> SignalSubscription {
        self.inner.subscribe_callback(callback)
    }
}

impl<T> ErasedSignal for Signal<T>
where
    T: Send + Sync + 'static,
{
    fn id(&self) -> u64 {
        self.inner.id
    }

    fn subscribe_callback(&self, callback: Arc<dyn Fn() + Send + Sync>) -> SignalSubscription {
        self.inner
            .observers
            .lock()
            .unwrap()
            .push(Arc::downgrade(&callback));
        SignalSubscription {
            _callback: callback,
            inner_id: self.inner.id,
        }
    }
}

impl<T> Signal<T>
where
    T: Send + Sync + 'static,
{
    pub fn erase(&self) -> ErasedSignalHandle {
        ErasedSignalHandle {
            inner: Arc::new(self.clone()),
        }
    }
}

pub struct Setter<T>
where
    T: Send + Sync + 'static,
{
    inner: Arc<SignalInner<T>>,
}

impl<T> Clone for Setter<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Setter<T>
where
    T: Send + Sync + 'static,
{
    pub fn set(&self, value: T) {
        *self.inner.value.write().unwrap() = value;
        self.inner.notify();
    }

    pub fn update<F>(&self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let mut guard = self.inner.value.write().unwrap();
        f(&mut guard);
        drop(guard);
        self.inner.notify();
    }
}

struct SignalInner<T>
where
    T: Send + Sync + 'static,
{
    id: u64,
    value: RwLock<T>,
    observers: Mutex<Vec<Weak<dyn Fn() + Send + Sync>>>,
}

impl<T> SignalInner<T>
where
    T: Send + Sync + 'static,
{
    fn new(value: T) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            value: RwLock::new(value),
            observers: Mutex::new(Vec::new()),
        }
    }

    fn notify(&self) {
        let mut callbacks = Vec::new();
        let mut gc = Vec::new();
        {
            let mut locked = self.observers.lock().unwrap();
            for (index, observer) in locked.iter().enumerate() {
                if let Some(callback) = observer.upgrade() {
                    callbacks.push(callback);
                } else {
                    gc.push(index);
                }
            }
            for index in gc.into_iter().rev() {
                locked.remove(index);
            }
        }
        for callback in callbacks {
            let queued = BATCH_QUEUE.with(|cell| {
                if let Some(queue) = cell.borrow_mut().as_mut() {
                    queue.push(callback.clone());
                    true
                } else {
                    false
                }
            });
            if !queued {
                callback();
            }
        }
    }
}

#[derive(Clone)]
pub struct SignalSubscription {
    _callback: Arc<dyn Fn() + Send + Sync>,
    inner_id: u64,
}

impl SignalSubscription {
    pub fn id(&self) -> u64 {
        self.inner_id
    }
}
