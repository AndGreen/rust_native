use counter::create_counter_app;
use mf_runtime::HostSize;

fn main() {
    let app = create_counter_app(HostSize::new(390.0, 844.0));
    app.run();
}
