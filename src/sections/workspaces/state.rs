use crate::desktop_environment::WorkspaceInfo;

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub workspace_info: WorkspaceInfo,
    animation_progress: f32, // 0.0 to 1.0
}

impl WorkspaceState {
    pub fn new(workspace_info: WorkspaceInfo) -> Self {
        Self {
            animation_progress: if workspace_info.is_active { 1.0 } else { 0.0 },
            workspace_info,
        }
    }

    pub fn needs_update(&self) -> bool {
        self.workspace_info.is_active && self.animation_progress < 1.0
            || !self.workspace_info.is_active && self.animation_progress > 0.0
    }

    pub fn update(&mut self) {
        const ANIMATION_SPEED: f32 = 0.08;

        if self.workspace_info.is_active && self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + ANIMATION_SPEED).min(1.0);
        } else if !self.workspace_info.is_active && self.animation_progress > 0.0 {
            self.animation_progress = (self.animation_progress - ANIMATION_SPEED).max(0.0);
        }
    }

    pub fn current_width_multiplier(&self) -> f32 {
        // Smooth easing function
        let t = self.animation_progress;
        let eased = t * t * (3.0 - 2.0 * t); // Smoothstep
        1.0 + eased * 1.25 // Goes from 1.0 to 2.25
    }
}
