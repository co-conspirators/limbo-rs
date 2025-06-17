use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
};

use niri_ipc::{Request, Response, Workspace, socket::Socket};
use tokio::sync::watch;

use super::{DesktopEvent, Monitor, MonitorInfo, WorkspaceInfo};

pub fn listen_monitors(mut socket: Socket) -> mpsc::Receiver<Monitor> {
    let reply = socket
        .send(Request::EventStream)
        .expect("niri should be running")
        .expect("starting event stream should succeed");
    assert!(
        matches!(reply, Response::Handled),
        "niri should handle request successfully"
    );

    let read_event = socket.read_events();
    let (mtx, mrx) = mpsc::channel();
    std::thread::spawn(move || run(read_event, mtx));

    mrx
}

fn run(
    mut read_event: impl FnMut() -> std::io::Result<niri_ipc::Event>,
    mtx: mpsc::Sender<Monitor>,
) {
    let mut workspaces = HashMap::<u64, Workspace>::new();
    let mut senders = HashMap::<String, watch::Sender<DesktopEvent>>::new();
    let mut overview_open = false;

    while let Ok(event) = read_event() {
        use niri_ipc::Event::*;
        match event {
            WorkspacesChanged {
                workspaces: new_workspaces,
            } => {
                let new_workspaces: HashMap<u64, Workspace> =
                    new_workspaces.into_iter().map(|w| (w.id, w)).collect();

                let old_outpus = workspaces
                    .values()
                    .filter_map(|w| w.output.clone())
                    .collect::<HashSet<_>>();
                let new_outputs = new_workspaces
                    .values()
                    .filter_map(|w| w.output.clone())
                    .collect::<HashSet<_>>();

                for output in old_outpus.difference(&new_outputs) {
                    if let Some(tx) = senders.remove(output) {
                        let _ = tx.send(DesktopEvent::Quit);
                    }
                }

                for output in new_outputs {
                    let mut workspaces_on_output = new_workspaces
                        .values()
                        .filter(|w| w.output.as_ref().map(|w| *w == output).unwrap_or_default())
                        .collect::<Vec<_>>();
                    workspaces_on_output.sort_by_key(|w| w.idx);

                    let workspaces_infos = workspaces_on_output
                        .iter()
                        .map(|w| WorkspaceInfo {
                            has_windows: w.active_window_id.is_some(),
                        })
                        .collect::<Vec<_>>();

                    let active_workspace_idx = workspaces_on_output
                        .iter()
                        .find(|w| w.is_active)
                        .map(|w| w.idx - 1);
                    let show_transparent = overview_open
                        || !active_workspace_idx
                            .map(|idx| workspaces_infos[idx as usize].has_windows)
                            .unwrap_or_default();

                    let workspaces_info = MonitorInfo {
                        workspaces: workspaces_infos,
                        active_workspace_idx,
                        show_transparent,
                    };

                    if let Some(tx) = senders.get_mut(&output) {
                        let _ = tx.send_if_modified(|e| {
                            if let DesktopEvent::MonitorInfoEvent(wi) = e {
                                *wi = workspaces_info;
                                true
                            } else {
                                false
                            }
                        });
                    } else {
                        let (tx, rx) =
                            watch::channel(DesktopEvent::MonitorInfoEvent(workspaces_info));

                        let _ = mtx.send(Monitor::new(output.clone(), rx));
                        senders.insert(output.clone(), tx);
                    }
                }

                workspaces = new_workspaces;
            }
            WorkspaceActivated { id, focused: _ } => {
                if let Some(workspace) = workspaces.get(&id)
                    && let Some(output) = &workspace.output
                    && let Some(tx) = senders.get_mut(output)
                {
                    let idx = workspace.idx - 1;
                    let _ = tx.send_if_modified(|e| {
                        if let DesktopEvent::MonitorInfoEvent(w) = e {
                            let show_transparent = overview_open
                                || !w
                                    .workspaces
                                    .get(idx as usize)
                                    .map(|w| w.has_windows)
                                    .unwrap_or_default();
                            let modified = (w.show_transparent, w.active_workspace_idx)
                                != (show_transparent, Some(idx));
                            w.show_transparent = show_transparent;
                            w.active_workspace_idx = Some(idx);
                            modified
                        } else {
                            false
                        }
                    });
                }
            }
            WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                if let Some(workspace) = workspaces.get(&workspace_id)
                    && let Some(output) = &workspace.output
                    && let Some(tx) = senders.get_mut(output)
                {
                    let has_windows = active_window_id.is_some();
                    let _ = tx.send_if_modified(|e| {
                        if let DesktopEvent::MonitorInfoEvent(w) = e
                            && let Some(wi) = w
                                .active_workspace_idx
                                .and_then(|i| w.workspaces.get_mut(i as usize))
                        {
                            let show_transparent = overview_open || !wi.has_windows;
                            let modified = (w.show_transparent, wi.has_windows)
                                != (show_transparent, has_windows);
                            w.show_transparent = show_transparent;
                            wi.has_windows = has_windows;
                            modified
                        } else {
                            false
                        }
                    });
                }
            }
            OverviewOpenedOrClosed { is_open } => {
                overview_open = is_open;
                for tx in senders.values_mut() {
                    let _ = tx.send_if_modified(|e| {
                        if let DesktopEvent::MonitorInfoEvent(w) = e {
                            let show_transparent = overview_open
                                || !w
                                    .active_workspace_idx
                                    .and_then(|idx| w.workspaces.get(idx as usize))
                                    .map(|w| w.has_windows)
                                    .unwrap_or_default();
                            let modified = w.show_transparent != show_transparent;
                            w.show_transparent = show_transparent;
                            modified
                        } else {
                            false
                        }
                    });
                }
            }
            _ => {}
        }
    }
}
