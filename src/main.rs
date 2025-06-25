use std::{collections::HashMap, sync::LazyLock};

use iced::{
    Alignment, Color, Element, Event, Length, Padding, Size, Task, Theme, layer_shell,
    theme::Palette, widget::row, window,
};

use crate::{
    components::{icon, section, side},
    desktop_environment::OutputInfo,
    sections::clock::{Clock, ClockMessage},
};

mod components;
mod desktop_environment;
mod icons;
mod sections;

static CATPPUCCIN_MOCHA_TRANSPARENT: LazyLock<Theme> = LazyLock::new(|| {
    Theme::custom(
        "CatpuccinMocha Transparent",
        Palette {
            background: Color::TRANSPARENT,
            ..Palette::CATPPUCCIN_MOCHA
        },
    )
});

pub fn main() {
    // Workaround for https://github.com/friedow/centerpiece/issues/237
    // WGPU picks the lower power GPU by default, which on some systems,
    // will pick an IGPU that doesn't exist leading to a black screen.
    if std::env::var("WGPU_POWER_PREF").is_err() {
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }

    iced::daemon(Limbo::new, Limbo::update, Limbo::view)
        .subscription(Limbo::subscription)
        .theme(Limbo::theme)
        .run()
        .unwrap();
}

struct Limbo {
    layers: HashMap<String, window::Id>,
    layer_aliases: HashMap<window::Id, String>,
    output_infos: HashMap<String, OutputInfo>,

    clock: Clock,
}

#[derive(Debug, Clone)]
pub enum Message {
    DesktopEvent(OutputInfo),
    IcedEvent(Event),
    WaylandEvent(iced::event::Wayland),

    LayerOpened((window::Id, String)),

    Clock(ClockMessage),
}

impl Limbo {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                layers: HashMap::new(),
                layer_aliases: HashMap::new(),
                output_infos: HashMap::new(),
                clock: Clock::new(),
            },
            desktop_environment::listen()
                .map(|stream| Task::stream(stream).map(Message::DesktopEvent))
                .unwrap_or(Task::none()),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let subscriptions = vec![
            iced::event::listen_wayland().map(Message::WaylandEvent),
            self.clock.subscription().map(Message::Clock),
        ];
        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DesktopEvent(output_info) => {
                let _ = self
                    .output_infos
                    .insert(output_info.name.clone(), output_info);
                Task::none()
            }
            Message::IcedEvent(event) => {
                println!("{event:?}");
                Task::none()
            }
            Message::WaylandEvent(event) => match event {
                iced::event::Wayland::OutputAdded(output) => {
                    let (_id, task) = layer_shell::open(layer_shell::Settings {
                        layer: layer_shell::Layer::Top,
                        namespace: Some("limbo".to_string()),
                        size: Size::new(0, 40),
                        anchor: layer_shell::Anchor::TOP
                            | layer_shell::Anchor::LEFT
                            | layer_shell::Anchor::RIGHT,
                        exclusive_zone: 40,
                        margin: Padding::default(),
                        keyboard_interactivity: layer_shell::KeyboardInteractivity::None,
                        output: Some(output.clone()),
                    });

                    task.map(move |id| Message::LayerOpened((id, output.clone())))
                }
                iced::event::Wayland::OutputRemoved(output) => {
                    if let Some(id) = self.layers.remove(&output) {
                        let _ = self.layer_aliases.remove(&id);
                        layer_shell::close::<Message>(id).discard()
                    } else {
                        Task::none()
                    }
                }
            },
            Message::LayerOpened((id, output)) => {
                let _ = self.layers.insert(output.clone(), id);
                let _ = self.layer_aliases.insert(id, output);
                Task::none()
            }
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
        }
    }

    fn view(&self, _id: window::Id) -> Element<'_, Message> {
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

    fn theme(&self, id: window::Id) -> Theme {
        let show_transparent = self
            .layer_aliases
            .get(&id)
            .and_then(|output| self.output_infos.get(output))
            .is_some_and(|info| info.show_transparent);

        if show_transparent {
            CATPPUCCIN_MOCHA_TRANSPARENT.clone()
        } else {
            Theme::CatppuccinMocha
        }
    }
}
