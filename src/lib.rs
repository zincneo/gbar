use std::sync::{LazyLock, RwLock};

use gpui::{Pixels, Size};
use niri_ipc::Workspace;
use smol::channel::Sender;

pub mod component;
pub mod gui;
pub mod ipc;

pub static WINDOW_SIZE: LazyLock<RwLock<Option<Size<Pixels>>>> =
    LazyLock::new(|| RwLock::new(None));

pub static WORKSPACES: LazyLock<RwLock<Option<Vec<Workspace>>>> =
    LazyLock::new(|| RwLock::new(None));

pub(crate) static WORKSPACES_NOTIFY_TX: LazyLock<RwLock<Option<Sender<()>>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn read_global<T: Clone>(variable: &LazyLock<RwLock<Option<T>>>) -> Option<T> {
    if let Ok(read_guard) = variable.read() {
        (*read_guard).clone()
    } else {
        None
    }
}

pub fn write_global<T>(variable: &LazyLock<RwLock<Option<T>>>, value: Option<T>) {
    if let Ok(mut write_guard) = variable.write() {
        *write_guard = value;
    }
}

pub fn notify_workspaces_changed() {
    if let Some(sender) = read_global(&WORKSPACES_NOTIFY_TX) {
        let _ = sender.send_blocking(());
    }
}
