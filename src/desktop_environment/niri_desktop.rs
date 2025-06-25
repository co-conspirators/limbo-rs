use std::collections::{HashMap, HashSet};

use niri_ipc::{Request, Response, Workspace, socket::Socket};
use tokio::sync::mpsc;

use super::{OutputInfo, WorkspaceInfo};

fn run(
    mut read_event: impl FnMut() -> std::io::Result<niri_ipc::Event>,
    tx: mpsc::UnboundedSender<OutputInfo>,
) {
    let mut workspaces = HashMap::<u64, Workspace>::new();
    let mut output_infos = HashMap::<String, OutputInfo>::new();
    let mut overview_open = false;

    while let Ok(event) = read_event() {
        use niri_ipc::Event::*;
        match event {
            WorkspacesChanged {
                workspaces: new_workspaces,
            } => {
                output_infos.clear();
                workspaces = new_workspaces.into_iter().map(|w| (w.id, w)).collect();

                let outputs = workspaces
                    .values()
                    .filter_map(|w| w.output.clone())
                    .collect::<HashSet<_>>();

                for output in outputs {
                    let mut workspaces_on_output = workspaces
                        .values()
                        .filter(|w| w.output.as_ref().is_some_and(|w| *w == output))
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
                        .map(|w| w.idx as usize - 1);
                    let show_transparent = overview_open
                        || !active_workspace_idx
                            .and_then(|idx| workspaces_infos.get(idx))
                            .is_some_and(|w| w.has_windows);

                    let output_info = OutputInfo {
                        name: output.clone(),
                        workspaces: workspaces_infos,
                        active_workspace_idx,
                        show_transparent,
                    };
                    let _ = tx.send(output_info.clone());
                    output_infos.insert(output, output_info);
                }
            }
            WorkspaceActivated { id, focused: _ } => {
                if let Some(workspace) = workspaces.get(&id)
                    && let Some(output) = &workspace.output
                    && let Some(output_info) = output_infos.get_mut(output)
                {
                    let idx = workspace.idx as usize - 1;
                    let show_transparent = overview_open
                        || !output_info
                            .workspaces
                            .get(idx)
                            .is_some_and(|w| w.has_windows);

                    if (
                        output_info.active_workspace_idx,
                        output_info.show_transparent,
                    ) != (Some(idx), show_transparent)
                    {
                        output_info.active_workspace_idx = Some(idx);
                        output_info.show_transparent = show_transparent;
                        let _ = tx.send(output_info.clone());
                    }
                }
            }
            WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                if let Some(workspace) = workspaces.get(&workspace_id)
                    && let Some(output) = &workspace.output
                    && let Some(output_info) = output_infos.get_mut(output)
                    && let Some(active_workspace) = output_info
                        .active_workspace_idx
                        .and_then(|idx| output_info.workspaces.get_mut(idx))
                {
                    let has_windows = active_window_id.is_some();
                    let show_transparent = overview_open || !active_workspace.has_windows;

                    if (active_workspace.has_windows, output_info.show_transparent)
                        != (has_windows, show_transparent)
                    {
                        active_workspace.has_windows = has_windows;
                        output_info.show_transparent = show_transparent;
                        let _ = tx.send(output_info.clone());
                    }
                }
            }
            OverviewOpenedOrClosed { is_open } => {
                overview_open = is_open;
                for output_info in output_infos.values_mut() {
                    let show_transparent = overview_open
                        || !output_info
                            .active_workspace_idx
                            .and_then(|idx| output_info.workspaces.get(idx))
                            .is_some_and(|w| w.has_windows);

                    if output_info.show_transparent != show_transparent {
                        output_info.show_transparent = show_transparent;
                        let _ = tx.send(output_info.clone());
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn listen(mut socket: Socket, tx: mpsc::UnboundedSender<OutputInfo>) {
    let reply = socket
        .send(Request::EventStream)
        .expect("niri should be running")
        .expect("starting event stream should succeed");
    assert!(
        matches!(reply, Response::Handled),
        "niri should handle request successfully"
    );
    let read_event = socket.read_events();

    std::thread::Builder::new()
        .name("niri event listener".to_string())
        .spawn(move || run(read_event, tx))
        .unwrap();
}
