use std::rc::Rc;

use iced::alignment::Vertical;
use iced::widget::Row;

use crate::GlobalState;
use crate::config::Config;
use crate::message::Message;

mod tray;

use tray::TrayView;

pub struct QuickSettings {
    config: Rc<Config>,
    tray_view: TrayView,
}

impl QuickSettings {
    pub fn new(global_state: &GlobalState) -> Self {
        Self {
            config: global_state.config.clone(),
            tray_view: TrayView::new(global_state),
        }
    }

    pub fn update(&mut self, message: &Message) {
        self.tray_view.update(message);
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let todo_icon = || crate::components::icon("nix-snowflake-white", None).into();

        self.config
            .section(
                Row::from_iter(
                    self.config
                        .bar
                        .quick_settings
                        .segments
                        .iter()
                        .map(|segment| match segment {
                            crate::config::types::QuickSettingSegment::Tray => {
                                self.tray_view.view()
                            }
                            crate::config::types::QuickSettingSegment::NightLight => todo_icon(),
                            crate::config::types::QuickSettingSegment::Brightness => todo_icon(),
                            crate::config::types::QuickSettingSegment::Caffeine => todo_icon(),
                            crate::config::types::QuickSettingSegment::Dnd => todo_icon(),
                            crate::config::types::QuickSettingSegment::Mic => todo_icon(),
                            crate::config::types::QuickSettingSegment::Notifs => todo_icon(),
                            crate::config::types::QuickSettingSegment::Volume => todo_icon(),
                            crate::config::types::QuickSettingSegment::Network => todo_icon(),
                            crate::config::types::QuickSettingSegment::Battery => todo_icon(),
                            crate::config::types::QuickSettingSegment::Toggle => todo_icon(),
                        }),
                )
                .align_y(Vertical::Center)
                .spacing(12),
            )
            .into()
    }
}
