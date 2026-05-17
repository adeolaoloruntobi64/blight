use std::net::SocketAddr;
use axum::{RequestExt, body::Body, extract::{ConnectInfo, Request, State, WebSocketUpgrade}, http::Response};
use crate::appstate::AppState;

mod ws;
mod req;
pub mod ws_meta;
pub mod ws_new_meta;


// #1: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-fetch-a-url-from-the-remote
// #2: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-create-a-websocket-tunnel-to-the-remote
pub async fn proxy(
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    mut request: Request<Body>
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a Bare V2 endpoint", connectinfo.0);
    let headers = request.headers().clone();
    match request.extract_parts::<WebSocketUpgrade>().await.ok() {
        Some(ws) => ws::proxy(connectinfo, appstate, ws, headers).await,
        None => req::proxy(appstate, connectinfo, headers, request).await
    }
}