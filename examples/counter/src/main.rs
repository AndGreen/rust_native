use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{use_signal, App};
use mf_widgets::prelude::*;

fn main() {
    let (count, set_count) = use_signal(0i32);
    let app = App::new(NativeBackend::default(), {
        let count_signal = count.clone();
        let setter = set_count.clone();
        move || {
            let current = count_signal.get();
            let decrement = setter.clone();
            let increment = setter.clone();
            ui! {
                VStack(spacing = 12.0, padding = 16.0) {
                    Text(format!("Count: {}", current))
                        .font(Font::bold(24.0))
                        .color(Color::primary())
                    HStack(spacing = 8.0) {
                        Button("−").on_click(move || {
                            decrement.update(|value| *value -= 1);
                        })
                        Button("+").on_click(move || {
                            increment.update(|value| *value += 1);
                        })
                    }
                }
            }
        }
    });
    let _watch = app.watch_signal(&count);
    app.repaint();
}
