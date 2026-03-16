use std::cell::RefCell;

type Cleanup = Box<dyn FnOnce() + Send>;
type CleanupFrame = Vec<Cleanup>;

thread_local! {
    static CLEANUP_STACK: RefCell<Vec<CleanupFrame>> = RefCell::new(Vec::new());
}

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

pub struct Scope {
    cleanups: CleanupFrame,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            cleanups: Vec::new(),
        }
    }

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
