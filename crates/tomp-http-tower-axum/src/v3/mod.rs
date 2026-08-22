use std::net::SocketAddr;
use axum::{RequestExt, body::Body, extract::{ConnectInfo, Request, State}, http::Response, response::IntoResponse};
use tokio_websockets::Limits;
use tokio_websockets_axum::WebSocketUpgrade;
use crate::appstate::AppState;

mod ws;
mod req;

// #1: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-fetch-a-url-from-the-remote
// #2: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-create-a-websocket-tunnel-to-the-remote
pub async fn proxy(
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    mut request: Request<Body>
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a Bare V3 endpoint", connectinfo.0);
    let headers = request.headers().clone();
    match request.extract_parts::<WebSocketUpgrade>().await.ok() {
        Some(ws) => {
            ws.limits(
                Limits::default().max_payload_len(Some(appstate.arcedinfo.max_message_size))
            ).on_upgrade(
                move |session| ws::proxy(session, appstate, headers, connectinfo)
            ).into_response()
        },
        None => req::proxy(appstate, connectinfo, headers, request).await
    }
}