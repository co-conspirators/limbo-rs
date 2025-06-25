use hyprland::{
    data::{Monitors, Workspaces},
    event_listener::{EventListener, WorkspaceEventData},
    shared::{HyprData, WorkspaceId},
};
use tokio::sync::mpsc;

use super::OutputInfo;
use crate::desktop_environment::WorkspaceInfo;

fn make_output_info(workspace_id: WorkspaceId) -> Option<OutputInfo> {
    let workspace = Workspaces::get()
        .ok()?
        .into_iter()
        .find(|m| m.id == workspace_id)?;
    let monitor = Monitors::get()
        .ok()?
        .into_iter()
        .find(|m| m.id == workspace.monitor_id)?;

    let mut workspaces = Workspaces::get()
        .ok()?
        .into_iter()
        .filter(|w| w.monitor_id == monitor.id)
        .collect::<Vec<_>>();
    workspaces.sort_by_key(|w| w.id);
    let active_workspace = workspaces
        .iter()
        .enumerate()
        .find(|(_, w)| w.id == monitor.active_workspace.id);

    Some(OutputInfo {
        name: monitor.name,
        workspaces: workspaces
            .iter()
            .map(|w| WorkspaceInfo {
                has_windows: w.windows > 0,
            })
            .collect(),
        active_workspace_idx: active_workspace.map(|w| w.0),
        show_transparent: active_workspace.is_some_and(|(_, w)| w.windows == 0),
    })
}

fn run(tx: mpsc::UnboundedSender<OutputInfo>) {
    let mut event_listener = EventListener::new();

    fn mk_handler(tx: mpsc::UnboundedSender<OutputInfo>) -> impl Fn(WorkspaceEventData) {
        move |workspace_event_data| {
            if let Some(output_info) = make_output_info(workspace_event_data.id) {
                let _ = tx.send(output_info);
            };
        }
    }

    event_listener.add_workspace_changed_handler(mk_handler(tx.clone()));
    event_listener.add_workspace_added_handler(mk_handler(tx.clone()));
    event_listener.add_workspace_deleted_handler(mk_handler(tx.clone()));
    let handler = mk_handler(tx.clone());
    event_listener.add_workspace_moved_handler(move |workspace_moved_event_data| {
        handler(WorkspaceEventData {
            name: workspace_moved_event_data.name,
            id: workspace_moved_event_data.id,
        })
    });

    event_listener.start_listener().unwrap();
}

pub fn listen(tx: mpsc::UnboundedSender<OutputInfo>) {
    std::thread::Builder::new()
        .name("hyprland event listener".to_string())
        .spawn(move || run(tx))
        .unwrap();
}
