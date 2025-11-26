use std::time::Duration;

use iced::{
    Border, Color,
    advanced::mouse,
    widget::{Row, container, mouse_area, text},
};

use crate::{components::section, message::Message};

mod state;
use state::WorkspaceState;

pub struct Workspaces {
    states: Vec<WorkspaceState>,
}

impl Workspaces {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::AnimationTick => {
                for w in &mut self.states {
                    w.update();
                }
            }
            Message::WorkspacesChanged(workspace_infos) => {
                // TODO: filter by output name
                self.states = workspace_infos
                    .iter()
                    .map(|info| WorkspaceState::from_existing(&self.states, info.clone()))
                    .collect();
            }
            _ => {}
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let workspace_icons = self
            .states
            .iter()
            .map(|w| {
                let color = if w.info.has_windows || w.active {
                    Color::from_rgb8(137, 180, 250)
                } else {
                    Color::from_rgb8(88, 91, 112)
                };
                let width = 5. + w.animation_progress * 6.;

                mouse_area(
                    container(container(text("")).padding([5., width]).style(
                        move |_: &iced::Theme| {
                            container::Style {
                                background: Some(color.into()),
                                border: Border {
                                    radius: 20.0.into(), // High radius for pill shape
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                    ))
                    .padding([8., 15. - width]),
                )
                .on_press(Message::FocusWorkspace(w.id))
                .on_scroll(|delta| {
                    let y = match delta {
                        mouse::ScrollDelta::Pixels { y, .. } => y,
                        mouse::ScrollDelta::Lines { y, .. } => y,
                    };
                    Message::CycleWorkspace { forward: y <= 0.0 }
                })
                .into()
            })
            .collect::<Vec<_>>();

        section(Row::from_vec(workspace_icons))
            .padding([0, 8])
            .into()
    }

    pub fn subscription(&self) -> Option<iced::Subscription<Message>> {
        self.states
            .iter()
            .any(|w| w.needs_update())
            .then(|| iced::time::every(Duration::from_millis(25)).map(|_| Message::AnimationTick))
    }
}
