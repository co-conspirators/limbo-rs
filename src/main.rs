use std::thread;

use iced::border::Radius;
use iced::widget::{Space, Svg, container, row, svg, text};
use iced::{Alignment, Border, Color, Element, Event, Length, Task as Command, Theme};
use iced_layershell::Application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;

use hyprland::data::{Monitors, Workspace, Workspaces};
use hyprland::prelude::*;

use crate::components::{icon, section, side};
use crate::hyprland_listener::hyprland_subscription;
use crate::sections::clock::{Clock, ClockMessage};

mod components;
mod hyprland_listener;
mod icons;
mod sections;

use icons::Icons;

#[tokio::main]
pub async fn main() -> () {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    // Run on all monitors
    let monitors = Monitors::get().expect("failed to get hyprland monitors");
    let monitors = monitors
        .iter()
        .map(|monitor| monitor.name.clone())
        .collect::<Vec<_>>();

    let tasks = monitors
        .into_iter()
        .map(|monitor| {
            thread::spawn(move || {
                Limbo::run(Settings {
                    layer_settings: LayerShellSettings {
                        size: Some((0, 40)),
                        exclusive_zone: 40,
                        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                        keyboard_interactivity: KeyboardInteractivity::None,
                        start_mode: StartMode::TargetScreen(monitor.clone()),
                        ..Default::default()
                    },
                    flags: Flags { monitor },
                    ..Default::default()
                })
            })
        })
        .collect::<Vec<_>>();

    for task in tasks {
        task.join().unwrap().unwrap();
    }
}

#[derive(Debug, Clone, Default)]
struct Flags {
    monitor: String,
}

struct Limbo {
    monitor: String,

    clock: Clock,
}

// Because new iced delete the custom command, so now we make a macro crate to generate
// the Command
#[to_layer_message]
#[derive(Debug, Clone)]
#[doc = "Some docs"]
pub enum Message {
    WorkspaceChanged(i32),
    WorkspaceCreated(i32),
    WorkspaceDestroyed(i32),
    IcedEvent(Event),

    Clock(ClockMessage),
}

impl Application for Limbo {
    type Message = Message;
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                monitor: flags.monitor,
                clock: Clock::new(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        "Limbo".to_string()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let subscriptions = vec![
            hyprland_subscription(),
            self.clock.subscription().map(Message::Clock),
        ];
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => {
                println!("{event:?}");
                Command::none()
            }
            Message::WorkspaceChanged(id) => {
                println!("workspace changed to {id:?}");
                Command::none()
            }
            Message::WorkspaceCreated(id) => {
                println!("workspace created: {id:?}");
                Command::none()
            }
            Message::WorkspaceDestroyed(id) => {
                println!("workspace destroyed: {id:?}");
                Command::none()
            }
            Message::Clock(msg) => {
                self.clock.update(msg);
                Command::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
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
            side(
                Alignment::Center,
                row![self.clock.view().map(Message::Clock)].spacing(12)
            ),
            // Right
            side(
                Alignment::End,
                row![section(icon("nix-snowflake-white", None))].spacing(12)
            ),
        ]
        .padding([4, 8])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::CatppuccinMocha
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        use iced_layershell::Appearance;

        let active_workspaces = Monitors::get()
            .unwrap()
            .iter()
            .map(|m| m.active_workspace.id)
            .collect::<Vec<_>>();
        let workspaces = Workspaces::get().unwrap();
        let workspace = workspaces
            .iter()
            .find(|w| w.monitor == self.monitor && active_workspaces.contains(&w.id))
            .unwrap();

        Appearance {
            background_color: if workspace.windows > 0 {
                theme.palette().background
            } else {
                Color::TRANSPARENT
            },
            text_color: theme.palette().text,
        }
    }
}
