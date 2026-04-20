use std::time::Duration;

use chrono::Local;

use super::*;

pub struct Clock {
    tx: Sender<()>,
}

impl Clock {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let tx = spawn_periodic_refresh(cx, Duration::from_secs(20));
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
        if let Some(size) = read_global(&WINDOW_SIZE) {
            text_size = (size.width * 0.015).max(text_size);
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
