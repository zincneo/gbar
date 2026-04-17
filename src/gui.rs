use std::sync::Arc;

use gpui_component::{Theme, ThemeRegistry};

use niri_ipc::Event;
use smol::channel::Receiver;

use gpui::*;
use gpui_component_assets::Assets;
use gpui_platform::application;
use image::Frame;

use crate::{WORKSPACES, notify_workspaces_changed, read_global, write_global};
pub const GPUI_COMPONENT_LINUX_ROOT_WINDOW_SHADOW_SIZE: Pixels = px(14.0);

pub mod component;
mod sidebar;

pub fn decode_png(bytes: &[u8]) -> Arc<RenderImage> {
    let mut rgba = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("Failed to decode embedded PNG")
        .into_rgba8();
    // RGBA → BGRA
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![Frame::new(rgba)]))
}

pub fn task(rx: Receiver<Event>) -> impl FnOnce() -> anyhow::Result<()> + Send {
    move || {
        application().with_assets(Assets).run(move |app| {
            gpui_component::init(app);
            let theme_name = SharedString::from("Catppuccin Custom");
            let theme_dir = dirs::home_dir()
                .expect("Failed to get home directory")
                .join(".config/gbar/themes");
            // Load and watch themes from ~/.config/gbar/themes directory
            if let Err(err) = ThemeRegistry::watch_dir(theme_dir, app, move |cx| {
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
                sidebar::open_window(app);
            })
            .detach();
        });
        Ok(())
    }
}

fn handle_niri_event(event: Event) {
    match event {
        Event::WorkspacesChanged { workspaces } => {
            write_global(&WORKSPACES, Some(workspaces));
            notify_workspaces_changed();
        }
        Event::WorkspaceActivated { id, focused } if focused == true => {
            let workspaces = read_global(&WORKSPACES);
            if let Some(mut workspaces) = workspaces {
                workspaces.sort_by_key(|workspace| workspace.id);
                workspaces.iter_mut().for_each(|workspace| {
                    if workspace.id == id {
                        workspace.is_focused = true;
                    } else {
                        workspace.is_focused = false;
                    }
                });
                write_global(&WORKSPACES, Some(workspaces));
            }
            notify_workspaces_changed();
        }
        _ => (),
    }
}
