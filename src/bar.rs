use iced::{
    Alignment, Element, Length, Theme,
    widget::{container, row},
    window,
};
use sctk::reexports::client::protocol::wl_output::WlOutput;

use crate::{
    components::{icon, section, side},
    message::Message,
    sections::{Clock, Sysmon, TrayView, Workspaces},
};

pub struct Bar {
    /// window id of the bar's layer surface.
    pub id: window::Id,
    pub wl_output: WlOutput,
    output_name: String,
    transparent: bool,

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
            transparent: false,

            workspaces: Workspaces::new(),
            clock: Clock::new(),
            sysmon: Sysmon::new(),
            tray_view: TrayView::new(),
        }
    }

    pub fn update(&mut self, message: &Message) {
        self.workspaces.update(message);
        self.clock.update(message, self.id);
        self.sysmon.update(message);
        self.tray_view.update(message);
        if let Message::WorkspacesChanged(workspace_infos) = message {
            self.transparent = workspace_infos
                .iter()
                .filter(|w| w.output.as_ref() == Some(&self.output_name))
                .find(|w| w.is_active)
                .is_some_and(|w| w.transparent_bar);
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(
            row![
                // Left
                side(
                    Alignment::Start,
                    row![section(icon("nix-snowflake-white", None))].spacing(12)
                ),
                // Center
                side(Alignment::Center, row![self.workspaces.view()].spacing(12)),
                // Right
                side(
                    Alignment::End,
                    row![
                        self.tray_view.view(),
                        self.sysmon.view(),
                        self.clock.view(self.id)
                    ]
                    .spacing(12)
                ),
            ]
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(move |theme: &Theme| {
            if self.transparent {
                iced::widget::container::transparent(theme)
            } else {
                iced::widget::container::background(theme.palette().background)
            }
        })
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let mut subscriptions = vec![self.clock.subscription(), self.sysmon.subscription()];
        if let Some(subscription) = self.workspaces.subscription() {
            subscriptions.push(subscription);
        }
        iced::Subscription::batch(subscriptions)
    }
}
