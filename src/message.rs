use iced::id::Id;
use iced::{Event, window};

use crate::desktop_environment::{WorkspaceId, WorkspaceInfo};
use crate::sections::SysInfo;

#[derive(Debug, Clone)]
pub enum Message {
    Iced(window::Id, Event),

    WorkspacesChanged(Vec<WorkspaceInfo>),
    FocusWorkspace(WorkspaceId),
    CycleWorkspace { forward: bool },

    ClockToggleExpanded(Id),
    ClockTick(jiff::Zoned),

    SysinfoUpdate(SysInfo),
    TrayItemsUpdate(Vec<crate::tray::TrayItem>),

    AnimationTick,
}
