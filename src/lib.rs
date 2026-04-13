use std::sync::{LazyLock, RwLock};

use gpui::{Pixels, Size};

pub mod gui;
pub mod ipc;

pub static WINDOW_SIZE: LazyLock<RwLock<Option<Size<Pixels>>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn read_global<T: Clone + PartialEq>(variable: &LazyLock<RwLock<Option<T>>>) -> Option<T> {
    if let Ok(read_guard) = variable.read() {
        (*read_guard).clone()
    } else {
        None
    }
}

pub fn write_global<T: Clone + PartialEq>(
    variable: &LazyLock<RwLock<Option<T>>>,
    value: Option<T>,
) {
    if let Ok(mut write_guard) = variable.write() {
        *write_guard = value;
    }
}
