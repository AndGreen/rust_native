#[cfg_attr(not(any(test, target_os = "ios")), allow(dead_code))]
mod executor;

#[cfg(not(target_os = "ios"))]
mod fallback;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "ios")]
pub use ios::NativeBackend;

#[cfg(not(target_os = "ios"))]
pub use fallback::NativeBackend;
