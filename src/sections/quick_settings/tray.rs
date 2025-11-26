//! note: rome wasn't built in a day

use iced::widget::Row;

use crate::{
    components::{section, system_icon},
    message::Message,
    tray::TrayItem,
};

#[derive(Debug)]
pub struct TrayView {
    items: Vec<TrayItem>,
}

impl TrayView {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::TrayItems(items) => {
                self.items = items.clone();
            }
            _ => {}
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let icons = self
            .items
            .iter()
            .filter_map(|item| system_icon(item.item.icon_name.as_ref()?))
            .collect::<Vec<_>>();

        section(Row::from_vec(icons).spacing(12)).into()
    }
}
