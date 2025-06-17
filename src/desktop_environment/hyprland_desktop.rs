use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
};

use hyprland::{
    data::{Monitor as HMonitor, Monitors, Workspace, Workspaces},
    event_listener::{EventListener, MonitorAddedEventData, WorkspaceEventData},
    shared::{HyprData, MonitorId, WorkspaceId},
};
use tokio::sync::watch;

use super::{Monitor, MonitorInfo};
use crate::desktop_environment::WorkspaceInfo;

pub fn listen_monitors() -> mpsc::Receiver<Monitor> {
    let (mtx, mrx) = mpsc::channel();
    tokio::task::spawn_blocking(move || run(mtx));

    mrx
}

fn get_monitor(id: MonitorId) -> Option<HMonitor> {
    Monitors::get().ok()?.into_iter().find(|m| m.id == id)
}

fn get_workspace(id: WorkspaceId) -> Option<Workspace> {
    Workspaces::get().ok()?.into_iter().find(|m| m.id == id)
}

fn make_monitor_info(monitor_id: MonitorId) -> Option<MonitorInfo> {
    let monitor = get_monitor(monitor_id)?;
    let mut workspaces = Workspaces::get()
        .ok()?
        .into_iter()
        .filter(|w| w.monitor_id == monitor_id)
        .collect::<Vec<_>>();
    workspaces.sort_by_key(|w| w.id);
    let active_workspace = workspaces
        .iter()
        .enumerate()
        .find(|(_, w)| w.id == monitor.active_workspace.id);

    Some(MonitorInfo {
        workspaces: workspaces
            .iter()
            .map(|w| WorkspaceInfo {
                has_windows: w.windows > 0,
            })
            .collect(),
        active_workspace_idx: active_workspace.map(|w| w.0),
        show_transparent: active_workspace
            .map(|(_, w)| w.windows == 0)
            .unwrap_or_default(),
    })
}

fn run(mtx: mpsc::Sender<Monitor>) {
    let mtx = mtx;

    let mut event_listener = EventListener::new();
    let senders = Arc::new(Mutex::new(
        HashMap::<MonitorId, watch::Sender<MonitorInfo>>::new(),
    ));

    let handler = {
        let mtx = mtx.clone();
        let senders = senders.clone();
        move |monitor_added_event_data: MonitorAddedEventData| {
            let Some(monitor_info) = make_monitor_info(monitor_added_event_data.id as MonitorId)
            else {
                return;
            };

            let (tx, mut rx) = watch::channel(monitor_info);
            rx.mark_changed();
            let _ = mtx.send(Monitor::new(monitor_added_event_data.name, rx));
            let mut senders = senders.lock().unwrap();
            senders.insert(monitor_added_event_data.id as MonitorId, tx);
        }
    };

    for monitor in Monitors::get().unwrap().into_iter() {
        handler(MonitorAddedEventData {
            id: monitor.id as u8,
            name: monitor.name,
            description: monitor.description,
        });
    }
    event_listener.add_monitor_added_handler(handler);

    fn mk_workspace_change_handler(
        senders: Arc<Mutex<HashMap<MonitorId, watch::Sender<MonitorInfo>>>>,
    ) -> impl Fn(WorkspaceEventData) {
        move |workspace_event_data: WorkspaceEventData| {
            let senders = senders.lock().expect("lock should not be poisoned");
            if let Some(workspace) = get_workspace(workspace_event_data.id)
                && let Some(monitor) = get_monitor(workspace.monitor_id)
                && let Some(tx) = senders.get(&monitor.id)
                && let Some(monitor_info) = make_monitor_info(monitor.id)
            {
                let _ = tx.send_if_modified(|mi| {
                    if mi != &monitor_info {
                        *mi = monitor_info;
                        true
                    } else {
                        false
                    }
                });
            };
        }
    }

    event_listener.add_workspace_changed_handler(mk_workspace_change_handler(senders.clone()));
    event_listener.add_workspace_added_handler(mk_workspace_change_handler(senders.clone()));
    event_listener.add_workspace_deleted_handler(mk_workspace_change_handler(senders.clone()));
    let handler = mk_workspace_change_handler(senders.clone());
    event_listener.add_workspace_moved_handler(move |workspace_moved_event_data| {
        handler(WorkspaceEventData {
            name: workspace_moved_event_data.name,
            id: workspace_moved_event_data.id,
        })
    });

    event_listener.start_listener().unwrap();
}
