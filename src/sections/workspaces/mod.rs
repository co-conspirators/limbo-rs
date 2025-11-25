use std::time::Duration;

use iced::advanced::mouse;
use iced::widget::{Row, container, mouse_area, text};
use iced::{Border, Color};

use crate::components::section;
use crate::desktop_environment::{WorkspaceId, WorkspaceInfo};

mod state;
use state::WorkspaceState;

pub struct Workspaces {
    states: Vec<WorkspaceState>,
}

#[derive(Debug, Clone)]
pub enum WorkspacesMessage {
    Tick,
    WorkspacesChanged(Vec<WorkspaceInfo>),
    FocusWorkspace(WorkspaceId),
    CycleWorkspace(bool),
}

impl Workspaces {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn update(&mut self, message: WorkspacesMessage) {
        match message {
            WorkspacesMessage::Tick => {
                for w in &mut self.states {
                    w.update();
                }
            }
            WorkspacesMessage::WorkspacesChanged(workspace_infos) => {
                self.states = workspace_infos
                    .into_iter()
                    .map(|info| WorkspaceState::from_existing(&self.states, info))
                    .collect();
            }
            _ => {}
        }
    }

    pub fn view(&self) -> iced::Element<'_, WorkspacesMessage> {
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
                .on_press(WorkspacesMessage::FocusWorkspace(w.id))
                .on_scroll(|delta| {
                    let y = match delta {
                        mouse::ScrollDelta::Pixels { y, .. } => y,
                        mouse::ScrollDelta::Lines { y, .. } => y,
                    };
                    match y {
                        0.0 => WorkspacesMessage::Tick,
                        ..0. => WorkspacesMessage::CycleWorkspace(true),
                        0.0.. => WorkspacesMessage::CycleWorkspace(false),
                        _ => WorkspacesMessage::Tick,
                    }
                })
                .into()
            })
            .collect::<Vec<_>>();

        section(Row::from_vec(workspace_icons))
            .padding([0, 8])
            .into()
    }

    pub fn subscription(&self) -> Option<iced::Subscription<WorkspacesMessage>> {
        self.states
            .iter()
            .any(|w| w.needs_update())
            .then(|| iced::time::every(Duration::from_millis(25)).map(|_| WorkspacesMessage::Tick))
    }
}
