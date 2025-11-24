use std::{collections::HashSet, sync::LazyLock, time::Duration};

use iced::{
    Color,
    widget::{row, text},
};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::components::{icon, section};

#[derive(Debug)]
pub struct Sysmon {
    system: System,
    components: Components,
}

#[derive(Debug, Clone)]
pub enum SysmonMessage {
    Tick,
}

impl Sysmon {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        let components = Components::new_with_refreshed_list();
        Self { system, components }
    }

    pub fn update(&mut self, message: SysmonMessage) {
        match message {
            SysmonMessage::Tick => {
                self.system.refresh_cpu_usage();
                self.system.refresh_memory();
                self.components.refresh(true);
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, SysmonMessage> {
        let cpu_usage = self.system.global_cpu_usage();
        let ram =
            (self.system.total_memory() - self.system.available_memory()) as f64 / 1_000_000_000.0;

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
                row![
                    icon("cpu", Some(Color::from_rgb8(0xb4, 0xbe, 0xfe))),
                    text(format!("{cpu_usage:.1}%"))
                ]
                .spacing(6),
                row![
                    icon("temperature", Some(Color::from_rgb8(0xf3, 0x8b, 0xa8))),
                    text(format!("{cpu_temp:.0} °C"))
                ]
                .spacing(6),
                row![
                    icon("cpu-2", Some(Color::from_rgb8(0xf5, 0xc2, 0xe7))),
                    text(format!("{ram:.1} GB"))
                ]
                .spacing(6),
            ]
            .spacing(12),
        )
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<SysmonMessage> {
        iced::time::every(Duration::from_secs(5)).map(|_| SysmonMessage::Tick)
    }
}
