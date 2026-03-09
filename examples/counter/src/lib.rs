use std::time::Duration;

use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{batch_updates, create_signal, start_interval, App, HostSize};
use mf_widgets::prelude::*;

pub fn create_counter_app(host_size: HostSize) -> App<NativeBackend> {
    App::new_with_host_size(NativeBackend::default(), host_size, {
        let (count, set_count) = create_signal(0i32);

        let interval = start_interval(Duration::from_secs(1), {
            let setter = set_count.clone();
            move || setter.update(|c| *c += 1)
        });

        move || {
            let current = count.get();
            let decrement = set_count.clone();
            let increment = set_count.clone();
            let _keep_alive = &interval;
            ui! {
                VStack(spacing = 12.0, padding = 16.0) {
                    Text(format!("Count: {}", current))
                        .font(Font::bold(24.0))
                        .color(Color::primary())
                    HStack(spacing = 8.0) {
                        Button("−")
                            .background(Color::new(0.82, 0.29, 0.26))
                            .foreground(Color::new(0.98, 0.96, 0.92))
                            .corner_radius(12.0)
                            .on_click(move || {
                                decrement.update(|value| *value -= 1);
                            })
                        Button("+")
                            .background(Color::new(0.14, 0.55, 0.38))
                            .foreground(Color::new(0.98, 0.96, 0.92))
                            .corner_radius(12.0)
                            .on_click(move || {
                                batch_updates(|| {
                                    increment.update(|value| *value += 1);
                                    increment.update(|value| *value += 1);
                                });
                            });
                    }
                }
            }
        }
    })
}
