use iced::Color;

use crate::animation::{EasedToggle, Easing};
use crate::desktop_environment::WorkspaceInfo;

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub info: WorkspaceInfo,
    width: EasedToggle<f32>,
    color: EasedToggle<Color>,
}

impl From<WorkspaceInfo> for WorkspaceState {
    fn from(info: WorkspaceInfo) -> Self {
        Self {
            width: EasedToggle::new(info.is_active, Easing::Linear, 0.25, 5.0, 11.0),
            color: EasedToggle::new(
                info.has_windows || info.is_active,
                Easing::Smoothstep,
                0.25,
                Color::from_rgb8(88, 91, 112),
                Color::from_rgb8(137, 180, 250),
            ),
            info,
        }
    }
}

impl WorkspaceState {
    pub fn from_existing(states: &[WorkspaceState], info: WorkspaceInfo) -> Self {
        if let Some(state) = states.iter().find(|s| s.info.id == info.id) {
            Self {
                width: state.width.with_target(info.is_active),
                color: state.color.with_target(info.has_windows || info.is_active),
                info,
            }
        } else {
            Self::from(info)
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
