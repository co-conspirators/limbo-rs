use std::rc::Rc;
use std::sync::Arc;

use iced::daemon::{Appearance, DefaultStyle};
use iced::event::{PlatformSpecific, wayland};
use iced::theme::Palette;
use iced::{Color, Element, Event, Settings, Task, Theme, window};

use crate::battery::{Battery, BatteryState};
use crate::desktop_environment::{Desktop, WorkspaceInfo};
use crate::message::Message;
use crate::sections::{SysInfo, Sysmon};
use crate::tray::{Tray, TrayItem};

mod animation;
mod bar;
mod battery;
mod components;
mod config;
mod desktop_environment;
mod icons;
mod message;
mod sections;
mod tray;

use bar::Bar;
use config::Config;

pub fn main() -> iced::Result {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    let config = Config::load().unwrap_or_default();

    let font_name = config.theme.font.clone().leak();
    iced::daemon("limbo", Limbo::update, Limbo::view)
        .settings(Settings {
            id: Some("limbo".to_string()),
            fonts: Vec::new(),
            default_font: iced::Font {
                family: iced::font::Family::Name(font_name),
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
        .run_with(move || Limbo::new(config))
}

/// Global state for use when initializing new bars.
#[derive(Default)]
pub struct GlobalState {
    config: Rc<Config>,
    workspace_infos: Vec<WorkspaceInfo>,
    sysinfo: SysInfo,
    tray_items: Arc<Vec<TrayItem>>,
    battery_state: BatteryState,
}

struct Limbo {
    global_state: GlobalState,
    bars: Vec<Bar>,
    desktop: Desktop,
    tray: Option<Tray>,
    battery: Option<Battery>,
}

impl Limbo {
    fn new(config: Config) -> (Self, Task<Message>) {
        (
            Self {
                global_state: GlobalState {
                    config: Rc::new(config),
                    ..Default::default()
                },
                bars: Vec::new(),
                desktop: Desktop::new(),
                tray: None,
                battery: None,
            },
            Task::batch([
                Tray::new().map(Message::TrayInit),
                Battery::new().map(Message::BatteryInit),
            ]),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let mut subscriptions = vec![
            iced::event::listen_with(|evt, _, window_id| match evt {
                Event::PlatformSpecific(PlatformSpecific::Wayland(
                    wayland::Event::Output(_, _)
                    | wayland::Event::Layer(wayland::LayerEvent::Done, _, _),
                ))
                | Event::Window(window::Event::Opened { .. }) => {
                    Some(Message::Iced(window_id, evt))
                }
                _ => None,
            }),
            Sysmon::subscription(&self.global_state.config),
            self.desktop.subscription(),
        ];

        if let Some(tray) = self.tray.as_ref() {
            subscriptions.push(tray.subscription());
        }

        if let Some(battery) = self.battery.as_ref() {
            subscriptions.push(battery.subscription());
        }

        if self.animation_running() {
            subscriptions.push(animation::subscription());
        }

        subscriptions.extend(self.bars.iter().map(|bar| bar.subscription()));

        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        for bar in self.bars.iter_mut() {
            bar.update(&message);
        }

        match message {
            Message::Iced(_, Event::PlatformSpecific(PlatformSpecific::Wayland(evt))) => {
                match evt {
                    wayland::Event::Output(
                        wayland::OutputEvent::Created(output_info),
                        wl_output,
                    ) => {
                        if let Some(output_name) = output_info.and_then(|o| o.name) {
                            let (bar, spawn_task) =
                                Bar::new(wl_output.clone(), output_name, &self.global_state);
                            self.bars.push(bar);
                            spawn_task
                        } else {
                            Task::none()
                        }
                    }
                    wayland::Event::Output(wayland::OutputEvent::Removed, wl_output) => {
                        let removed_bars =
                            self.bars.extract_if(.., |bar| bar.wl_output == wl_output);
                        Task::batch(removed_bars.map(|bar| bar.destroy()))
                    }
                    wayland::Event::Layer(wayland::LayerEvent::Done, _wl_surface, id) => {
                        self.bars.retain(|bar| bar.id != id);
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
            Message::TrayInit(Some(tray)) => {
                self.tray = Some(tray);
                Task::none()
            }
            Message::BatteryInit(Some(battery)) => {
                self.battery = Some(battery);
                Task::none()
            }
            Message::WorkspacesChanged(workspace_infos) => {
                self.global_state.workspace_infos = workspace_infos;
                Task::none()
            }
            Message::FocusWorkspace(id) => {
                self.desktop.focus_workspace(id);
                Task::none()
            }
            Message::CycleWorkspace { forward } => {
                self.desktop.cycle_workspace(forward);
                Task::none()
            }
            Message::SysinfoUpdate(sysinfo) => {
                self.global_state.sysinfo = sysinfo;
                Task::none()
            }
            Message::TrayItemsUpdate(tray_items) => {
                self.global_state.tray_items = tray_items;
                Task::none()
            }
            Message::BatteryUpdate(battery_state) => {
                self.global_state.battery_state = battery_state;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let bar = self
            .bars
            .iter()
            .find(|b| b.id == window_id)
            .expect("All windows are bars");
        bar.view()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        let cfg = &self.global_state.config;
        let text = cfg
            .theme
            .resolve_color(&cfg.bar.theme.fg)
            .unwrap_or(Color::WHITE);
        let background = cfg
            .theme
            .resolve_color(&cfg.bar.theme.bg)
            .unwrap_or(Color::from_rgb(
                0x1e as f32 / 255.0,
                0x1e as f32 / 255.0,
                0x2e as f32 / 255.0,
            ));

        Theme::custom(
            "internal".to_string(),
            Palette {
                text,
                background,
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

    fn animation_running(&self) -> bool {
        // HACK: Newly opened windows sometimes exhibit a rendering glitch where the initial
        // framebuffer is drawn with incorrect scaling, where text and other contents appear
        // horizontally stretched with visible interpolation artifacts. This appears to the user as
        // a flicker during window creation. The issue resolves itself as soon as any event
        // triggers a redraw. To mask this, we force early redraws by emitting
        // `Message::AnimationTick` while any window is opening.
        self.bars
            .iter()
            .any(|bar| !bar.opened() || bar.animation_running())
    }
}
