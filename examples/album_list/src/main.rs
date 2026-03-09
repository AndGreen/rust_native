use album_list::create_album_list_app;
use mf_runtime::HostSize;

fn main() {
    let app = create_album_list_app(HostSize::new(390.0, 844.0));
    app.repaint();
}
