use std::{time::Duration, u8};

use chrono::Local;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt, label::Label};
use smol::channel::Sender;

use crate::{WINDOW_SIZE, WORKSPACES, WORKSPACES_NOTIFY_TX, read_global, write_global};

pub struct Clock {
    tx: Sender<()>,
}

impl Clock {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = smol::channel::unbounded::<()>();

        cx.spawn(async move |this, cx| {
            loop {
                let should_stop = smol::future::or(
                    async {
                        let _ = rx.recv().await;
                        true
                    },
                    async {
                        cx.background_executor()
                            .timer(Duration::from_secs(20))
                            .await;
                        false
                    },
                )
                .await;

                if should_stop {
                    break;
                }

                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();

        Clock { tx }
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        let _ = self.tx.send_blocking(());
    }
}

impl Render for Clock {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Local::now();
        let hour = now.format("%H").to_string();
        let minute = now.format("%M").to_string();
        let mut text_size = px(8.);
        {
            if let Some(size) = read_global(&WINDOW_SIZE) {
                text_size = (size.width * 0.015).max(text_size);
            }
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .children([hour, minute].map(|content| {
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .paddings(Edges {
                        top: text_size / 8.,
                        left: px(0.),
                        right: px(0.),
                        bottom: text_size / 8.,
                    })
                    .child(
                        Label::new(content)
                            .font_family("Liberation Mono")
                            .w_full()
                            .text_center()
                            .font_bold()
                            .text_size(text_size)
                            .text_color(cx.theme().border),
                    )
            }))
    }
}

pub struct Workspaces {
    tx: Sender<()>,
}

impl Workspaces {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (tx, stop_rx) = smol::channel::unbounded::<()>();
        let (notify_tx, notify_rx) = smol::channel::unbounded::<()>();

        write_global(&WORKSPACES_NOTIFY_TX, Some(notify_tx));

        cx.spawn(async move |this, cx| {
            loop {
                let should_stop = smol::future::or(
                    async {
                        let _ = stop_rx.recv().await;
                        true
                    },
                    async {
                        match notify_rx.recv().await {
                            Ok(_) => false,
                            Err(_) => true,
                        }
                    },
                )
                .await;

                if should_stop {
                    break;
                }

                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();

        Self { tx }
    }
}

impl Drop for Workspaces {
    fn drop(&mut self) {
        write_global(&WORKSPACES_NOTIFY_TX, None);
        let _ = self.tx.send_blocking(());
    }
}

impl Render for Workspaces {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut workspaces = read_global(&WORKSPACES).unwrap_or_default();
        workspaces.sort_by_key(|ele| ele.id);
        let mut text_size = px(8.);
        let mut height = px(20.);
        let mut width = px(20.);
        let len = workspaces.len();
        {
            if let Some(size) = read_global(&WINDOW_SIZE) {
                text_size = (size.width * 0.008).max(text_size);
                height = (size.width * 0.02 + px(4.)) * len;
                width = size.width * 0.02;
            }
        }

        div()
            .w(width)
            .h(height)
            .margins(Edges {
                top: px(4.),
                right: px(0.),
                bottom: px(0.),
                left: px(0.),
            })
            .flex()
            .flex_col()
            .justify_around()
            .items_center()
            .children(workspaces.into_iter().map(|workspace| {
                let id = workspace.id;
                div()
                    .w(width)
                    .h(width)
                    .on_mouse_down(MouseButton::Left, move |_, _, _| {
                        let _ = crate::ipc::set_active_workspace(id);
                    })
                    .child(
                        Label::new(workspace.id.to_string())
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .border_2()
                            .w(width)
                            .h(width)
                            .rounded(width)
                            .font_family("Maple Mono NF CN")
                            .text_size(text_size)
                            .text_center()
                            .when_else(
                                workspace.is_focused,
                                |label| {
                                    label
                                        .bg(cx.theme().colors.red)
                                        .border_color(cx.theme().colors.red_light)
                                },
                                |label| label.bg(rgb(0x363a4f)),
                            ),
                    )
            }))
    }
}

pub struct Cpu;

impl Cpu {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Cpu
    }
}

impl Render for Cpu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut width = px(20.);
        {
            if let Some(size) = read_global(&WINDOW_SIZE) {
                width = size.width * 0.02;
            }
        }
        div()
            .margins(Edges {
                top: px(4.),
                right: px(4.),
                left: px(4.),
                bottom: px(4.),
            })
            .w(width)
            .h(width)
            .child(
                Icon::new(IconName::Cpu)
                    .w(width)
                    .h(width)
                    .text_color(cx.theme().colors.blue),
            )
            .on_mouse_down(MouseButton::Left, |_, _, _| {
                let _ = std::process::Command::new("kitty")
                    .arg("-e")
                    .arg("btop")
                    .spawn();
            })
    }
}

pub struct Battery {
    tx: Sender<()>,
}

impl Battery {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = smol::channel::unbounded::<()>();

        cx.spawn(async move |this, cx| {
            loop {
                let should_stop = smol::future::or(
                    async {
                        let _ = rx.recv().await;
                        true
                    },
                    async {
                        cx.background_executor()
                            .timer(Duration::from_secs(120))
                            .await;
                        false
                    },
                )
                .await;

                if should_stop {
                    break;
                }

                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();

        Battery { tx }
    }
}

impl Drop for Battery {
    fn drop(&mut self) {
        let _ = self.tx.send_blocking(());
    }
}

impl Render for Battery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let (mut icon_name, mut color) = (IconName::BatteryWarning, colors.red);
        let manager = battery::Manager::new();
        if let Ok(manager) = manager
            && let Ok(batteries) = manager.batteries()
            && let Some(battery) = batteries.into_iter().next()
            && let Ok(battery) = battery
        {
            (icon_name, color) = match (battery.state_of_charge().value * 100.) as u8 {
                0..=10 => (IconName::BatteryWarning, colors.red_light),
                11..=33 => (IconName::BatteryLow, colors.red),
                34..=70 => (IconName::BatteryMedium, colors.green),
                71..=u8::MAX => (IconName::BatteryFull, colors.green_light),
            };
        }
        let mut width = px(20.);
        {
            if let Some(size) = read_global(&WINDOW_SIZE) {
                width = size.width * 0.02;
            }
        }
        div()
            .margins(Edges {
                top: px(4.),
                right: px(4.),
                left: px(4.),
                bottom: px(24.),
            })
            .w(width)
            .h(width)
            .child(Icon::new(icon_name).w(width).h(width).text_color(color))
    }
}
