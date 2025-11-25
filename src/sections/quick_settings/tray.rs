//! note: rome wasn't built in a day

use iced::widget::Row;

use crate::components::{section, system_icon};
use crate::tray::TrayItem;

#[derive(Debug)]
pub struct TrayView {
    items: Vec<TrayItem>,
}

#[derive(Debug, Clone)]
pub enum TrayMessage {
    Items(Vec<TrayItem>),
}

impl TrayView {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn update(&mut self, message: TrayMessage) {
        match message {
            TrayMessage::Items(items) => {
                self.items = items;
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, TrayMessage> {
        let icons = self
            .items
            .iter()
            .filter_map(|item| system_icon(item.item.icon_name.as_ref()?))
            .collect::<Vec<_>>();

        section(Row::from_vec(icons).spacing(12)).into()
    }
}
