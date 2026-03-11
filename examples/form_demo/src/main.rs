use form_demo::create_form_demo_app;
use mf_runtime::HostSize;

fn main() {
    let app = create_form_demo_app(HostSize::new(390.0, 844.0));
    app.run();
}
