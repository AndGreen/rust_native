#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unexpected_cfgs)]

mod adapter;
mod events;
mod uikit;

use backend_api::{Backend, BackendError};
use native_schema::{LayoutFrame, Mutation, UiEvent};
use objc::runtime::{Object, BOOL, YES};
use objc::{class, msg_send, sel, sel_impl};

use crate::executor::ExecutorState;

use self::adapter::IosAdapter;

pub struct NativeBackend {
    state: ExecutorState<*mut Object>,
    adapter: IosAdapter,
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self {
            state: ExecutorState::default(),
            adapter: IosAdapter::default(),
        }
    }
}

// Objective-C handles are raw pointers but all UI work is guarded to the main thread.
unsafe impl Send for NativeBackend {}

impl Backend for NativeBackend {
    fn apply_mutations(&mut self, mutations: &[Mutation]) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.apply_mutations(&mut self.adapter, mutations)
    }

    fn apply_layout(&mut self, frames: &[LayoutFrame]) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.apply_layout(frames)
    }

    fn flush(&mut self) -> Result<(), BackendError> {
        ensure_main_thread()?;
        self.state.flush(&mut self.adapter)
    }

    fn drain_events(&mut self) -> Vec<UiEvent> {
        self.state.drain_events(&mut self.adapter)
    }
}

fn ensure_main_thread() -> Result<(), BackendError> {
    let is_main: BOOL = unsafe { msg_send![class!(NSThread), isMainThread] };
    if is_main == YES {
        Ok(())
    } else {
        Err(BackendError::BatchRejected(
            "ios backend requires main thread".to_string(),
        ))
    }
}
