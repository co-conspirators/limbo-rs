use std::time::Duration;

use iced::{
    Border, Color,
    advanced::mouse,
    widget::{Row, container, mouse_area, text},
};

use crate::{
    components::section,
    desktop_environment::{Desktop, WorkspaceId, WorkspaceInfos},
};

mod state;
use state::WorkspaceState;

pub struct Workspaces {
    workspaces: Vec<WorkspaceState>,
}

#[derive(Debug, Clone)]
pub enum WorkspacesMessage {
    Tick,
    WorkspacesChanged(WorkspaceInfos),
    FocusWorkspace(WorkspaceId),
    CycleWorkspace(bool),
}

impl Workspaces {
    pub fn new(mut workspace_infos: WorkspaceInfos) -> Self {
        workspace_infos.sort_by_key(|w| w.idx);
        Self {
            workspaces: workspace_infos
                .into_iter()
                .map(WorkspaceState::new)
                .collect(),
        }
    }

    pub fn update(&mut self, message: WorkspacesMessage, desktop: &mut Desktop) {
        match message {
            WorkspacesMessage::Tick => {
                for w in &mut self.workspaces {
                    w.update();
                }
            }
            WorkspacesMessage::WorkspacesChanged(mut workspace_infos) => {
                workspace_infos.sort_by_key(|w| w.idx);
                let len = workspace_infos.len();
                for (i, workspace_info) in workspace_infos.into_iter().enumerate() {
                    if let Some(state) = self.workspaces.get_mut(i) {
                        state.workspace_info = workspace_info;
                    } else {
                        self.workspaces.push(WorkspaceState::new(workspace_info));
                    }
                }
                self.workspaces.truncate(len);
            }
            WorkspacesMessage::FocusWorkspace(workspace_id) => {
                desktop.focus_workspace(workspace_id)
            }
            WorkspacesMessage::CycleWorkspace(forward) => desktop.cycle_workspace(forward),
        }
    }

    pub fn view(&self) -> iced::Element<'_, WorkspacesMessage> {
        let workspace_icons = self
            .workspaces
            .iter()
            .map(|w| {
                let color = if w.workspace_info.has_windows || w.workspace_info.is_active {
                    Color::from_rgb8(137, 180, 250)
                } else {
                    Color::from_rgb8(88, 91, 112)
                };
                let width = (w.current_width_multiplier() * 5.).floor() as u16;

                mouse_area(
                    container(container(text("")).padding([5, width]).style(
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
                    .padding([8, 15 - width]),
                )
                .on_press(WorkspacesMessage::FocusWorkspace(w.workspace_info.id))
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
        self.workspaces
            .iter()
            .any(|w| w.needs_update())
            .then(|| iced::time::every(Duration::from_millis(8)).map(|_| WorkspacesMessage::Tick))
    }
}
