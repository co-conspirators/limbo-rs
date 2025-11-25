use hyprland::data::{Clients, Monitors, WorkspaceRules, Workspaces};
use hyprland::dispatch;
use hyprland::dispatch::WorkspaceIdentifierWithSpecial;
use hyprland::error::HyprError;
use hyprland::event_listener::{Event as HyprEvent, EventStream};
use hyprland::shared::HyprData;
use iced::futures::StreamExt;
use iced::futures::stream::once;

use super::{DesktopEvent, WorkspaceId, WorkspaceInfo};

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
        struct HyprlandEvents;

        iced::Subscription::run_with_id(
            HyprlandEvents,
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
        | WindowMoved(_) | FloatStateChanged(_) => make_workspace_infos()
            .await
            .map(DesktopEvent::WorkspacesChanged),
        _ => None,
    }
}

async fn make_workspace_infos() -> Option<Vec<WorkspaceInfo>> {
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

    let all_workspaces = WorkspaceRules::get_async()
        .await
        .ok()?
        .into_iter()
        .filter_map(|rule| {
            rule.workspace_string
                .parse::<i64>()
                .ok()
                .map(|id| (id, rule.monitor))
        })
        .collect::<Vec<_>>();
    let workspaces = Workspaces::get_async()
        .await
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();

    Some(
        all_workspaces
            .into_iter()
            .map(|(id, output)| {
                if let Some(w) = workspaces.iter().find(|w| (w.id as i64) == id) {
                    WorkspaceInfo {
                        output: Some(w.monitor.clone()),
                        id: w.id as WorkspaceId,
                        idx: w.id,
                        is_active: monitors.iter().any(|m| m.active_workspace.id == w.id),
                        has_windows: w.windows > 0,
                        transparent_bar: w.windows == 0
                            || clients
                                .iter()
                                .filter(|c| c.workspace.id == w.id)
                                .all(|c| c.floating),
                    }
                } else {
                    WorkspaceInfo {
                        output,
                        id,
                        idx: id as i32,
                        is_active: false,
                        has_windows: false,
                        transparent_bar: false,
                    }
                }
            })
            .collect(),
    )
}
