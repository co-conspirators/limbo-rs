use iced::widget::{container, row};
use iced::{Alignment, Element, Length, Theme, window};
use sctk::reexports::client::protocol::wl_output::WlOutput;

use crate::components::{icon, section, side};
use crate::desktop_environment::WorkspaceInfo;
use crate::sections::{
    Clock, ClockMessage, Sysmon, SysmonMessage, TrayMessage, TrayView, Workspaces,
    WorkspacesMessage,
};

#[derive(Debug, Clone)]
pub enum BarMessage {
    Workspaces(WorkspacesMessage),
    Clock(ClockMessage),
    Sysmon(SysmonMessage),
    Tray(TrayMessage),
}

pub struct Bar {
    /// window id of the bar's layer surface.
    pub id: window::Id,
    pub wl_output: WlOutput,
    output_name: String,
    workspace_infos: Vec<WorkspaceInfo>,

    workspaces: Workspaces,
    clock: Clock,
    sysmon: Sysmon,
    tray_view: TrayView,
}

impl Bar {
    pub fn new(id: window::Id, wl_output: WlOutput, output_name: String) -> Self {
        Self {
            id,
            wl_output,
            output_name,
            workspace_infos: vec![],

            workspaces: Workspaces::new(),
            clock: Clock::new(),
            sysmon: Sysmon::new(),
            tray_view: TrayView::new(),
        }
    }

    pub fn update(&mut self, message: BarMessage) {
        match message {
            BarMessage::Workspaces(WorkspacesMessage::WorkspacesChanged(workspace_infos)) => {
                self.workspace_infos = workspace_infos
                    .iter()
                    .filter(|w| w.output == Some(self.output_name.clone()))
                    .cloned()
                    .collect();
                self.workspaces.update(WorkspacesMessage::WorkspacesChanged(
                    self.workspace_infos.clone(),
                ));
            }
            BarMessage::Workspaces(msg) => self.workspaces.update(msg),
            BarMessage::Clock(msg) => self.clock.update(msg),
            BarMessage::Sysmon(msg) => self.sysmon.update(msg),
            BarMessage::Tray(msg) => self.tray_view.update(msg),
        };
    }

    pub fn view(&self) -> Element<'_, BarMessage> {
        let transparent = self
            .workspace_infos
            .iter()
            .find(|w| w.is_active)
            .is_some_and(|w| w.transparent_bar);

        container(
            row![
                // Left
                side(
                    Alignment::Start,
                    row![section(icon("nix-snowflake-white", None))].spacing(12)
                ),
                // Center
                side(
                    Alignment::Center,
                    row![self.workspaces.view().map(BarMessage::Workspaces)].spacing(12)
                ),
                // Right
                side(
                    Alignment::End,
                    row![
                        self.tray_view.view().map(BarMessage::Tray),
                        self.sysmon.view().map(BarMessage::Sysmon),
                        self.clock.view().map(BarMessage::Clock)
                    ]
                    .spacing(12)
                ),
            ]
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(move |theme: &Theme| {
            if transparent {
                iced::widget::container::transparent(theme)
            } else {
                iced::widget::container::background(theme.palette().background)
            }
        })
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<BarMessage> {
        let mut subscriptions = vec![
            self.clock.subscription().map(BarMessage::Clock),
            self.sysmon.subscription().map(BarMessage::Sysmon),
        ];
        if let Some(subscription) = self.workspaces.subscription() {
            subscriptions.push(subscription.map(BarMessage::Workspaces));
        }
        iced::Subscription::batch(subscriptions)
    }
}
