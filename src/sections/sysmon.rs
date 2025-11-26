use std::{collections::HashSet, sync::LazyLock, time::Duration};

use iced::{Color, widget::row};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::{
    components::{section, text_with_icon},
    message::Message,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct SysInfo {
    cpu_usage: f32,
    cpu_temp: f32,
    ram_usage: f32,
}

#[derive(Debug)]
pub struct Sysmon {
    system: System,
    components: Components,
    info: SysInfo,
}

impl Sysmon {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        let components = Components::new_with_refreshed_list();
        Self {
            system,
            components,
            info: SysInfo::default(),
        }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::SysinfoUpdate(info) => {
                self.system.refresh_cpu_usage();
                self.system.refresh_memory();
                self.components.refresh(true);
                // TODO:
                self.info = *info;
            }
            _ => {}
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let cpu_usage = self.system.global_cpu_usage();
        let ram =
            (self.system.total_memory() - self.system.available_memory()) as f64 / 1_000_000_000.;

        let temperatures = self.components.list();
        static LABELS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
            HashSet::from([
                // AMD Zen CPUs
                // Tctl = Control Temperature
                // Tccd1 = Core Complex Die 1
                // Tccd2 = Core Complex Die 2
                "k10temp Tctl",
                // Intel CPUs
                "coretemp Package id 0",
            ])
        });
        let cpu_temp = temperatures
            .iter()
            .find(|t| LABELS.contains(t.label()))
            .and_then(|t| t.temperature());
        if cpu_temp.is_none() {
            println!("CPU temperature not found.");
        }
        let cpu_temp = cpu_temp.unwrap_or_default();

        section(
            row![
                text_with_icon(
                    "cpu",
                    Some(Color::from_rgb8(0xb4, 0xbe, 0xfe)),
                    format!("{cpu_usage:.1}%")
                ),
                text_with_icon(
                    "temperature",
                    Some(Color::from_rgb8(0xf3, 0x8b, 0xa8)),
                    format!("{cpu_temp:.0}°")
                ),
                text_with_icon(
                    "cpu-2",
                    Some(Color::from_rgb8(0xf5, 0xc2, 0xe7)),
                    format!("{ram:.1} GB")
                ),
            ]
            .spacing(12),
        )
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_secs(5)).map(|_| Message::SysinfoUpdate(todo!()))
    }
}
