use iced::Task;
use iced::futures::StreamExt;
use upower_dbus::{DeviceProxy, UPowerProxy};

use crate::message::Message;

#[derive(Debug, Clone, Copy)]
pub struct BatteryState {
    pub percentage: f64,
    pub state: upower_dbus::BatteryState,
    pub time_to_full: Option<i64>,
    pub time_to_empty: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Battery(DeviceProxy<'static>);

impl Battery {
    pub fn new() -> Task<Option<Self>> {
        Task::future(async {
            let conn = zbus::Connection::system().await.ok()?;
            let upower = UPowerProxy::new(&conn).await.ok()?;
            let battery = upower.get_display_device().await.ok()?;

            Some(Self(battery))
        })
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::advanced::subscription::from_recipe(self.clone())
    }
}

impl iced::advanced::subscription::Recipe for Battery {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> iced::runtime::futures::BoxStream<Self::Output> {
        let battery = self.0;

        iced::futures::stream::unfold((battery, None), |(battery, changes)| async move {
            let mut changes = match changes {
                Some(changes) => changes,
                None => {
                    let percentage = battery.receive_percentage_changed().await;
                    let state = battery.receive_state_changed().await;
                    let time_to_full = battery.receive_time_to_full_changed().await;
                    let time_to_empty = battery.receive_time_to_empty_changed().await;
                    iced::futures::prelude::stream::select_all([
                        percentage.map(|_| ()).boxed(),
                        state.map(|_| ()).boxed(),
                        time_to_full.map(|_| ()).boxed(),
                        time_to_empty.map(|_| ()).boxed(),
                    ])
                }
            };

            changes.next().await;

            let percentage = battery.percentage().await.ok()?;
            let state = battery.state().await.ok()?;
            let time_to_full = battery.time_to_full().await.ok();
            let time_to_empty = battery.time_to_empty().await.ok();
            let state = BatteryState {
                percentage,
                state,
                time_to_full,
                time_to_empty,
            };

            Some((Message::BatteryUpdate(state), (battery, Some(changes))))
        })
        .boxed()
    }
}
