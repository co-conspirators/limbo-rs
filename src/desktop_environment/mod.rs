use std::sync::{Arc, mpsc};

use tokio::sync::{Mutex, watch};

#[cfg(feature = "hyprland")]
mod hyprland_desktop;
#[cfg(feature = "niri")]
mod niri_desktop;

#[allow(inactive_code)]
#[cfg(not(any(feature = "hyprland", feature = "niri")))]
compile_error!("At least one of \"hyprland\" or \"niri\" must be enabled.");

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WorkspaceInfo {
    pub has_windows: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonitorInfo {
    pub workspaces: Vec<WorkspaceInfo>,
    pub active_workspace_idx: Option<usize>,
    pub show_transparent: bool,
}

#[derive(Debug)]
pub struct Monitor {
    name: String,
    rx: Arc<Mutex<watch::Receiver<MonitorInfo>>>,
}

impl Monitor {
    fn new(name: String, rx: watch::Receiver<MonitorInfo>) -> Self {
        Self {
            name,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn subscription(&self) -> iced::Subscription<MonitorInfo> {
        iced::Subscription::run_with_id(
            self.name.clone(),
            iced::futures::stream::unfold(self.rx.clone(), |rx| async move {
                let value = {
                    let mut rx = rx.lock().await;
                    if rx.changed().await.is_ok() {
                        Some(rx.borrow().clone())
                    } else {
                        None
                    }
                };
                value.map(|v| (v, rx))
            }),
        )
    }
}

pub fn listen_monitors() -> mpsc::Receiver<Monitor> {
    #[cfg(feature = "hyprland")]
    {
        use hyprland::shared::HyprData;
        if hyprland::data::Version::get().is_ok() {
            return hyprland_desktop::listen_monitors();
        }
    }

    #[cfg(feature = "niri")]
    if let Ok(socket) = niri_ipc::socket::Socket::connect() {
        return niri_desktop::listen_monitors(socket);
    }

    panic!("no compatible desktop environment detected");
}
