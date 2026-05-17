use std::net::SocketAddr;
use axum::{RequestExt, body::Body, extract::{ConnectInfo, OriginalUri, Request, State}, http::{Response, StatusCode}};
use fastwebsockets::upgrade;
use crate::{appstate::AppState, util, versions::WispServerVersion};

// #1: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-fetch-a-url-from-the-remote
// #2: https://github.com/tomphttp/specifications/blob/master/BareServerV1.md#request-the-server-to-create-a-websocket-tunnel-to-the-remote
pub async fn proxy(
    original: OriginalUri,
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    mut request: Request<Body>,
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a Wisp V1 endpoint", connectinfo.0);

    match request.extract_parts::<upgrade::IncomingUpgrade>().await.ok() {
        Some(ws) => {
            let Ok((res, fut)) = ws.upgrade() else {
                return Response::builder()
                    .status(StatusCode::OK)
                    .body("Couldn't create web socket connection".into())
                    .unwrap()
            };
            tokio::spawn(util::proxy(appstate, fut, connectinfo, WispServerVersion::V1));
            Response::from_parts(
                res.into_parts().0,
                Body::empty(),
            )
        },
        None => Response::builder()
            .status(StatusCode::OK)
            .body(format!("Bonjour, Comment ca va? Tu es a '{:?}' at {:?}", '/', original).into())
            .unwrap()
    }    
}