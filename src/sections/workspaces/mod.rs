use iced::advanced::mouse;
use iced::widget::{Row, container, mouse_area, text};
use iced::{Border, Color};

use crate::GlobalState;
use crate::components::section;
use crate::desktop_environment::WorkspaceInfo;
use crate::message::Message;

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
                let color = if w.info.has_windows || w.info.is_active {
                    Color::from_rgb8(137, 180, 250)
                } else {
                    Color::from_rgb8(88, 91, 112)
                };
                let width = w.width();

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
                .on_press(Message::FocusWorkspace(w.info.id))
                .on_scroll(|delta| {
                    let y = match delta {
                        mouse::ScrollDelta::Pixels { y, .. } => y,
                        mouse::ScrollDelta::Lines { y, .. } => y,
                    };
                    if y == 0.0 {
                        Message::AnimationTick
                    } else {
                        // TODO: debounce scrolling with `ScrollDelta::Pixels`
                        Message::CycleWorkspace { forward: y <= 0.0 }
                    }
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
