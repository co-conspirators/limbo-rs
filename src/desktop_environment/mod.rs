#[cfg(feature = "hyprland")]
mod hyprland_desktop;
#[cfg(feature = "niri")]
mod niri_desktop;

pub type WorkspaceId = i64;

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub output: Option<String>,
    pub id: WorkspaceId,
    pub idx: i32,
    pub is_active: bool,
    pub has_windows: bool,
    pub transparent_bar: bool,
}

pub type WorkspaceInfos = Vec<WorkspaceInfo>;

#[derive(Debug, Clone)]
pub enum DesktopEvent {
    WorkspacesChanged(WorkspaceInfos),
}

pub enum Desktop {
    #[cfg(feature = "hyprland")]
    Hyprland(hyprland_desktop::HyprlandDesktop),
    #[cfg(feature = "niri")]
    Niri(niri_desktop::NiriDesktop),
}
impl Desktop {
    pub fn new() -> Option<Self> {
        #[cfg(feature = "hyprland")]
        {
            use hyprland::shared::HyprData;
            if hyprland::data::Version::get().is_ok() {
                return Some(Self::Hyprland(hyprland_desktop::HyprlandDesktop::new()));
            }
        }

        #[cfg(feature = "niri")]
        if let Ok(socket) = niri_ipc::socket::Socket::connect() {
            return Some(Self::Niri(niri_desktop::NiriDesktop::new(socket)));
        }

        None
    }

    pub fn focus_workspace(&mut self, id: WorkspaceId) {
        match self {
            #[cfg(feature = "hyprland")]
            Desktop::Hyprland(hyprland_desktop) => hyprland_desktop.focus_workspace(id),
            #[cfg(feature = "niri")]
            Desktop::Niri(niri_desktop) => niri_desktop.focus_workspace(id),
            #[allow(unused)]
            _ => unreachable!(),
        }
    }

    pub fn cycle_workspace(&mut self, forward: bool) {
        match self {
            #[cfg(feature = "hyprland")]
            Desktop::Hyprland(hyprland_desktop) => hyprland_desktop.cycle_workspace(forward),
            #[cfg(feature = "niri")]
            Desktop::Niri(niri_desktop) => niri_desktop.cycle_workspace(forward),
            #[allow(unused)]
            _ => unreachable!(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<DesktopEvent> {
        match self {
            #[cfg(feature = "hyprland")]
            Desktop::Hyprland(hyprland_desktop) => hyprland_desktop.subscription(),
            #[cfg(feature = "niri")]
            Desktop::Niri(niri_desktop) => niri_desktop.subscription(),
            #[allow(unused)]
            _ => unreachable!(),
        }
    }
}
