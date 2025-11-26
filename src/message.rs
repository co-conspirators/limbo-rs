use iced::event::wayland::Event as WaylandEvent;
use iced::window;

use crate::desktop_environment::{WorkspaceId, WorkspaceInfo};
use crate::sections::SysInfo;

#[derive(Debug, Clone)]
pub enum BarMessage {
    ClockToggleExpanded,
}

#[derive(Debug, Clone)]
pub enum Message {
    Wayland(WaylandEvent),
    Bar(window::Id, BarMessage),

    WorkspacesChanged(Vec<WorkspaceInfo>),
    FocusWorkspace(WorkspaceId),
    CycleWorkspace {
        forward: bool,
    },

    ClockTick {
        local_time: jiff::Zoned,
        expanded: bool,
    },

    SysinfoUpdate(SysInfo),
    TrayItemsUpdate(Vec<crate::tray::TrayItem>),

    AnimationTick,
}
