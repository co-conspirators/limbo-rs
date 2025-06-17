use iced::{Alignment, Color, Element, Event, Length, Task, Theme, widget::row};
use iced_layershell::{
    Application,
    reexport::{Anchor, KeyboardInteractivity},
    settings::{LayerShellSettings, Settings, StartMode},
    to_layer_message,
};

use crate::{
    components::{icon, section, side},
    desktop_environment::{DesktopEvent, Monitor, MonitorInfo},
    sections::clock::{Clock, ClockMessage},
};

mod components;
mod desktop_environment;
mod icons;
mod sections;

pub fn main() {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    let monitors = desktop_environment::listen_monitors();

    let mut tasks = vec![];
    while let Ok(monitor) = monitors.recv() {
        let task = std::thread::spawn(move || {
            Limbo::run(Settings {
                layer_settings: LayerShellSettings {
                    size: Some((0, 40)),
                    exclusive_zone: 40,
                    anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    start_mode: StartMode::TargetScreen(monitor.name()),
                    ..Default::default()
                },
                flags: Flags {
                    monitor, // desktop_msgs
                },
                id: None,
                fonts: Vec::new(),
                default_font: iced::Font::default(),
                default_text_size: iced::Pixels(16.0),
                antialiasing: false,
                virtual_keyboard_support: None,
            })
        });
        tasks.push(task);
    }

    for task in tasks {
        task.join().unwrap().unwrap();
    }
}

#[derive(Debug)]
struct Flags {
    monitor: Monitor,
}

struct Limbo {
    monitor: Monitor,
    workspaces_info: MonitorInfo,

    clock: Clock,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    DesktopEvent(DesktopEvent),
    IcedEvent(Event),

    Clock(ClockMessage),
}

impl Application for Limbo {
    type Message = Message;
    type Flags = Flags;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Task<Message>) {
        (
            Self {
                monitor: flags.monitor,
                workspaces_info: Default::default(),
                clock: Clock::new(),
            },
            Task::none(),
        )
    }

    fn namespace(&self) -> String {
        "Limbo".to_string()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let subscriptions = vec![
            self.monitor.subscription().map(Message::DesktopEvent),
            self.clock.subscription().map(Message::Clock),
        ];
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesktopEvent(DesktopEvent::Quit) => {
                iced::window::get_oldest().then(|id| iced::window::close(id.unwrap()))
            }
            Message::DesktopEvent(DesktopEvent::MonitorInfoEvent(workspaces_info)) => {
                self.workspaces_info = workspaces_info;
                Task::none()
            }
            Message::IcedEvent(event) => {
                println!("{event:?}");
                Task::none()
            }
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
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

        Appearance {
            background_color: if self.workspaces_info.show_transparent {
                Color::TRANSPARENT
            } else {
                theme.palette().background
            },
            text_color: theme.palette().text,
        }
    }
}
