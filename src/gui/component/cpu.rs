use super::*;

pub struct Cpu;

impl Cpu {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Cpu
    }
}

impl Render for Cpu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = icon_width();
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
