use std::env;

use dev_support::run_worker;
use form_demo::{create_form_demo_app, create_form_demo_native_app};
use mf_runtime::HostSize;

fn main() {
    if env::var_os("MF_DEV_REMOTE_WORKER").is_some() {
        run_worker(create_form_demo_app).expect("remote worker failed");
        return;
    }

    let app = create_form_demo_native_app(HostSize::new(390.0, 844.0));
    app.run();
}
