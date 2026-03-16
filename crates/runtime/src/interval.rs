use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

pub fn start_interval<F>(duration: Duration, mut f: F) -> IntervalHandle
where
    F: FnMut() + Send + 'static,
{
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
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
