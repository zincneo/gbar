use gbar::{gui, ipc};
use smol::channel::unbounded;
use std::sync::mpsc::channel;
use std::thread;

fn main() -> anyhow::Result<()> {
    crate::ipc::set_window_size()?;
    let (tx, rx) = channel::<()>();
    let (ipc_tx, ipc_rx) = unbounded::<niri_ipc::Event>();
    let tasks: [(Box<dyn FnOnce() -> anyhow::Result<()> + Send>, _); 2] = [
        (Box::new(ipc::task(ipc_tx)), tx.clone()),
        (Box::new(gui::task(ipc_rx)), tx.clone()),
    ];
    tasks.into_iter().for_each(|(task, tx)| {
        thread::spawn(move || {
            if let anyhow::Result::Err(e) = task() {
                println!("{:#?}", e);
            }
            let _ = tx.send(());
        });
    });
    let _ = rx.recv();
    Ok(())
}
