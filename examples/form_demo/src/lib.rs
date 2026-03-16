mod ios_bridge;

use backend_api::Backend;
use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{create_signal, App, HostSize};
use mf_widgets::prelude::*;

pub fn create_form_demo_app<B>(backend: B, host_size: HostSize) -> App<B>
where
    B: Backend + Send + 'static,
{
    App::new_with_host_size(backend, host_size, {
        let (name, set_name) = create_signal(String::new());
        let (email, set_email) = create_signal(String::new());
        let (focused_field, set_focused_field) = create_signal(String::from("name"));

        move || {
            let current_name = name.get();
            let current_email = email.get();
            let focus = focused_field.get();
            let set_name_value = set_name.clone();
            let set_email_value = set_email.clone();
            let set_focus_name = set_focused_field.clone();
            let set_focus_email = set_focused_field.clone();
            let set_focus_button = set_focused_field.clone();
            let submit_name = current_name.clone();
            let submit_email = current_email.clone();

            ui! {
                SafeArea {
                    VStack(spacing = 16.0, padding = 24.0)
                        .background(Color::hex_or_black("#FAF6F1")) {
                        Text("Profile Form").font(Font::bold(28.0)).color(Color::primary())
                        Text("Controlled inputs with Rust-owned focus state")
                            .foreground(Color::secondary())
                        Container(
                            padding = 16.0,
                            background = Color::hex_or_black("#F6F0EB"),
                            corner_radius = 22.0,
                        )
                            .border(1.0, Color::hex_or_black("#E7DCCF"))
                            .shadow(Color::hex_or_black("#1C130A").with_alpha(0.10), 12.0, 0.0, 6.0) {
                            VStack(spacing = 12.0) {
                                Text("Name").font(Font::semibold(16.0))
                                Input(current_name.clone())
                                    .font(Font::regular(18.0))
                                    .foreground(Color::primary())
                                    .background(Color::hex_or_black("#F3ECE7"))
                                    .corner_radius(12.0)
                                    .focused(focus == "name")
                                    .on_input(move |value| {
                                        set_name_value.set(value);
                                    })
                                    .on_focus_change(move |focused| {
                                        if focused {
                                            set_focus_name.set("name".to_string());
                                        }
                                    })
                                Text("Email").font(Font::semibold(16.0))
                                Input(current_email.clone())
                                    .font(Font::regular(18.0))
                                    .foreground(Color::primary())
                                    .background(Color::hex_or_black("#F3ECE7"))
                                    .corner_radius(12.0)
                                    .focused(focus == "email")
                                    .on_input(move |value| {
                                        set_email_value.set(value);
                                    })
                                    .on_focus_change(move |focused| {
                                        if focused {
                                            set_focus_email.set("email".to_string());
                                        }
                                    })
                            }
                        }
                        HStack(spacing = 12.0) {
                            Button("Focus Email")
                                .background(Color::hex_or_black("#2B6CB0"))
                                .foreground(Color::hex_or_black("#F7FAFC"))
                                .corner_radius(12.0)
                                .on_click(move || {
                                    set_focus_button.set("email".to_string());
                                })
                            Button("Submit")
                                .background(Color::hex_or_black("#248C61"))
                                .foreground(Color::hex_or_black("#F6F0EB"))
                                .corner_radius(12.0)
                                .on_click(move || {
                                    println!(
                                        "submit form_demo name={submit_name:?} email={submit_email:?}"
                                    );
                                })
                        }
                        VStack(spacing = 10.0, alignment = Alignment::Leading) {
                            Text("Container Preview")
                                .font(Font::semibold(16.0))
                                .foreground(Color::secondary())
                            HStack(spacing = 12.0, alignment = Alignment::Center) {
                                Container(
                                    width = 16.0,
                                    height = 16.0,
                                    background = Color::hex_or_black("#248C61"),
                                    full_round = true,
                                )
                                Container(
                                    min_width = 96.0,
                                    padding_insets = EdgeInsets::new(8.0, 12.0, 8.0, 12.0),
                                    background = Color::hex_or_black("#EADFCF"),
                                )
                                    .alignment(Alignment::Center)
                                    .justify_content(JustifyContent::Center)
                                    .corner_radius_per_corner(14.0, 4.0, 14.0, 4.0)
                                    .offset(0.0, -1.0) {
                                    Container(
                                        background = Color::hex_or_black("#ff7d18"),
                                    ) {
                                        Text("Preview")
                                            .font(Font::semibold(14.0))
                                            .foreground(Color::primary())
                                    }
                                }
                                Container(
                                    width = 44.0,
                                    height = 44.0,
                                    full_round = true,
                                )
                                    .border(2.0, Color::hex_or_black("#2B6CB0"))
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn create_form_demo_native_app(host_size: HostSize) -> App<NativeBackend> {
    create_form_demo_app(native_backend(), host_size)
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn native_backend() -> NativeBackend {
    NativeBackend::default()
}

#[cfg(all(not(target_os = "ios"), not(target_os = "android")))]
fn native_backend() -> NativeBackend {
    NativeBackend
}
