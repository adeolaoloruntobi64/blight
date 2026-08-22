use std::net::SocketAddr;
use axum::{body::Body, extract::{ConnectInfo, OriginalUri, State}, http::{Response, StatusCode}};
use tokio_websockets_axum::OptionalWebSocketUpgrade;
use crate::{appstate::AppState, util, versions::WispServerVersion};

// #1: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-fetch-a-url-from-the-remote
// #2: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-create-a-websocket-tunnel-to-the-remote
pub async fn proxy(
    original: OriginalUri,
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    ws: OptionalWebSocketUpgrade,
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a Wisp V2 endpoint", connectinfo.0);
    match ws.0 {
        Some(ws) => {
            ws.on_upgrade(move |socket| async move {
                util::proxy(appstate, socket, connectinfo, WispServerVersion::V1).await
            })
        },
        None => {
             Response::builder()
                .status(StatusCode::OK)
                .body(format!("Bonjour, Comment ca va? Tu es a '{:?}' at {:?}", '/', original).into())
                .unwrap()
        }
    }
}