mod bat;
mod clock;
mod cpu;
mod workspaces;

pub use bat::Bat;
pub use clock::Clock;
pub use cpu::Cpu;
pub use workspaces::Workspaces;

use std::time::Duration;

pub(crate) use gpui::{prelude::FluentBuilder, *};
pub(crate) use gpui_component::{ActiveTheme, Icon, IconName, StyledExt, label::Label};
pub(crate) use smol::channel::Sender;

pub(crate) use crate::{WINDOW_SIZE, WORKSPACES, WORKSPACES_NOTIFY_TX, read_global, write_global};

/// 从全局 WINDOW_SIZE 读取 icon 标准宽度 (width * 0.02)，默认 px(20.)
pub(crate) fn icon_width() -> Pixels {
    read_global(&WINDOW_SIZE)
        .map(|s| s.width * 0.02)
        .unwrap_or(px(20.))
}

/// 生成定时刷新组件的后台任务，返回用于停止的 Sender。
/// `interval` 为刷新间隔。
pub(crate) fn spawn_periodic_refresh<T: 'static>(
    cx: &mut Context<T>,
    interval: Duration,
) -> Sender<()> {
    let (tx, rx) = smol::channel::unbounded::<()>();

    cx.spawn(async move |this, cx| {
        loop {
            let should_stop = smol::future::or(
                async {
                    let _ = rx.recv().await;
                    true
                },
                async {
                    cx.background_executor().timer(interval).await;
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

    tx
}
