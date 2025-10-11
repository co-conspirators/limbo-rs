use hyprland::{
    data::{Clients, Monitors, Workspaces},
    dispatch,
    dispatch::WorkspaceIdentifierWithSpecial,
    error::HyprError,
    event_listener::{Event as HyprEvent, EventStream},
    shared::HyprData,
};
use iced::futures::{StreamExt, stream::once};

use super::{DesktopEvent, WorkspaceId, WorkspaceInfo, WorkspaceInfos};

pub struct HyprlandDesktop;
impl HyprlandDesktop {
    pub fn new() -> Self {
        HyprlandDesktop
    }

    pub fn focus_workspace(&mut self, id: WorkspaceId) {
        let _ = dispatch!(Workspace, WorkspaceIdentifierWithSpecial::Id(id as i32));
    }

    pub fn cycle_workspace(&mut self, forward: bool) {
        let _ = dispatch!(
            Workspace,
            WorkspaceIdentifierWithSpecial::RelativeMonitor(match forward {
                true => 1,
                false => -1,
            })
        );
    }

    pub fn subscription(&self) -> iced::Subscription<DesktopEvent> {
        #[derive(Hash)]
        struct NiriEvents;

        iced::Subscription::run_with_id(
            NiriEvents,
            once(async {
                make_workspace_infos()
                    .await
                    .map(DesktopEvent::WorkspacesChanged)
            })
            .filter_map(|e| async { e })
            .chain(EventStream::new().filter_map(process_event)),
        )
    }
}

async fn process_event(event: Result<HyprEvent, HyprError>) -> Option<DesktopEvent> {
    let Ok(event) = event else {
        return None;
    };
    use HyprEvent::*;
    match event {
        MonitorAdded(_) | MonitorRemoved(_) | WorkspaceChanged(_) | WorkspaceDeleted(_)
        | WorkspaceAdded(_) | WorkspaceMoved(_) | WindowOpened(_) | WindowClosed(_)
        | WindowMoved(_) => make_workspace_infos()
            .await
            .map(DesktopEvent::WorkspacesChanged),
        _ => None,
    }
}

async fn make_workspace_infos() -> Option<WorkspaceInfos> {
    let monitors = Monitors::get_async()
        .await
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();
    let clients = Clients::get_async()
        .await
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();

    Some(
        Workspaces::get_async()
            .await
            .ok()?
            .into_iter()
            .map(|w| WorkspaceInfo {
                output: Some(w.monitor),
                id: w.id as WorkspaceId,
                idx: w.id,
                is_active: monitors.iter().any(|m| m.active_workspace.id == w.id),
                has_windows: w.windows > 0,
                transparent_bar: w.windows == 0
                    || clients
                        .iter()
                        .filter(|c| c.workspace.id == w.id)
                        .all(|c| c.floating),
            })
            .collect(),
    )
}
