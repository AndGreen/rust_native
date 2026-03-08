use std::time::Duration;

use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{batch_updates, create_signal, start_interval, App};
use mf_widgets::prelude::*;

fn main() {
    // All reactive state is created inside the app initializer.
    let app = App::new(NativeBackend, {
        let (count, set_count) = create_signal(0i32);

        // Auto-increment every second; handle lives as long as the closure does.
        let interval = start_interval(Duration::from_secs(1), {
            let setter = set_count.clone();
            move || setter.update(|c| *c += 1)
        });

        move || {
            let current = count.get();
            let decrement = set_count.clone();
            let increment = set_count.clone();
            // Keep interval alive by capturing it.
            let _keep_alive = &interval;
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
                            batch_updates(|| {
                                increment.update(|value| *value += 1);
                                increment.update(|value| *value += 1);
                            });
                        })
                    }
                }
            }
        }
    });

    // Long-lived run-loop; Ctrl+C to exit.
    app.run();
}
