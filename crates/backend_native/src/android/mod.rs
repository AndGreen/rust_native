#![cfg_attr(test, allow(dead_code))]

mod adapter;
mod events;
mod jni;

use backend_api::{Backend, BackendError};
use native_schema::{LayoutFrame, Mutation, UiEvent};

use crate::executor::ExecutorState;

use self::adapter::AndroidAdapter;
use self::jni::AndroidJniBridge;

pub struct NativeBackend {
    state: ExecutorState<usize>,
    adapter: AndroidAdapter<AndroidJniBridge>,
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self {
            state: ExecutorState::default(),
            adapter: AndroidAdapter::new(AndroidJniBridge),
        }
    }
}

impl Backend for NativeBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        ensure_ui_thread(&self.adapter)?;
        self.state.apply_mutations(&mut self.adapter, mutations)
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        ensure_ui_thread(&self.adapter)?;
        self.state.apply_layout(frames)
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        ensure_ui_thread(&self.adapter)?;
        self.state.flush(&mut self.adapter)
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        self.state.drain_events(&mut self.adapter)
    }
}

fn ensure_ui_thread(adapter: &AndroidAdapter<AndroidJniBridge>) -> Result<(), BackendError> {
    if adapter.is_ui_thread() {
        Ok(())
    } else {
        Err(BackendError::BatchRejected(
            "android backend requires UI thread".to_string(),
        ))
    }
}
