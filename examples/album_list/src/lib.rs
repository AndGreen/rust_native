mod ios_bridge;

use backend_api::Backend;
use backend_native::NativeBackend;
use mf_macros::ui;
use mf_runtime::{App, HostSize};
use mf_widgets::prelude::*;

#[derive(Clone)]
struct Album {
    title: &'static str,
    artist: &'static str,
    cover: &'static str,
}

pub fn create_album_list_app<B>(backend: B, host_size: HostSize) -> App<B>
where
    B: Backend + Send + 'static,
{
    let albums = vec![
        Album {
            title: "Explorations",
            artist: "Nova Collective",
            cover: "explorations.jpg",
        },
        Album {
            title: "Analog Dreams",
            artist: "Chromatic",
            cover: "analog_dreams.jpg",
        },
        Album {
            title: "Signal Flow",
            artist: "Greyline",
            cover: "signal_flow.jpg",
        },
    ];

    App::new_with_host_size(backend, host_size, {
        let albums = albums.clone();
        move || {
            let data = albums.clone();
            ui! {
                SafeArea {
                    VStack(spacing = 16.0, padding = 24.0,) {
                        Text("Albums").font(Font::bold(32.0))
                        List(data.into_iter(), |album: Album| {
                            let title = album.title.to_string();
                            let artist = album.artist.to_string();
                            let cover = album.cover.to_string();
                            let like_title = title.clone();
                            let like_artist = artist.clone();
                            ui! {
                                HStack(spacing = 12.0, padding = 8.0) {
                                    Image(cover.clone()).size(60.0, 60.0).corner_radius(8.0)
                                    VStack(alignment = Alignment::Leading) {
                                        Text(title.clone()).font(Font::semibold(18.0))
                                        Text(artist.clone()).foreground(Color::secondary())
                                    }
                                    Button("Like").on_click(move || {
                                        println!("❤  Liked {} by {}", like_title, like_artist);
                                    })
                                }
                            }
                        })
                    }
                }
            }
        }
    })
}

pub fn create_album_list_native_app(host_size: HostSize) -> App<NativeBackend> {
    create_album_list_app(NativeBackend::default(), host_size)
}
