use std::time::Duration;

use chrono::Local;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{ActiveTheme, StyledExt, label::Label};
use smol::channel::Sender;

use crate::{WINDOW_SIZE, read_global};

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
                text_size = (size.width * 0.0125).max(text_size);
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
                            .w_full()
                            .text_center()
                            .font_bold()
                            .text_size(text_size)
                            .text_color(cx.theme().border),
                    )
            }))
    }
}
