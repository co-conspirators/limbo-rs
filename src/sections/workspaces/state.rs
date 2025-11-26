use crate::desktop_environment::{WorkspaceId, WorkspaceInfo};

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub id: WorkspaceId,
    pub active: bool,
    pub animation_progress: f32, // 0.0 to 1.0
    pub info: WorkspaceInfo,
}

impl From<WorkspaceInfo> for WorkspaceState {
    fn from(info: WorkspaceInfo) -> Self {
        Self {
            id: info.id,
            active: info.is_active,
            animation_progress: 0.0,
            info,
        }
    }
}

impl WorkspaceState {
    pub fn from_existing(states: &[WorkspaceState], info: WorkspaceInfo) -> Self {
        if let Some(state) = states.iter().find(|s| s.id == info.id) {
            Self {
                id: info.id,
                active: info.is_active,
                animation_progress: state.animation_progress,
                info,
            }
        } else {
            Self::from(info)
        }
    }

    pub fn animation_running(&self) -> bool {
        self.active && self.animation_progress < 1.0
            || !self.active && self.animation_progress > 0.0
    }

    pub fn update(&mut self) {
        const ANIMATION_SPEED: f32 = 0.25;

        if self.active && self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + ANIMATION_SPEED).min(1.0);
        } else if !self.active && self.animation_progress > 0.0 {
            self.animation_progress = (self.animation_progress - ANIMATION_SPEED).max(0.0);
        }
    }
}
