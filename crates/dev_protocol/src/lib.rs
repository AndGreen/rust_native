use std::io::{self, BufRead, Write};

use native_schema::{EdgeInsets, LayoutFrame, Mutation, ProtocolVersion, UiEvent};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HostMetrics {
    pub width: f32,
    pub height: f32,
    pub safe_area: EdgeInsets,
}

impl HostMetrics {
    pub const fn new(width: f32, height: f32, safe_area: EdgeInsets) -> Self {
        Self {
            width,
            height,
            safe_area,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkerControlMessage {
    SetHostMetrics { host: HostMetrics },
    UiEvent(UiEvent),
    RequestRepaint,
    RequestFullResync,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkerOutputMessage {
    RenderBatch {
        protocol_version: ProtocolVersion,
        mutations: Vec<Mutation>,
        layout: Vec<LayoutFrame>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello { app_id: String, host: HostMetrics },
    HostResized { host: HostMetrics },
    UiEvent(UiEvent),
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    HelloAck {
        app_id: String,
        protocol_version: ProtocolVersion,
    },
    Reloading,
    ResetUi,
    RenderBatch {
        protocol_version: ProtocolVersion,
        mutations: Vec<Mutation>,
        layout: Vec<LayoutFrame>,
    },
    BuildFailed {
        message: String,
    },
}

pub fn write_json_line<T>(writer: &mut impl Write, value: &T) -> io::Result<()>
where
    T: Serialize,
{
    serde_json::to_writer(&mut *writer, value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    writer.write_all(b"\n")?;
    writer.flush()
}

pub fn read_json_line<T>(reader: &mut impl BufRead) -> io::Result<Option<T>>
where
    T: DeserializeOwned,
{
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = serde_json::from_str(trimmed)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        return Ok(Some(value));
    }
}
