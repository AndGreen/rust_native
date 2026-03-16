#[cfg_attr(not(any(test, target_os = "ios")), allow(dead_code))]
mod executor;
#[cfg_attr(
    not(any(test, target_os = "ios", target_os = "android")),
    allow(dead_code)
)]
mod shared;

#[cfg(any(test, target_os = "android"))]
mod android;

#[cfg(all(not(target_os = "ios"), not(target_os = "android")))]
mod fallback;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "android")]
pub use android::NativeBackend;

#[cfg(target_os = "ios")]
pub use ios::NativeBackend;

#[cfg(all(not(target_os = "ios"), not(target_os = "android")))]
pub use fallback::NativeBackend;
