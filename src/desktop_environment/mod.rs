use iced::futures::{SinkExt, Stream};
use tokio::sync::mpsc;

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
pub struct OutputInfo {
    pub name: String,
    pub workspaces: Vec<WorkspaceInfo>,
    pub active_workspace_idx: Option<usize>,
    pub show_transparent: bool,
}

pub fn listen() -> Option<impl Stream<Item = OutputInfo>> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let stream = iced::stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<OutputInfo>| async move {
            while let Some(output_info) = rx.recv().await {
                let _ = output.send(output_info).await;
            }
        },
    );

    #[cfg(feature = "hyprland")]
    {
        use hyprland::shared::HyprData;
        if hyprland::data::Version::get().is_ok() {
            hyprland_desktop::listen(tx);
            return Some(stream);
        }
    }

    #[cfg(feature = "niri")]
    if let Ok(socket) = niri_ipc::socket::Socket::connect() {
        niri_desktop::listen(socket, tx);
        return Some(stream);
    }

    None
}
