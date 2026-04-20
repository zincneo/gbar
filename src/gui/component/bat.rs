use std::time::Duration;
use std::u8;

use super::*;

pub struct Bat {
    tx: Sender<()>,
}

impl Bat {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let tx = spawn_periodic_refresh(cx, Duration::from_secs(120));
        Bat { tx }
    }
}

impl Drop for Bat {
    fn drop(&mut self) {
        let _ = self.tx.send_blocking(());
    }
}

impl Render for Bat {
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
        let width = icon_width();
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
