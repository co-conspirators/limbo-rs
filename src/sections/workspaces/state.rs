use iced::Color;

use crate::animation::{Eased, Easing};
use crate::desktop_environment::WorkspaceInfo;

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub info: WorkspaceInfo,
    width: Eased<f32>,
    color: Eased<Color>,
}

impl WorkspaceState {
    fn new(info: WorkspaceInfo, config: &crate::config::Config) -> Self {
        Self {
            width: Eased::new(width_target_idx(&info), Easing::Linear, 100., &[5.0, 11.0]),
            color: Eased::new(
                color_target_idx(&info),
                Easing::Smoothstep,
                100.,
                &[
                    config
                        .theme
                        .resolve_color(&config.bar.workspaces.color.normal)
                        .unwrap_or(Color::from_rgb8(88, 91, 112)),
                    config
                        .theme
                        .resolve_color(&config.bar.workspaces.color.has_windows)
                        .unwrap_or(Color::from_rgb8(88, 91, 112)),
                    config
                        .theme
                        .resolve_color(&config.bar.workspaces.color.active)
                        .unwrap_or(Color::from_rgb8(137, 180, 250)),
                ],
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
                width: state.width.clone().with_target_idx(width_target_idx(&info)),
                color: state.color.clone().with_target_idx(color_target_idx(&info)),
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

fn width_target_idx(info: &WorkspaceInfo) -> usize {
    if info.is_active { 1 } else { 0 }
}

fn color_target_idx(info: &WorkspaceInfo) -> usize {
    if info.is_active {
        2
    } else if info.has_windows {
        1
    } else {
        0
    }
}
