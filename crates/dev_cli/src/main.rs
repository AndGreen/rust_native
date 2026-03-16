use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use dev_protocol::{
    read_json_line, write_json_line, ClientMessage, HostMetrics, ServerMessage, WorkerOutputMessage,
};
use dev_support::{request_worker_full_resync, send_worker_control, worker_control_from_host};
use native_schema::{EdgeInsets, ProtocolVersion};

struct SharedState {
    app_id: String,
    client: Option<Arc<Mutex<TcpStream>>>,
    worker_stdin: Option<Arc<Mutex<ChildStdin>>>,
    host: HostMetrics,
}

fn main() -> Result<(), String> {
    let config = Config::from_env()?;
    let shared = Arc::new(Mutex::new(SharedState {
        app_id: config.app_id.clone(),
        client: None,
        worker_stdin: None,
        host: config.default_host,
    }));

    let listener = TcpListener::bind((config.bind_host.as_str(), config.port))
        .map_err(|error| format!("failed to bind dev server: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure listener: {error}"))?;

    let accept_state = shared.clone();
    thread::spawn(move || accept_loop(listener, accept_state));

    let mut worker = build_and_spawn(&config, &shared)?;
    let mut watched = collect_watch_snapshot(&config.workspace_root);

    loop {
        thread::sleep(Duration::from_millis(350));
        let next = collect_watch_snapshot(&config.workspace_root);
        if next != watched {
            watched = next;
            if let Err(error) = reload_worker(&config, &shared, &mut worker) {
                broadcast(
                    &shared,
                    &ServerMessage::BuildFailed {
                        message: error.clone(),
                    },
                );
                eprintln!("{error}");
            }
        }
    }
}

#[derive(Clone)]
struct Config {
    app_id: String,
    bind_host: String,
    port: u16,
    workspace_root: PathBuf,
    default_host: HostMetrics,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let mut args = env::args().skip(1);
        let mut app_id = None;
        let mut bind_host = "127.0.0.1".to_string();
        let mut port = 4488u16;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app" => app_id = args.next(),
                "--host" => bind_host = args.next().unwrap_or(bind_host),
                "--port" => {
                    let raw = args
                        .next()
                        .ok_or_else(|| "missing value after --port".to_string())?;
                    port = raw
                        .parse()
                        .map_err(|_| "invalid --port value".to_string())?;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        let app_id =
            app_id.ok_or_else(|| "usage: cargo run -p dev_cli -- --app counter".to_string())?;
        let workspace_root = env::current_dir().map_err(|error| error.to_string())?;
        Ok(Self {
            app_id,
            bind_host,
            port,
            workspace_root,
            default_host: HostMetrics::new(390.0, 844.0, EdgeInsets::all(0.0)),
        })
    }
}

fn reload_worker(
    config: &Config,
    shared: &Arc<Mutex<SharedState>>,
    worker: &mut Child,
) -> Result<(), String> {
    broadcast(shared, &ServerMessage::Reloading);

    let output = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg(&config.app_id)
        .current_dir(&config.workspace_root)
        .output()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let _ = send_shutdown(shared);
    let _ = worker.kill();
    let _ = worker.wait();
    *worker = spawn_worker_process(config, shared)?;
    Ok(())
}

fn build_and_spawn(config: &Config, shared: &Arc<Mutex<SharedState>>) -> Result<Child, String> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg(&config.app_id)
        .current_dir(&config.workspace_root)
        .output()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    spawn_worker_process(config, shared)
}

fn spawn_worker_process(
    config: &Config,
    shared: &Arc<Mutex<SharedState>>,
) -> Result<Child, String> {
    broadcast(shared, &ServerMessage::Reloading);
    broadcast(shared, &ServerMessage::ResetUi);

    let host_json = {
        let state = shared.lock().unwrap();
        serde_json::to_string(&state.host).map_err(|error| error.to_string())?
    };
    let binary = config
        .workspace_root
        .join("target")
        .join("debug")
        .join(&config.app_id);
    let mut child = Command::new(binary)
        .current_dir(&config.workspace_root)
        .env("MF_DEV_REMOTE_WORKER", "1")
        .env("MF_DEV_HOST_METRICS", host_json)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to spawn worker: {error}"))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "worker stdin is unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "worker stdout is unavailable".to_string())?;

    {
        let mut state = shared.lock().unwrap();
        state.worker_stdin = Some(Arc::new(Mutex::new(stdin)));
    }

    let read_state = shared.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            match read_json_line::<WorkerOutputMessage>(&mut reader) {
                Ok(Some(WorkerOutputMessage::RenderBatch {
                    protocol_version,
                    mutations,
                    layout,
                })) => {
                    broadcast(
                        &read_state,
                        &ServerMessage::RenderBatch {
                            protocol_version,
                            mutations,
                            layout,
                        },
                    );
                }
                Ok(None) => break,
                Err(error) => {
                    eprintln!("worker output error: {error}");
                    break;
                }
            }
        }
    });

    let worker_stdin = {
        let state = shared.lock().unwrap();
        state.worker_stdin.clone()
    };
    let host = {
        let state = shared.lock().unwrap();
        state.host
    };
    if let Some(writer) = worker_stdin {
        let _ = send_worker_control(&writer, &worker_control_from_host(host));
    }

    Ok(child)
}

fn send_shutdown(shared: &Arc<Mutex<SharedState>>) -> io::Result<()> {
    let writer = {
        let state = shared.lock().unwrap();
        state.worker_stdin.clone()
    };
    if let Some(writer) = writer {
        send_worker_control(&writer, &dev_protocol::WorkerControlMessage::Shutdown)?;
    }
    Ok(())
}

fn accept_loop(listener: TcpListener, shared: Arc<Mutex<SharedState>>) {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(error) = stream.set_nonblocking(false) {
                    eprintln!("failed to switch client stream to blocking mode: {error}");
                    continue;
                }
                let writer = match stream.try_clone() {
                    Ok(writer) => Arc::new(Mutex::new(writer)),
                    Err(error) => {
                        eprintln!("failed to clone client stream: {error}");
                        continue;
                    }
                };
                {
                    let mut state = shared.lock().unwrap();
                    state.client = Some(writer.clone());
                }
                let app_id = {
                    let state = shared.lock().unwrap();
                    state.app_id.clone()
                };
                let _ = write_json_line(
                    &mut *writer.lock().unwrap(),
                    &ServerMessage::HelloAck {
                        app_id,
                        protocol_version: ProtocolVersion::V1,
                    },
                );
                let read_state = shared.clone();
                thread::spawn(move || handle_client(stream, read_state));
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                eprintln!("listener error: {error}");
                thread::sleep(Duration::from_millis(250));
            }
        }
    }
}

fn handle_client(stream: TcpStream, shared: Arc<Mutex<SharedState>>) {
    let mut reader = BufReader::new(stream);
    loop {
        match read_json_line::<ClientMessage>(&mut reader) {
            Ok(Some(ClientMessage::Hello { host, .. })) => {
                update_host(shared.clone(), host, true);
            }
            Ok(Some(ClientMessage::HostResized { host })) => {
                update_host(shared.clone(), host, false);
            }
            Ok(Some(ClientMessage::UiEvent(event))) => {
                let writer = {
                    let state = shared.lock().unwrap();
                    state.worker_stdin.clone()
                };
                if let Some(writer) = writer {
                    let _ = send_worker_control(
                        &writer,
                        &dev_protocol::WorkerControlMessage::UiEvent(event),
                    );
                }
            }
            Ok(Some(ClientMessage::Ping)) => {}
            Ok(None) => break,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(16));
            }
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                eprintln!("ignoring malformed client message: {error}");
                thread::sleep(Duration::from_millis(16));
            }
            Err(error) => {
                eprintln!("client read error: {error}");
                break;
            }
        }
    }
}

fn update_host(shared: Arc<Mutex<SharedState>>, host: HostMetrics, force_full_resync: bool) {
    let writer = {
        let mut state = shared.lock().unwrap();
        state.host = host;
        state.worker_stdin.clone()
    };
    if let Some(writer) = writer {
        let _ = send_worker_control(&writer, &worker_control_from_host(host));
        if force_full_resync {
            broadcast(&shared, &ServerMessage::ResetUi);
            let _ = request_worker_full_resync(&writer);
        }
    }
}

fn broadcast(shared: &Arc<Mutex<SharedState>>, message: &ServerMessage) {
    let client = {
        let state = shared.lock().unwrap();
        state.client.clone()
    };
    if let Some(client) = client {
        let _ = write_json_line(&mut *client.lock().unwrap(), message);
    }
}

fn collect_watch_snapshot(root: &Path) -> HashMap<PathBuf, SystemTime> {
    let mut snapshot = HashMap::new();
    collect_watch_snapshot_inner(root, &mut snapshot);
    snapshot
}

fn collect_watch_snapshot_inner(root: &Path, snapshot: &mut HashMap<PathBuf, SystemTime>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("target") {
            continue;
        }
        if path.is_dir() {
            collect_watch_snapshot_inner(&path, snapshot);
            continue;
        }

        let watch = matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("rs") | Some("toml") | Some("swift") | Some("h")
        );
        if !watch {
            continue;
        }

        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                snapshot.insert(path, modified);
            }
        }
    }
}
