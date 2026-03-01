mod fallback;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "ios")]
pub use ios::NativeBackend;

#[cfg(not(target_os = "ios"))]
pub use fallback::NativeBackend;
