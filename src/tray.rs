use std::sync::Arc;

use iced::Task;
use iced::futures::StreamExt;
use system_tray::client::Client;
use system_tray::item::StatusNotifierItem;
use system_tray::menu::TrayMenu;

use crate::message::Message;

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

#[derive(Debug, Clone)]
pub struct Tray(Arc<Client>);

impl Tray {
    pub fn new() -> Task<Option<Self>> {
        Task::future(async {
            let client = Client::new().await;
            match client {
                Ok(client) => Some(Self(Arc::new(client))),
                Err(e) => {
                    eprintln!("failed to connect to tray: {e}");
                    None
                }
            }
        })
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::advanced::subscription::from_recipe(self.clone())
    }
}

impl iced::advanced::subscription::Recipe for Tray {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> iced::runtime::futures::BoxStream<Self::Output> {
        let tray_rx = self.0.subscribe();

        let stream = iced::futures::stream::unfold(
            (self.0, tray_rx, true),
            |(client, mut tray_rx, first)| async move {
                // during the first iteration, update immediately
                if !first && let Err(e) = tray_rx.recv().await {
                    eprintln!("Failed to receive tray items: {}", e);
                    return None;
                }

                let items = client
                    .items()
                    .lock()
                    .expect("mutex should not be poisoned")
                    .values()
                    .map(TrayItem::from)
                    .collect();

                Some((
                    Message::TrayItemsUpdate(Arc::new(items)),
                    (client, tray_rx, false),
                ))
            },
        );

        stream.boxed()
    }
}
