use std::thread;

use hyprland::event_listener::EventListener;
use iced::{Subscription, stream};
use tokio::sync::mpsc;

use crate::Message;

pub fn hyprland_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(100, |mut output| async move {
            let (tx, mut rx) = mpsc::unbounded_channel();

            // Spawn a dedicated thread for the non-Send EventListener
            thread::spawn(move || {
                let mut event_listener = EventListener::new();

                // Set up event handlers
                let sender = tx.clone();
                event_listener.add_workspace_changed_handler(move |workspace| {
                    let _ = sender.send(Message::WorkspaceChanged(workspace.id));
                });

                let sender = tx.clone();
                event_listener.add_workspace_added_handler(move |workspace| {
                    let _ = sender.send(Message::WorkspaceCreated(workspace.id));
                });

                let sender = tx.clone();
                event_listener.add_workspace_deleted_handler(move |workspace| {
                    let _ = sender.send(Message::WorkspaceDestroyed(workspace.id));
                });

                event_listener.start_listener()
            });

            // Forward messages asynchronously
            while let Some(msg) = rx.recv().await {
                let _ = output.try_send(msg);
            }
        })
    })
}
