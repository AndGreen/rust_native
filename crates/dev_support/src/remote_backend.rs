use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

use backend_api::{Backend, BackendError};
use dev_protocol::{read_json_line, write_json_line, WorkerControlMessage, WorkerOutputMessage};
use mf_runtime::HostSize;
use native_schema::{LayoutFrame, Mutation, ProtocolVersion, UiEvent};

#[derive(Default)]
struct RemoteState {
    events: Vec<UiEvent>,
    pending_host: Option<HostSize>,
    repaint: bool,
    full_resync: bool,
    shutdown: bool,
}

pub struct RemoteBackend {
    writer: Arc<Mutex<std::io::Stdout>>,
    state: Arc<Mutex<RemoteState>>,
    pending_mutations: Vec<Mutation>,
    pending_layout: Vec<LayoutFrame>,
}

#[derive(Clone)]
pub struct RemoteBackendHandle {
    state: Arc<Mutex<RemoteState>>,
}

impl RemoteBackend {
    pub fn stdio() -> (Self, RemoteBackendHandle) {
        let state = Arc::new(Mutex::new(RemoteState::default()));
        let handle = RemoteBackendHandle {
            state: Arc::clone(&state),
        };
        spawn_worker_control_reader(Arc::clone(&state));

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
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        self.pending_mutations.extend_from_slice(mutations);
        Ok(())
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        self.pending_layout.extend_from_slice(frames);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        if self.pending_mutations.is_empty() && self.pending_layout.is_empty() {
            return Ok(());
        }

        let payload = WorkerOutputMessage::RenderBatch {
            protocol_version: ProtocolVersion::V1,
            mutations: std::mem::take(&mut self.pending_mutations),
            layout: std::mem::take(&mut self.pending_layout),
        };

        let mut writer = self.writer.lock().unwrap();
        write_json_line(&mut *writer, &payload).map_err(|error| {
            BackendError::BatchRejected(format!("failed to write worker batch: {error}"))
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

    pub fn take_full_resync(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let full_resync = state.full_resync;
        state.full_resync = false;
        full_resync
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
                    state.lock().unwrap().pending_host = Some(HostSize::with_safe_area(
                        host.width,
                        host.height,
                        host.safe_area,
                    ));
                }
                Ok(Some(WorkerControlMessage::UiEvent(event))) => {
                    state.lock().unwrap().events.push(event);
                }
                Ok(Some(WorkerControlMessage::RequestRepaint)) => {
                    state.lock().unwrap().repaint = true;
                }
                Ok(Some(WorkerControlMessage::RequestFullResync)) => {
                    state.lock().unwrap().full_resync = true;
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
