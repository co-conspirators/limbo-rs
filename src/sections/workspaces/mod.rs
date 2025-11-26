use std::time::Duration;

use iced::{
    Border, Color,
    advanced::mouse,
    widget::{Row, container, mouse_area, text},
};

use crate::{
    GlobalState, components::section, desktop_environment::WorkspaceInfo, message::Message,
};

mod state;
use state::WorkspaceState;

pub struct Workspaces {
    states: Vec<WorkspaceState>,
    output_name: String,
}

impl Workspaces {
    pub fn new(output_name: String, global_state: &GlobalState) -> Self {
        Self {
            states: update_states(&output_name, &global_state.workspace_infos, &[]),
            output_name,
        }
    }

    pub fn update(&mut self, message: &Message) {
        match message {
            Message::AnimationTick => {
                for w in &mut self.states {
                    w.update();
                }
            }
            Message::WorkspacesChanged(workspace_infos) => {
                self.states = update_states(&self.output_name, workspace_infos, &self.states);
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

    pub fn animation_running(&self) -> bool {
        self.states.iter().any(|w| w.animation_running())
    }
}

fn update_states(
    output_name: &String,
    workspace_infos: &[WorkspaceInfo],
    old_states: &[WorkspaceState],
) -> Vec<WorkspaceState> {
    workspace_infos
        .iter()
        .filter(|info| info.output.as_ref() == Some(output_name))
        .map(|info| WorkspaceState::from_existing(old_states, info.clone()))
        .collect()
}
