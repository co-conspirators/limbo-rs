use std::sync::Arc;

use system_tray::client::Client;
use system_tray::item::StatusNotifierItem;
use system_tray::menu::TrayMenu;
use tokio::sync::{Mutex, watch};

#[derive(Debug, Clone)]
pub struct TrayItem {
    pub item: StatusNotifierItem,
    pub menu: Option<TrayMenu>,
}

impl From<&(StatusNotifierItem, Option<TrayMenu>)> for TrayItem {
    fn from(item: &(StatusNotifierItem, Option<TrayMenu>)) -> Self {
        Self {
            item: item.0.clone(),
            menu: item.1.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Tray {
    rx: Arc<Mutex<watch::Receiver<Vec<TrayItem>>>>,
}

impl Tray {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(vec![]);

        tokio::spawn(async move {
            let client = Client::new().await.expect("failed to connect to tray");
            let mut tray_rx = client.subscribe();
            loop {
                let items = client
                    .items()
                    .lock()
                    .expect("mutex should not be poisoned")
                    .values()
                    .map(|item| item.into())
                    .collect();

                if let Err(e) = tx.send(items) {
                    eprintln!("Failed to send tray items: {}", e);
                    break;
                }
                if let Err(e) = tray_rx.recv().await {
                    eprintln!("Failed to receive tray items: {}", e);
                    break;
                }
            }
        });

        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn subscribe(&self) -> iced::Subscription<Vec<TrayItem>> {
        iced::Subscription::run_with_id(
            "tray".to_string(),
            iced::futures::stream::unfold(self.rx.clone(), |rx| async move {
                let value = {
                    let mut rx = rx.lock().await;
                    if rx.changed().await.is_ok() {
                        Some(rx.borrow().clone())
                    } else {
                        None
                    }
                };
                value.map(|v| (v, rx))
            }),
        )
    }
}
