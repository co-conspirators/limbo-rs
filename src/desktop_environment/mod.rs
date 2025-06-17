use std::sync::{Arc, mpsc};

use tokio::sync::{Mutex, watch};

#[cfg(feature = "hyprland")]
mod hyprland_desktop;
#[cfg(feature = "niri")]
mod niri_desktop;

#[cfg(not(any(feature = "hyprland", feature = "niri")))]
compile_error!("At least one of \"hyprland\" or \"niri\" must be enabled.");

#[derive(Debug, Clone, Copy, Default)]
pub struct WorkspaceInfo {
    pub has_windows: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MonitorInfo {
    pub workspaces: Vec<WorkspaceInfo>,
    pub active_workspace_idx: Option<u8>,
    pub show_transparent: bool,
}

#[derive(Debug, Clone)]
pub enum DesktopEvent {
    Quit,
    MonitorInfoEvent(MonitorInfo),
}

#[derive(Debug)]
pub struct Monitor {
    name: String,
    rx: Arc<Mutex<watch::Receiver<DesktopEvent>>>,
}

impl Monitor {
    fn new(name: String, rx: watch::Receiver<DesktopEvent>) -> Self {
        Self {
            name,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn subscription(&self) -> iced::Subscription<DesktopEvent> {
        iced::advanced::subscription::from_recipe(watch_subscription::WatchRecipe {
            monitor: self.name.clone(),
            rx: self.rx.clone(),
        })
    }
}

pub fn listen_monitors() -> mpsc::Receiver<Monitor> {
    #[cfg(feature = "hyprland")]
    {
        use hyprland::shared::HyprData;
        if hyprland::data::Version::get().is_ok() {
            todo!();
        }
    }

    #[cfg(feature = "niri")]
    if let Ok(socket) = niri_ipc::socket::Socket::connect() {
        return niri_desktop::listen_monitors(socket);
    }

    panic!("no compatible desktop environment detected");
}

mod watch_subscription {
    use std::{any::TypeId, hash::Hash, sync::Arc};

    use iced::futures::StreamExt;
    use tokio::sync::{Mutex, watch};

    pub struct WatchRecipe<T: Clone + Send + 'static> {
        pub monitor: String,
        pub rx: Arc<Mutex<watch::Receiver<T>>>,
    }

    impl<T> iced::advanced::subscription::Recipe for WatchRecipe<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        type Output = T;

        fn hash(&self, state: &mut iced::advanced::subscription::Hasher) {
            TypeId::of::<WatchRecipe<T>>().hash(state);
            self.monitor.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced::advanced::subscription::EventStream,
        ) -> iced::advanced::graphics::futures::BoxStream<Self::Output> {
            let rx = self.rx.clone();
            async_stream::stream! {
                let mut rx = rx.lock().await;
                let initial = rx.borrow_and_update().clone();
                yield initial;

                while rx.changed().await.is_ok() {
                    let updated = rx.borrow_and_update().clone();
                    yield updated;
                }
            }
            .boxed()
        }
    }
}
