use niri_ipc::Event;
use smol::channel::Receiver;

use gpui::{layer_shell::Anchor, *};
use gpui_platform::application;

use crate::{WINDOW_SIZE, read_global};

struct RootView;

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(white())
    }
}

fn open_window(app: &mut AsyncApp, anchor: Anchor) {
    let mut size = size(px(10.), px(10.));
    if let Some(window_size) = read_global(&WINDOW_SIZE) {
        if anchor.bits() == 0b0111 || anchor.bits() == 0b1011 {
            size.width = window_size.width * 0.02;
            size.height = window_size.height;
        } else {
            size.width = window_size.width;
            size.height = window_size.height * 0.02;
        }
    }
    app.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size,
            })),
            kind: WindowKind::LayerShell(layer_shell::LayerShellOptions {
                namespace: "gbar".to_string(),
                layer: layer_shell::Layer::Top,
                anchor,
                margin: None,
                ..Default::default()
            }),
            ..Default::default()
        },
        |_, app| app.new(|_| RootView),
    )
    .expect("Failed to open window");
}

pub fn task(rx: Receiver<Event>) -> impl FnOnce() -> anyhow::Result<()> + Send {
    move || {
        application().run(move |app| {
            app.spawn(async move |_| {
                while let Ok(event) = rx.recv().await {
                    handle_niri_event(event);
                }
            })
            .detach();
            [0b0111, 0b1011, 0b1101, 0b1110]
                .into_iter()
                .map(|bits| Anchor::from_bits_retain(bits))
                .for_each(|anchor| {
                    app.spawn(async move |app| {
                        open_window(app, anchor);
                    })
                    .detach();
                });
        });
        Ok(())
    }
}

fn handle_niri_event(event: Event) {
    match event {
        _ => (),
    }
}
