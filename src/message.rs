use std::sync::Arc;

use iced::id::Id;
use iced::{Event, window};

use crate::battery::{Battery, BatteryState};
use crate::desktop_environment::{WorkspaceId, WorkspaceInfo};
use crate::sections::SysInfo;
use crate::tray::Tray;

#[derive(Debug, Clone)]
pub enum Message {
    Iced(window::Id, Event),

    WorkspacesChanged(Vec<WorkspaceInfo>),
    FocusWorkspace(WorkspaceId),
    CycleWorkspace { forward: bool },

    ClockToggleExpanded(Id),
    ClockTick(jiff::Zoned),

    SysinfoUpdate(SysInfo),

    TrayInit(Option<Tray>),
    TrayItemsUpdate(Arc<Vec<crate::tray::TrayItem>>),

    BatteryInit(Option<Battery>),
    BatteryUpdate(BatteryState),

    AnimationTick,
}
