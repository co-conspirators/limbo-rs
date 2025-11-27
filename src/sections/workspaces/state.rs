use iced::Color;

use crate::animation::{EasedToggle, Easing};
use crate::desktop_environment::WorkspaceInfo;

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub info: WorkspaceInfo,
    width: EasedToggle<f32>,
    color: EasedToggle<Color>,
}

impl WorkspaceState {
    fn new(info: WorkspaceInfo, config: &crate::config::Config) -> Self {
        Self {
            width: EasedToggle::new(info.is_active, Easing::Linear, 100., 5.0, 11.0),
            // TODO: use has_windows color
            color: EasedToggle::new(
                info.has_windows || info.is_active,
                Easing::Smoothstep,
                100.,
                config
                    .theme
                    .resolve_color(&config.bar.workspaces.color.normal)
                    .unwrap_or(Color::from_rgb8(88, 91, 112)),
                config
                    .theme
                    .resolve_color(&config.bar.workspaces.color.active)
                    .unwrap_or(Color::from_rgb8(137, 180, 250)),
            ),
            info,
        }
    }
}

impl WorkspaceState {
    pub fn from_existing(
        states: &[WorkspaceState],
        info: WorkspaceInfo,
        config: &crate::config::Config,
    ) -> Self {
        if let Some(state) = states.iter().find(|s| s.info.id == info.id) {
            Self {
                width: state.width.with_target(info.is_active),
                color: state.color.with_target(info.has_windows || info.is_active),
                info,
            }
        } else {
            Self::new(info, config)
        }
    }

    pub fn animation_running(&self) -> bool {
        self.width.is_running() || self.color.is_running()
    }

    pub fn update(&mut self) {
        self.width.update();
        self.color.update();
    }

    pub fn width(&self) -> f32 {
        self.width.get()
    }

    pub fn color(&self) -> Color {
        self.color.get()
    }
}
