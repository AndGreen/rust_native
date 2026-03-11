mod ios_bridge;

use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{create_signal, App, HostSize};
use mf_widgets::prelude::*;

pub fn create_form_demo_app(host_size: HostSize) -> App<NativeBackend> {
    App::new_with_host_size(NativeBackend::default(), host_size, {
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
                    VStack(spacing = 16.0, padding = 24.0, alignment = Alignment::Leading)
                        .background(Color::hex_or_black("#FAF6F1")) {
                        Text("Profile Form").font(Font::bold(28.0)).color(Color::primary())
                        Text("Controlled inputs with Rust-owned focus state")
                            .foreground(Color::secondary())
                        VStack(spacing = 12.0, alignment = Alignment::Leading)
                            .background(Color::hex_or_black("#F6F0EB")) {
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
                    }
                }
            }
        }
    })
}
