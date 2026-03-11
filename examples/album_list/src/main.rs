use std::env;

use album_list::{create_album_list_app, create_album_list_native_app};
use dev_support::run_worker;
use mf_runtime::HostSize;

fn main() {
    if env::var_os("MF_DEV_REMOTE_WORKER").is_some() {
        run_worker(create_album_list_app).expect("remote worker failed");
        return;
    }

    let app = create_album_list_native_app(HostSize::new(390.0, 844.0));
    app.repaint();
}
