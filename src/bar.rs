use iced::platform_specific::shell::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface,
};
use iced::runtime::platform_specific::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use iced::widget::{container, row};
use iced::{Alignment, Element, Length, Task, Theme, window};
use sctk::reexports::client::protocol::wl_output::WlOutput;
use sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer};

use crate::GlobalState;
use crate::components::{icon, section, side};
use crate::desktop_environment::WorkspaceInfo;
use crate::message::Message;
use crate::sections::{Clock, Sysmon, TrayView, Workspaces};

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
    pub fn new(
        wl_output: WlOutput,
        output_name: String,
        global_state: &GlobalState,
    ) -> (Self, Task<Message>) {
        let id = window::Id::unique();
        (
            Self {
                id,
                wl_output: wl_output.clone(),
                output_name: output_name.clone(),
                transparent: is_transparent(&output_name, &global_state.workspace_infos),

                workspaces: Workspaces::new(output_name, global_state),
                clock: Clock::new(),
                sysmon: Sysmon::new(global_state),
                tray_view: TrayView::new(global_state),
            },
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
            }),
        )
    }

    pub fn destroy(self) -> Task<Message> {
        destroy_layer_surface(self.id)
    }

    pub fn update(&mut self, message: &Message) {
        self.workspaces.update(message);
        self.clock.update(message, self.id);
        self.sysmon.update(message);
        self.tray_view.update(message);
        if let Message::WorkspacesChanged(workspace_infos) = message {
            self.transparent = is_transparent(&self.output_name, workspace_infos);
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

    pub fn animation_running(&self) -> bool {
        self.workspaces.animation_running()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.clock.subscription()
    }
}

fn is_transparent(output_name: &String, workspace_infos: &[WorkspaceInfo]) -> bool {
    workspace_infos
        .iter()
        .filter(|w| w.output.as_ref() == Some(output_name))
        .find(|w| w.is_active)
        .is_some_and(|w| w.transparent_bar)
}
