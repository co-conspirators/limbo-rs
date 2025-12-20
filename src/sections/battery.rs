use std::rc::Rc;

use crate::GlobalState;
use crate::battery::BatteryState;
use crate::config::Config;
use crate::message::Message;

#[derive(Debug)]
pub struct BatteryView {
    config: Rc<Config>,
    state: BatteryState,
}

impl BatteryView {
    pub fn new(global_state: &GlobalState) -> Self {
        Self {
            config: global_state.config.clone(),
            state: global_state.battery_state,
        }
    }

    pub fn update(&mut self, message: &Message) {
        if let Message::BatteryUpdate(battery_state) = message {
            self.state = *battery_state;
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let cfg = &self.config.bar.battery;

        let percentage = if self.state.percentage > cfg.full_threshold as f64 {
            100.0
        } else {
            self.state.percentage
        };

        let icon = match self.state.state {
            upower_dbus::BatteryState::Charging => &cfg.charging_icon,
            upower_dbus::BatteryState::PendingCharge | upower_dbus::BatteryState::FullyCharged => {
                cfg.ramp_icons.last().unwrap()
            }
            _ => {
                &cfg.ramp_icons[((cfg.ramp_icons.len() as f64 * percentage).floor() as usize)
                    .min(cfg.ramp_icons.len() - 1)]
            }
        };

        self.config
            .section(
                self.config
                    .text_with_icon(icon, format!("{:.0}%", percentage)),
            )
            .into()
    }
}
