use std::env;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use dev_protocol::{write_json_line, HostMetrics, WorkerControlMessage};
use mf_runtime::{App, HostSize};

use crate::remote_backend::RemoteBackend;

pub fn run_worker<F>(factory: F) -> Result<(), String>
where
    F: Fn(RemoteBackend, HostSize) -> App<RemoteBackend>,
{
    let initial_host = initial_host_from_env();
    let (backend, handle) = RemoteBackend::stdio();
    let app = factory(backend, initial_host);
    app.repaint();

    loop {
        if let Some(host) = handle.take_pending_host() {
            app.set_host_size(host);
        }
        if handle.take_full_resync() {
            app.request_full_resync();
        }
        if handle.take_repaint() {
            app.request_repaint();
        }
        if handle.is_shutdown() {
            break;
        }
        app.tick();
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

pub fn request_worker_repaint(writer: &Arc<Mutex<std::process::ChildStdin>>) -> io::Result<()> {
    send_worker_control(writer, &WorkerControlMessage::RequestRepaint)
}

pub fn request_worker_full_resync(writer: &Arc<Mutex<std::process::ChildStdin>>) -> io::Result<()> {
    send_worker_control(writer, &WorkerControlMessage::RequestFullResync)
}

pub fn worker_control_from_host(host: HostMetrics) -> WorkerControlMessage {
    WorkerControlMessage::SetHostMetrics { host }
}

pub fn send_worker_control(
    writer: &Arc<Mutex<std::process::ChildStdin>>,
    message: &WorkerControlMessage,
) -> io::Result<()> {
    let mut writer = writer.lock().unwrap();
    write_json_line(&mut *writer, message)
}

fn initial_host_from_env() -> HostSize {
    match env::var("MF_DEV_HOST_METRICS") {
        Ok(value) => serde_json::from_str::<HostMetrics>(&value)
            .map(|host| HostSize::with_safe_area(host.width, host.height, host.safe_area))
            .unwrap_or_default(),
        Err(_) => HostSize::default(),
    }
}
