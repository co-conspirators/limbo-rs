use iced::{
    Color, Element, Settings, Task, Theme,
    daemon::{Appearance, DefaultStyle},
    event::wayland::{Event as WaylandEvent, LayerEvent, OutputEvent},
    platform_specific::shell::commands::layer_surface::{destroy_layer_surface, get_layer_surface},
    runtime::platform_specific::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    theme::Palette,
    window,
};
use sctk::{
    reexports::client::protocol::wl_output::WlOutput,
    shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer},
};

use crate::{desktop_environment::Desktop, message::Message, tray::Tray};

mod bar;
mod components;
mod desktop_environment;
mod icons;
mod message;
mod sections;
mod tray;

use bar::Bar;

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

struct Limbo {
    bars: Vec<Bar>,
    desktop: Desktop,
    tray: Tray,
}

impl Limbo {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                bars: Vec::new(),
                desktop: Desktop::new(),
                tray: Tray::new(),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let mut subscriptions = vec![
            iced::event::listen_with(|evt, _, _| {
                if let iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(evt)) =
                    evt
                    && matches!(
                        evt,
                        WaylandEvent::Output(_, _) | WaylandEvent::Layer(LayerEvent::Done, _, _)
                    )
                {
                    Some(Message::Wayland(evt))
                } else {
                    None
                }
            }),
            self.tray.subscription(),
            self.desktop.subscription(),
        ];
        subscriptions.extend(self.bars.iter().map(|bar| bar.subscription()));

        iced::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        for bar in self.bars.iter_mut() {
            bar.update(&message);
        }

        match message {
            Message::Wayland(evt) => match evt {
                WaylandEvent::Output(OutputEvent::Created(output_info), wl_output) => {
                    if let Some(output_name) = output_info.and_then(|o| o.name) {
                        self.spawn_bar(wl_output, output_name)
                    } else {
                        Task::none()
                    }
                }
                WaylandEvent::Output(OutputEvent::Removed, wl_output) => {
                    let removed_bars = self.bars.extract_if(.., |bar| bar.wl_output == wl_output);
                    Task::batch(removed_bars.map(|bar| destroy_layer_surface(bar.id)))
                }
                WaylandEvent::Layer(LayerEvent::Done, _wl_surface, id) => {
                    self.bars.retain(|bar| bar.id != id);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::FocusWorkspace(id) => {
                self.desktop.focus_workspace(id);
                Task::none()
            }
            Message::CycleWorkspace { forward } => {
                self.desktop.cycle_workspace(forward);
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

    fn spawn_bar(&mut self, wl_output: WlOutput, output_name: String) -> Task<Message> {
        let id = window::Id::unique();
        self.bars.push(Bar::new(id, wl_output.clone(), output_name));

        get_layer_surface(SctkLayerSurfaceSettings {
            id,
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            input_zone: None,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            output: IcedOutput::Output(wl_output),
            namespace: "limbo:bar".to_string(),
            margin: IcedMargin::default(),
            size: Some((None, Some(40))),
            exclusive_zone: 40,
            size_limits: iced::Limits::NONE,
        })
    }
}
