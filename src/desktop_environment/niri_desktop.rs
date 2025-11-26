use iced::futures::{
    StreamExt,
    stream::{once, unfold},
};
use niri_ipc::{
    Action, Request, Response,
    socket::Socket,
    state::{EventStreamState, EventStreamStatePart},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixSocket, UnixStream},
};

use super::{WorkspaceId, WorkspaceInfo};
use crate::message::Message;

pub struct NiriDesktop {
    socket: Socket,
}
impl NiriDesktop {
    pub fn new(socket: Socket) -> Self {
        Self { socket }
    }

    pub fn focus_workspace(&mut self, id: WorkspaceId) {
        let _ = self.socket.send(Request::Action(Action::FocusWorkspace {
            reference: niri_ipc::WorkspaceReferenceArg::Id(id as u64),
        }));
    }

    pub fn cycle_workspace(&mut self, forward: bool) {
        let action = if forward {
            Action::FocusWorkspaceUp {}
        } else {
            Action::FocusWorkspaceDown {}
        };
        let _ = self.socket.send(Request::Action(action));
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        #[derive(Hash)]
        struct NiriEvents;

        iced::Subscription::run_with_id(
            NiriEvents,
            once(new_event_stream())
                .filter_map(|e| async { e })
                .flat_map(|socket| {
                    unfold(
                        (socket, String::new(), EventStreamState::default()),
                        |(mut socket, mut buf, mut state)| async {
                            loop {
                                // Ignore errors.
                                // In particular, ignore Event::WindowFocusTimestampChanged, which we
                                // do not know how to deserialize since it hasn't been released yet.
                                if let Some(event) = read_event(&mut buf, &mut socket).await {
                                    state.apply(event.clone());
                                    use niri_ipc::Event::*;

                                    // Only emit messages on relevant events.
                                    if let WorkspacesChanged { .. }
                                    | WorkspaceActivated { .. }
                                    | WorkspaceActiveWindowChanged { .. }
                                    | WindowOpenedOrChanged { .. }
                                    | WindowFocusChanged { .. }
                                    | WindowClosed { .. }
                                    | OverviewOpenedOrClosed { .. } = event
                                    {
                                        break;
                                    }
                                };
                            }

                            Some((
                                Message::WorkspacesChanged(make_workspace_infos(&state)),
                                (socket, buf, state),
                            ))
                        },
                    )
                }),
        )
    }
}

async fn new_event_stream() -> Option<BufReader<UnixStream>> {
    let socket = UnixSocket::new_stream()
        .ok()?
        .connect(std::env::var(niri_ipc::socket::SOCKET_PATH_ENV).ok()?)
        .await
        .ok()?;
    let mut socket = BufReader::new(socket);

    let mut buf = serde_json::to_string(&Request::EventStream).ok()?;
    buf.push('\n');
    socket.write_all(buf.as_bytes()).await.ok()?;
    buf.clear();

    socket.read_line(&mut buf).await.ok()?;
    let reply: niri_ipc::Reply = serde_json::from_str(&buf).ok()?;
    if let Ok(Response::Handled) = reply {
        Some(socket)
    } else {
        None
    }
}

async fn read_event(
    buf: &mut String,
    socket: &mut BufReader<UnixStream>,
) -> Option<niri_ipc::Event> {
    buf.clear();
    socket.read_line(buf).await.ok()?;
    serde_json::from_str(buf).ok()
}

fn make_workspace_infos(state: &EventStreamState) -> Vec<WorkspaceInfo> {
    state
        .workspaces
        .workspaces
        .values()
        .map(|w| {
            let has_windows = state
                .windows
                .windows
                .values()
                .any(|win| win.workspace_id == Some(w.id));
            WorkspaceInfo {
                output: w.output.clone(),
                id: w.id as WorkspaceId,
                idx: w.idx as i32,
                is_active: w.is_active,
                has_windows,
                transparent_bar: !has_windows
                    || state.overview.is_open
                    || state
                        .windows
                        .windows
                        .values()
                        .filter(|win| win.workspace_id == Some(w.id))
                        .all(|win| win.is_floating),
            }
        })
        .collect()
}
