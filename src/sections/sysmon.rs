use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;

use iced::futures::StreamExt;
use iced::widget::Row;
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::GlobalState;
use crate::config::Config;
use crate::config::types::SysMonSegment;
use crate::message::Message;

#[derive(Debug, Default, Clone, Copy)]
pub struct SysInfo {
    cpu_usage: f32,
    cpu_temp: f32,
    ram: f32,
}

#[derive(Debug)]
pub struct Sysmon {
    config: Rc<Config>,
    info: SysInfo,
}

impl Sysmon {
    pub fn new(global_state: &GlobalState) -> Self {
        Self {
            config: global_state.config.clone(),
            info: global_state.sysinfo,
        }
    }

    pub fn update(&mut self, message: &Message) {
        if let Message::SysinfoUpdate(info) = message {
            self.info = *info;
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let cfg = &self.config.bar.sysmon;

        let segments = cfg.segments.iter().map(|segment| match segment {
            SysMonSegment::Cpu => self.config.text_with_icon(
                &self.config.bar.sysmon.cpu.icon,
                format!("{:.*}%", cfg.precision, self.info.cpu_usage),
            ),
            SysMonSegment::Temp => self.config.text_with_icon(
                &self.config.bar.sysmon.temp.icon,
                format!("{:.*}°", cfg.precision, self.info.cpu_temp),
            ),
            SysMonSegment::Ram => self.config.text_with_icon(
                &self.config.bar.sysmon.ram.icon,
                format!("{:.*} GB", cfg.precision, self.info.ram),
            ),
        });

        self.config
            .section(Row::from_iter(segments).spacing(12))
            .into()
    }

    pub fn subscription(config: &Config) -> iced::Subscription<Message> {
        iced::advanced::subscription::from_recipe(SysmonSubscription {
            probe_interval_ms: config.bar.sysmon.probe_interval_ms,
        })
    }
}

#[derive(Hash)]
struct SysmonSubscription {
    probe_interval_ms: u64,
}

impl iced::advanced::subscription::Recipe for SysmonSubscription {
    type Output = Message;

    fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
        <Self as std::hash::Hash>::hash(self, state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced::advanced::subscription::EventStream,
    ) -> iced::runtime::futures::BoxStream<Self::Output> {
        let prove_interval_ms = self.probe_interval_ms;

        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        let components = Components::new_with_refreshed_list();

        let stream = iced::futures::stream::unfold(
            (system, components, true),
            move |(mut system, mut components, first)| async move {
                // during the first iteration, update immediately
                if !first {
                    tokio::time::sleep(Duration::from_millis(prove_interval_ms)).await;
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
            },
        );
        stream.boxed()
    }
}
