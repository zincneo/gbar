use gpui::{px, size};
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Event, Request, Response};
use smol::channel::Sender;

use crate::{WINDOW_SIZE, write_global};

pub fn task(tx: Sender<Event>) -> impl FnOnce() -> anyhow::Result<()> + Send {
    move || {
        let mut socket = Socket::connect()?;
        let reply = socket.send(Request::EventStream)?;
        if let Ok(_) = reply {
            let mut read_event = socket.read_events();
            while let Ok(event) = read_event() {
                let _ = tx.send_blocking(event);
            }
        } else {
            return anyhow::Result::Err(anyhow::Error::msg("Ipc closed"));
        }
        Ok(())
    }
}

pub fn set_window_size() -> anyhow::Result<()> {
    let mut socket = Socket::connect()?;
    let reply = socket.send(Request::Outputs)?;
    if let Ok(Response::Outputs(outputs)) = reply {
        let _ = outputs
            .iter()
            .next()
            .and_then(|(_, output)| output.logical)
            .is_some_and(|logical_output| {
                let size = size(
                    px(logical_output.width as f32),
                    px(logical_output.height as f32),
                );
                write_global(&WINDOW_SIZE, Some(size));
                true
            });
    }
    Ok(())
}

pub fn set_active_workspace(id: u64) -> anyhow::Result<()> {
    let mut socket = Socket::connect()?;
    let request = Request::Action(Action::FocusWorkspace {
        reference: niri_ipc::WorkspaceReferenceArg::Id(id),
    });
    let _ = socket.send(request)?;
    Ok(())
}
