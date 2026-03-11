use std::env;
use std::ffi::{c_char, CStr, CString};
use std::io::{self, BufReader};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use backend_api::Backend;
use backend_native::NativeBackend;
use dev_protocol::{
    read_json_line, write_json_line, HostMetrics, ServerMessage, WorkerControlMessage,
    WorkerOutputMessage,
};
use mf_runtime::{App, HostSize};
use native_schema::UiEvent;

#[derive(Default)]
struct RemoteState {
    events: Vec<UiEvent>,
    pending_host: Option<HostSize>,
    repaint: bool,
    shutdown: bool,
}

pub struct RemoteBackend {
    writer: Arc<Mutex<std::io::Stdout>>,
    state: Arc<Mutex<RemoteState>>,
    pending_mutations: Vec<native_schema::Mutation>,
    pending_layout: Vec<native_schema::LayoutFrame>,
}

#[derive(Clone)]
pub struct RemoteBackendHandle {
    state: Arc<Mutex<RemoteState>>,
}

impl RemoteBackend {
    pub fn stdio() -> (Self, RemoteBackendHandle) {
        let state = Arc::new(Mutex::new(RemoteState::default()));
        let handle = RemoteBackendHandle {
            state: state.clone(),
        };
        spawn_worker_control_reader(state.clone());
        (
            Self {
                writer: Arc::new(Mutex::new(std::io::stdout())),
                state,
                pending_mutations: Vec::new(),
                pending_layout: Vec::new(),
            },
            handle,
        )
    }
}

impl Backend for RemoteBackend {
    fn apply_mutations(
        &mut self,
        mutations: &[native_schema::Mutation],
    ) -> Result<(), backend_api::BackendError> {
        self.pending_mutations.extend_from_slice(mutations);
        Ok(())
    }

    fn apply_layout(
        &mut self,
        frames: &[native_schema::LayoutFrame],
    ) -> Result<(), backend_api::BackendError> {
        self.pending_layout.extend_from_slice(frames);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), backend_api::BackendError> {
        if self.pending_mutations.is_empty() && self.pending_layout.is_empty() {
            return Ok(());
        }

        let payload = WorkerOutputMessage::RenderBatch {
            protocol_version: native_schema::ProtocolVersion::V1,
            mutations: std::mem::take(&mut self.pending_mutations),
            layout: std::mem::take(&mut self.pending_layout),
        };

        let mut writer = self.writer.lock().unwrap();
        write_json_line(&mut *writer, &payload).map_err(|error| {
            backend_api::BackendError::BatchRejected(format!(
                "failed to write worker batch: {error}"
            ))
        })
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.events)
    }
}

impl RemoteBackendHandle {
    pub fn take_pending_host(&self) -> Option<HostSize> {
        self.state.lock().unwrap().pending_host.take()
    }

    pub fn take_repaint(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let repaint = state.repaint;
        state.repaint = false;
        repaint
    }

    pub fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }
}

fn spawn_worker_control_reader(state: Arc<Mutex<RemoteState>>) {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        loop {
            match read_json_line::<WorkerControlMessage>(&mut reader) {
                Ok(Some(WorkerControlMessage::SetHostMetrics { host })) => {
                    state.lock().unwrap().pending_host =
                        Some(HostSize::with_safe_area(host.width, host.height, host.safe_area));
                }
                Ok(Some(WorkerControlMessage::UiEvent(event))) => {
                    state.lock().unwrap().events.push(event);
                }
                Ok(Some(WorkerControlMessage::RequestRepaint)) => {
                    state.lock().unwrap().repaint = true;
                }
                Ok(Some(WorkerControlMessage::Shutdown)) => {
                    state.lock().unwrap().shutdown = true;
                    break;
                }
                Ok(None) => break,
                Err(_) => {
                    state.lock().unwrap().shutdown = true;
                    break;
                }
            }
        }
    });
}

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

fn initial_host_from_env() -> HostSize {
    match env::var("MF_DEV_HOST_METRICS") {
        Ok(value) => serde_json::from_str::<HostMetrics>(&value)
            .map(|host| HostSize::with_safe_area(host.width, host.height, host.safe_area))
            .unwrap_or_default(),
        Err(_) => HostSize::default(),
    }
}

pub fn request_worker_repaint(writer: &Arc<Mutex<std::process::ChildStdin>>) -> io::Result<()> {
    send_worker_control(writer, &WorkerControlMessage::RequestRepaint)
}

struct DevRenderer {
    backend: NativeBackend,
}

impl Default for DevRenderer {
    fn default() -> Self {
        Self {
            backend: NativeBackend::default(),
        }
    }
}

static DEV_RENDERER: OnceLock<Mutex<DevRenderer>> = OnceLock::new();
static DEV_RENDERER_EVENTS: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

fn renderer_slot() -> &'static Mutex<DevRenderer> {
    DEV_RENDERER.get_or_init(|| Mutex::new(DevRenderer::default()))
}

fn renderer_events_slot() -> &'static Mutex<Option<CString>> {
    DEV_RENDERER_EVENTS.get_or_init(|| Mutex::new(None))
}

pub fn mf_dev_renderer_reset() {
    let mut renderer = renderer_slot().lock().unwrap();
    *renderer = DevRenderer::default();
}

pub fn mf_dev_renderer_apply_message(json: *const c_char) -> bool {
    if json.is_null() {
        return false;
    }

    let payload = unsafe { CStr::from_ptr(json) };
    let Ok(payload) = payload.to_str() else {
        return false;
    };
    let Ok(message) = serde_json::from_str::<ServerMessage>(payload) else {
        return false;
    };

    match message {
        ServerMessage::ResetUi => {
            mf_dev_renderer_reset();
            true
        }
        ServerMessage::RenderBatch {
            mutations, layout, ..
        } => {
            let mut renderer = renderer_slot().lock().unwrap();
            renderer
                .backend
                .apply_mutations(&mutations)
                .and_then(|_| renderer.backend.apply_layout(&layout))
                .and_then(|_| renderer.backend.flush())
                .is_ok()
        }
        ServerMessage::Reloading | ServerMessage::BuildFailed { .. } | ServerMessage::HelloAck { .. } => true,
    }
}

pub fn mf_dev_renderer_take_events_json() -> *const c_char {
    let mut renderer = renderer_slot().lock().unwrap();
    let events = renderer.backend.drain_events();
    let serialized = match serde_json::to_string(&events) {
        Ok(value) => value,
        Err(_) => "[]".to_string(),
    };
    let cstring = CString::new(serialized).unwrap_or_else(|_| CString::new("[]").unwrap());
    let ptr = cstring.as_ptr();
    *renderer_events_slot().lock().unwrap() = Some(cstring);
    ptr
}

pub fn mf_dev_renderer_clear_events_json() {
    *renderer_events_slot().lock().unwrap() = None;
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
