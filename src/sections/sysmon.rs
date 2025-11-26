use std::{collections::HashSet, sync::LazyLock, time::Duration};

use iced::{Color, futures::StreamExt, widget::row};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::{
    GlobalState,
    components::{section, text_with_icon},
    message::Message,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct SysInfo {
    cpu_usage: f32,
    cpu_temp: f32,
    ram: f32,
}

#[derive(Debug)]
pub struct Sysmon {
    info: SysInfo,
}

impl Sysmon {
    pub fn new(global_state: &GlobalState) -> Self {
        Self {
            info: global_state.sysinfo,
        }
    }

    pub fn update(&mut self, message: &Message) {
        if let Message::SysinfoUpdate(info) = message {
            self.info = *info;
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        section(
            row![
                text_with_icon(
                    "cpu",
                    Some(Color::from_rgb8(0xb4, 0xbe, 0xfe)),
                    format!("{:.1}%", self.info.cpu_usage)
                ),
                text_with_icon(
                    "temperature",
                    Some(Color::from_rgb8(0xf3, 0x8b, 0xa8)),
                    format!("{:.0}°", self.info.cpu_temp)
                ),
                text_with_icon(
                    "cpu-2",
                    Some(Color::from_rgb8(0xf5, 0xc2, 0xe7)),
                    format!("{:.1} GB", self.info.ram)
                ),
            ]
            .spacing(12),
        )
        .into()
    }

    pub fn subscription() -> iced::Subscription<Message> {
        let init = iced::futures::stream::once(async {
            // run these inside the stream with `once` to avoid initializing on each call to `subscription`
            (
                System::new_with_specifics(
                    RefreshKind::nothing()
                        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                        .with_memory(MemoryRefreshKind::nothing().with_ram()),
                ),
                Components::new_with_refreshed_list(),
                true,
            )
        });

        let stream = init.flat_map(|init| {
            iced::futures::stream::unfold(init, |(mut system, mut components, first)| async move {
                // during the first iteration, update immediately
                if !first {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }

                system.refresh_cpu_usage();
                system.refresh_memory();
                components.refresh(true);

                let cpu_usage = system.global_cpu_usage();
                let ram = ((system.total_memory() - system.available_memory()) as f64
                    / 1_000_000_000.) as f32;

                let temperatures = components.list();
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

                let info = SysInfo {
                    cpu_usage,
                    cpu_temp,
                    ram,
                };

                Some((Message::SysinfoUpdate(info), (system, components, false)))
            })
        });

        #[derive(Hash)]
        struct SysmonSubscription;

        iced::Subscription::run_with_id(SysmonSubscription, stream)
    }
}
