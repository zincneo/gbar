use super::*;

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
        if let Some(size) = read_global(&WINDOW_SIZE) {
            text_size = (size.width * 0.008).max(text_size);
            height = (size.width * 0.02 + px(4.)) * len;
            width = size.width * 0.02;
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
