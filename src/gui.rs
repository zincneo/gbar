use std::path::PathBuf;

use gpui_component::{ActiveTheme, Root, Theme, ThemeRegistry};
use niri_ipc::Event;
use smol::channel::Receiver;

use gpui::{layer_shell::Anchor, *};
use gpui_platform::application;

use crate::{WINDOW_SIZE, component::Clock, read_global};

const GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE: Pixels = px(14.0);

struct RootView {
    clock: Entity<Clock>,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let clock = cx.new(|cx| Clock::new(cx));
        RootView { clock }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.theme().primary_foreground)
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .child(self.clock.clone())
    }
}

fn open_window(app: &mut AsyncApp) {
    let mut size = size(px(10.), px(10.));
    if let Some(window_size) = read_global(&WINDOW_SIZE) {
        size.width = (window_size.width * 0.05).min(px(42.));
        size.height = window_size.height - px(4.);
    }
    app.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size,
            })),
            window_background: WindowBackgroundAppearance::Transparent,
            window_decorations: None,
            kind: WindowKind::LayerShell(layer_shell::LayerShellOptions {
                namespace: "gbar".to_string(),
                layer: layer_shell::Layer::Top,
                anchor: Anchor::from_bits_retain(0b0111),
                margin: Some((
                    -GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE,
                    px(0.),
                    px(0.),
                    -GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE,
                )),
                ..Default::default()
            }),
            ..Default::default()
        },
        |window, app| {
            let root_view = app.new(|cx| RootView::new(cx));
            let root = app.new(|cx| Root::new(root_view, window, cx));
            root
        },
    )
    .expect("Failed to open window");
}

pub fn task(rx: Receiver<Event>) -> impl FnOnce() -> anyhow::Result<()> + Send {
    move || {
        application().run(move |app| {
            gpui_component::init(app);
            let theme_name = SharedString::from("Catppuccin Custom");
            // Load and watch themes from ./themes directory
            if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), app, move |cx| {
                if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
                    Theme::global_mut(cx).apply_config(&theme);
                }
            }) {
                eprintln!("Failed to watch themes directory: {}", err);
            }
            app.spawn(async move |_| {
                while let Ok(event) = rx.recv().await {
                    handle_niri_event(event);
                }
            })
            .detach();

            app.spawn(async move |app| {
                open_window(app);
            })
            .detach();
        });
        Ok(())
    }
}

fn handle_niri_event(event: Event) {
    match event {
        _ => (),
    }
}
