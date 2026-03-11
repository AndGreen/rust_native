use std::env;

use counter::{create_counter_app, create_counter_native_app};
use dev_support::run_worker;
use mf_runtime::HostSize;

fn main() {
    if env::var_os("MF_DEV_REMOTE_WORKER").is_some() {
        run_worker(create_counter_app).expect("remote worker failed");
        return;
    }

    let app = create_counter_native_app(HostSize::new(390.0, 844.0));
    app.run();
}
