use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};

/// Creates a new signal/value pair.
pub fn signal<T>(value: T) -> (Signal<T>, Setter<T>)
where
    T: Send + Sync + 'static,
{
    let inner = Arc::new(SignalInner::new(value));
    (Signal { inner: inner.clone() }, Setter { inner })
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
            callback();
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
