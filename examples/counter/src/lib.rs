mod ios_bridge;

use std::time::Duration;

use backend_api::Backend;
use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{batch_updates, create_signal, start_interval, App, HostSize};
use mf_widgets::prelude::*;

pub fn create_counter_app<B>(backend: B, host_size: HostSize) -> App<B>
where
    B: Backend + Send + 'static,
{
    App::new_with_host_size(backend, host_size, {
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
                SafeArea {
                    VStack(spacing = 12.0, padding = 16.0, alignment = Alignment::Center).background(Color::hex_or_black("#FAF6F1")) {
                        Text(format!("Count: {}", current))
                            .font(Font::bold(24.0))
                            .color(Color::primary())
                        HStack(spacing = 8.0) {
                            Button("−")
                                .background(Color::hex_or_black("#D14A42"))
                                .foreground(Color::hex_or_black("#F6F0EB"))
                                .corner_radius(12.0)
                                .on_click(move || {
                                    decrement.update(|value| *value -= 1);
                                })
                            Button("+")
                                .background(Color::hex_or_black("#248C61"))
                                .foreground(Color::hex_or_black("#F6F0EB"))
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
        }
         
    })
}

pub fn create_counter_native_app(host_size: HostSize) -> App<NativeBackend> {
    create_counter_app(NativeBackend::default(), host_size)
}
