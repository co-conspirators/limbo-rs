use std::rc::Rc;

use iced::alignment::Vertical;
use iced::widget::Row;

use crate::GlobalState;
use crate::config::Config;
use crate::config::types::QuickSettingSegment;
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

    pub fn view(&self) -> Option<iced::Element<'_, Message>> {
        let todo_icon = || Some(crate::components::icon("nix-snowflake-white", None).into());

        Some(
            self.config
                .section(
                    Row::from_iter(self.config.bar.quick_settings.segments.iter().filter_map(
                        |segment| match segment {
                            QuickSettingSegment::Tray => self.tray_view.view(),
                            QuickSettingSegment::NightLight => todo_icon(),
                            QuickSettingSegment::Brightness => todo_icon(),
                            QuickSettingSegment::Caffeine => todo_icon(),
                            QuickSettingSegment::Dnd => todo_icon(),
                            QuickSettingSegment::Mic => todo_icon(),
                            QuickSettingSegment::Notifs => todo_icon(),
                            QuickSettingSegment::Volume => todo_icon(),
                            QuickSettingSegment::Network => todo_icon(),
                            QuickSettingSegment::Battery => todo_icon(),
                            QuickSettingSegment::Toggle => todo_icon(),
                        },
                    ))
                    .align_y(Vertical::Center)
                    .spacing(12),
                )
                .into(),
        )
    }
}
