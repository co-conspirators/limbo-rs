use std::collections::HashMap;

use iced::{
    Alignment, Color, Element, Length, Settings, Task, Theme,
    daemon::{Appearance, DefaultStyle},
    event::wayland::{Event as WaylandEvent, LayerEvent, OutputEvent},
    platform_specific::shell::commands::layer_surface::{destroy_layer_surface, get_layer_surface},
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    theme::Palette,
    widget::{container, row},
    window,
};
use sctk::{
    output::OutputInfo,
    reexports::client::protocol::wl_output::WlOutput,
    shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer},
};

use crate::{
    components::{icon, section, side},
    desktop_environment::{Desktop, DesktopEvent, WorkspaceInfo, WorkspaceInfos},
    sections::{
        clock::{Clock, ClockMessage},
        sysmon::{Sysmon, SysmonMessage},
        workspaces::{Workspaces, WorkspacesMessage},
    },
};

mod components;
mod desktop_environment;
mod icons;
mod sections;

#[tokio::main]
pub async fn main() -> iced::Result {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    iced::daemon("limbo", Limbo::update, Limbo::view)
        .settings(Settings {
            id: Some("limbo".to_string()),
            fonts: Vec::new(),
            default_font: iced::Font {
                family: iced::font::Family::Name("DejaVu Sans Mono"),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            },
            default_text_size: iced::Pixels(14.0),
            antialiasing: true,
            exit_on_close_request: false,
            is_daemon: true,
        })
        .subscription(Limbo::subscription)
        .theme(Limbo::theme)
        .style(Limbo::style)
        .run_with(Limbo::new)
}

struct Bar {
    /// window id of the bar's layer surface.
    id: window::Id,
    wl_output: WlOutput,

    workspaces: Option<Workspaces>,
}

struct Limbo {
    desktop: Option<Desktop>,
    workspace_infos: WorkspaceInfos,

    output_infos: HashMap<WlOutput, OutputInfo>,
    bars: Vec<Bar>,

    clock: Clock,
    sysmon: Sysmon,
}

#[derive(Debug, Clone)]
pub enum Message {
    DesktopEvent(DesktopEvent),
    WaylandEvent(WaylandEvent),
    Clock(ClockMessage),
    Workspaces(window::Id, WorkspacesMessage),
    Sysmon(SysmonMessage),
}

impl Limbo {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                desktop: Desktop::new(),
                workspace_infos: Vec::new(),
                output_infos: HashMap::new(),
                bars: Vec::new(),
                clock: Clock::new(),
                sysmon: Sysmon::new(),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let mut subscriptions = vec![
            iced::event::listen_with(|evt, _, _| {
                if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) =
                    evt
                {
                    Some(Message::WaylandEvent(evt))
                } else {
                    None
                }
            }),
            self.clock.subscription().map(Message::Clock),
            self.sysmon.subscription().map(Message::Sysmon),
        ];
        if let Some(desktop) = &self.desktop {
            subscriptions.push(desktop.subscription().map(Message::DesktopEvent));
        }
        for bar in self.bars.iter() {
            if let Some(workspaces) = &bar.workspaces
                && let Some(workspace_subscription) = workspaces.subscription()
            {
                subscriptions.push(
                    workspace_subscription
                        .with(bar.id)
                        .map(|(id, msg)| Message::Workspaces(id, msg)),
                );
            }
        }
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesktopEvent(DesktopEvent::WorkspacesChanged(workspace_infos)) => {
                self.workspace_infos = workspace_infos.clone();

                let workspace_infoss = self
                    .bars
                    .iter()
                    .map(|bar| self.workspace_infos_for_output(&bar.wl_output).collect())
                    .collect::<Vec<_>>();
                for (bar, workspace_infos) in self.bars.iter_mut().zip(workspace_infoss) {
                    if let Some(workspaces) = &mut bar.workspaces
                        && let Some(desktop) = self.desktop.as_mut()
                    {
                        workspaces.update(
                            WorkspacesMessage::WorkspacesChanged(workspace_infos),
                            desktop,
                        );
                    }
                }
                Task::none()
            }
            Message::WaylandEvent(evt) => match evt {
                WaylandEvent::Output(OutputEvent::Created(output_info), wl_output) => {
                    if let Some(output_info) = output_info {
                        self.output_infos.insert(wl_output.clone(), output_info);
                    }

                    self.spawn_bar(wl_output)
                }
                WaylandEvent::Output(OutputEvent::InfoUpdate(output_info), wl_output) => {
                    self.output_infos.insert(wl_output, output_info);
                    Task::none()
                }
                WaylandEvent::Output(OutputEvent::Removed, wl_output) => {
                    self.output_infos.remove(&wl_output);

                    let removed_bars = self.bars.extract_if(.., |bar| bar.wl_output == wl_output);
                    Task::batch(removed_bars.map(|bar| destroy_layer_surface(bar.id)))
                }
                WaylandEvent::Layer(LayerEvent::Done, _wl_surface, id) => {
                    self.bars.retain(|bar| bar.id != id);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
            Message::Workspaces(bar_id, message) => {
                if let Some(bar) = self.bars.iter_mut().find(|b| b.id == bar_id)
                    && let Some(workspaces) = &mut bar.workspaces
                    && let Some(desktop) = self.desktop.as_mut()
                {
                    workspaces.update(message, desktop);
                }
                Task::none()
            }
            Message::Sysmon(msg) => {
                self.sysmon.update(msg);
                Task::none()
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(bar) = self.bars.iter().find(|b| b.id == window_id) {
            self.view_bar(bar)
        } else {
            unreachable!("All windows are bars");
        }
    }

    fn view_bar<'a>(&'a self, bar: &'a Bar) -> Element<'a, Message> {
        let transparent = if let Some(bar) = self.bars.iter().find(|b| b.id == bar.id) {
            self.workspace_infos_for_output(&bar.wl_output)
                .find(|w| w.is_active)
                .is_some_and(|w| w.transparent_bar)
        } else {
            false
        };

        let mut center = Vec::new();
        if let Some(workspaces) = &bar.workspaces {
            center.push(
                workspaces
                    .view()
                    .map(|msg| Message::Workspaces(bar.id, msg)),
            )
        }

        container(
            row![
                // Left
                side(
                    Alignment::Start,
                    row![
                        section(icon("nix-snowflake-white", None)),
                        section(icon("nix-snowflake-white", None)),
                        section(icon("nix-snowflake-white", None)),
                        section(icon("nix-snowflake-white", None))
                    ]
                    .spacing(12)
                ),
                // Center
                side(Alignment::Center, row(center).spacing(12)),
                // Right
                side(
                    Alignment::End,
                    row![
                        self.sysmon.view().map(Message::Sysmon),
                        self.clock.view().map(Message::Clock)
                    ]
                    .spacing(12)
                ),
            ]
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(move |theme| {
            if transparent {
                iced::widget::container::transparent(theme)
            } else {
                iced::widget::container::background(theme.palette().background)
            }
        })
        .into()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::custom(
            "internal".to_string(),
            Palette {
                text: Color::WHITE,
                background: Color::from_rgb(
                    0x1e as f32 / 255.0,
                    0x1e as f32 / 255.0,
                    0x2e as f32 / 255.0,
                ),
                // Unused
                primary: Color::BLACK,
                success: Color::BLACK,
                danger: Color::BLACK,
            },
        )
    }

    fn style(&self, theme: &Theme) -> Appearance {
        Appearance {
            background_color: iced::Color::TRANSPARENT,
            ..DefaultStyle::default_style(theme)
        }
    }

    fn workspace_infos_for_output(
        &self,
        wl_output: &WlOutput,
    ) -> impl Iterator<Item = WorkspaceInfo> {
        let output = self
            .output_infos
            .get(wl_output)
            .and_then(|oi| oi.name.as_ref());
        self.workspace_infos
            .iter()
            .filter(move |wi| wi.output.as_ref() == output)
            .cloned()
    }

    fn spawn_bar(&mut self, wl_output: WlOutput) -> Task<Message> {
        let id = window::Id::unique();

        let workspace_infos = self.workspace_infos_for_output(&wl_output).collect();
        self.bars.push(Bar {
            id,
            wl_output: wl_output.clone(),
            workspaces: if self.desktop.is_some() {
                Some(Workspaces::new(workspace_infos))
            } else {
                None
            },
        });

        get_layer_surface(SctkLayerSurfaceSettings {
            id,
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            pointer_interactivity: true,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            output: IcedOutput::Output(wl_output),
            namespace: "limbo:bar".to_string(),
            margin: IcedMargin {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            },
            size: Some((None, Some(40))),
            exclusive_zone: 40,
            size_limits: iced::Limits::NONE,
        })
    }
}
