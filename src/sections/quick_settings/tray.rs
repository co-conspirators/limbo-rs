//! note: rome wasn't built in a day

use std::rc::Rc;

use iced::widget::Row;

use crate::GlobalState;
use crate::components::system_icon;
use crate::config::Config;
use crate::message::Message;
use crate::tray::TrayItem;

#[derive(Debug)]
pub struct TrayView {
    config: Rc<Config>,
    items: Vec<TrayItem>,
}

impl TrayView {
    pub fn new(global_state: &GlobalState) -> Self {
        Self {
            config: global_state.config.clone(),
            items: global_state.tray_items.clone(),
        }
    }

    pub fn update(&mut self, message: &Message) {
        if let Message::TrayItemsUpdate(items) = message {
            self.items = items.clone();
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let icons = self
            .items
            .iter()
            .filter_map(|item| system_icon(item.item.icon_name.as_ref()?))
            .collect::<Vec<_>>();

        self.config.section(Row::from_vec(icons).spacing(12)).into()
    }
}
