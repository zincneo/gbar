use super::component::{Bat, Clock, Cpu, Workspaces};
use crate::{WINDOW_SIZE, read_global};
use gpui::{layer_shell::Anchor, *};
use gpui_component::{ActiveTheme, Root};

use std::sync::Arc;
const ICON_BYTES: [&[u8]; 2] = [
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/catppuccin0.png"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/catppuccin1.png"
    )),
];

pub fn open_window(app: &mut AsyncApp) {
    let mut size = size(px(10.), px(10.));
    if let Some(window_size) = read_global(&WINDOW_SIZE) {
        size.width = (window_size.width * 0.06).min(px(48.));
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
            is_movable: false,
            is_resizable: false,
            focus: false,
            kind: WindowKind::LayerShell(layer_shell::LayerShellOptions {
                namespace: "sidebar".to_string(),
                layer: layer_shell::Layer::Top,
                anchor: Anchor::from_bits_retain(0b0111),
                margin: Some((
                    -super::GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE,
                    px(0.),
                    px(0.),
                    -super::GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE,
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

pub struct RootView {
    workspaces: Entity<Workspaces>,
    clock: Entity<Clock>,
    cpu: Entity<Cpu>,
    battery: Entity<Bat>,
    icon_index: usize,
    icons: [Arc<RenderImage>; 2],
}

impl RootView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        RootView {
            workspaces: cx.new(|cx| Workspaces::new(cx)),
            clock: cx.new(|cx| Clock::new(cx)),
            cpu: cx.new(|cx| Cpu::new(cx)),
            battery: cx.new(|cx| Bat::new(cx)),
            icon_index: 1,
            icons: [
                super::decode_png(ICON_BYTES[0]),
                super::decode_png(ICON_BYTES[1]),
            ],
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut width = px(20.);
        {
            if let Some(size) = read_global(&WINDOW_SIZE) {
                width = size.width * 0.02;
            }
        }
        div()
            .size_full()
            .bg(cx.theme().primary_foreground)
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .children([
                div()
                    .w_full()
                    .h_1_3()
                    .flex()
                    .flex_col()
                    .justify_start()
                    .items_center()
                    .child(div().w(width).h(px(8.)))
                    .child(
                        img(self.icons[self.icon_index].clone())
                            .w(width)
                            .h(width)
                            .object_fit(ObjectFit::Contain)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _event, _window, _cx| {
                                    this.icon_index = (this.icon_index + 1) % 2;
                                }),
                            ),
                    )
                    .child(self.workspaces.clone()),
                div().w_full().h_1_3().child(self.clock.clone()),
                div()
                    .w_full()
                    .h_1_3()
                    .flex()
                    .flex_col()
                    .justify_end()
                    .items_center()
                    .child(self.cpu.clone())
                    .child(self.battery.clone()),
            ])
    }
}
