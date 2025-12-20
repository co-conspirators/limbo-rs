use std::rc::Rc;

use iced::platform_specific::shell::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface,
};
use iced::runtime::platform_specific::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use iced::widget::{Row, container, row};
use iced::{Alignment, Element, Event, Length, Size, Task, Theme, window};
use sctk::reexports::client::protocol::wl_output::WlOutput;
use sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer};

use crate::GlobalState;
use crate::animation::{Eased, Easing};
use crate::components::{icon, side};
use crate::config::Config;
use crate::config::types::ModuleName;
use crate::desktop_environment::WorkspaceInfo;
use crate::message::Message;
use crate::sections::{BatteryView, Clock, QuickSettings, Sysmon, Workspaces};

pub struct Bar {
    /// window id of the bar's layer surface.
    pub id: window::Id,
    pub wl_output: WlOutput,
    output_name: String,
    size: Option<Size>,
    background_alpha_factor: Eased<f32>,

    config: Rc<Config>,
    workspaces: Workspaces,
    clock: Clock,
    sysmon: Sysmon,
    quick_settings: QuickSettings,
    battery_view: BatteryView,
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
                size: None,
                background_alpha_factor: Eased::new(
                    alpha_target_idx(&output_name, &global_state.workspace_infos),
                    Easing::Smoothstep,
                    200.,
                    &[1.0, 0.0],
                ),

                config: global_state.config.clone(),
                workspaces: Workspaces::new(output_name, global_state),
                clock: Clock::new(global_state),
                sysmon: Sysmon::new(global_state),
                quick_settings: QuickSettings::new(global_state),
                battery_view: BatteryView::new(global_state),
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
        self.clock.update(message);
        self.sysmon.update(message);
        self.quick_settings.update(message);
        self.battery_view.update(message);
        match message {
            Message::AnimationTick => {
                self.background_alpha_factor.update();
            }
            Message::WorkspacesChanged(workspace_infos) => {
                self.background_alpha_factor
                    .set_target_idx(alpha_target_idx(&self.output_name, workspace_infos));
            }
            Message::Iced(window_id, Event::Window(window::Event::Opened { size, .. }))
                if *window_id == self.id =>
            {
                self.size = Some(*size)
            }
            _ => (),
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        let background_alpha_factor = self.background_alpha_factor.get();

        let todo_module = || {
            self.config
                .section(icon("nix-snowflake-white", None))
                .into()
        };

        let mk_side = |modules: &Vec<ModuleName>| {
            Row::from_iter(modules.iter().map(|module| match module {
                ModuleName::AppLauncher => todo_module(),
                ModuleName::Battery => self.battery_view.view(),
                ModuleName::Clock => self.clock.view(),
                ModuleName::Music => todo_module(),
                ModuleName::Notifications => todo_module(),
                ModuleName::QuickSettings => self.quick_settings.view(),
                ModuleName::Sysmon => self.sysmon.view(),
                ModuleName::Todo => todo_module(),
                ModuleName::Workspaces => self.workspaces.view(),
            }))
            .spacing(12)
        };

        let left = mk_side(&self.config.bar.modules.left);
        let center = mk_side(&self.config.bar.modules.center);
        let right = mk_side(&self.config.bar.modules.right);

        container(
            row![
                side(Alignment::Start, left),
                side(Alignment::Center, center),
                side(Alignment::End, right),
            ]
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .style(move |theme: &Theme| {
            iced::widget::container::background(
                theme
                    .palette()
                    .background
                    .scale_alpha(background_alpha_factor),
            )
        })
        .into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.clock.subscription()
    }

    /// Whether the window has opened, indicated by receiving an
    /// `iced::Event::Window(iced::window::Event::Opened { .. })`
    pub fn opened(&self) -> bool {
        self.size.is_some()
    }

    pub fn animation_running(&self) -> bool {
        self.background_alpha_factor.is_running() || self.workspaces.animation_running()
    }
}

fn alpha_target_idx(output_name: &String, workspace_infos: &[WorkspaceInfo]) -> usize {
    if workspace_infos
        .iter()
        .filter(|w| w.output.as_ref() == Some(output_name))
        .find(|w| w.is_active)
        .is_some_and(|w| w.transparent_bar)
    {
        1
    } else {
        0
    }
}
