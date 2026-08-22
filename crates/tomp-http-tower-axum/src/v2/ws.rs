
use std::net::SocketAddr;

use axum::{body::Body, extract::{ConnectInfo, State, WebSocketUpgrade}, http::{self, HeaderMap}, response::{IntoResponse, Response}};
use common::ip;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::{appstate::AppState, err::{BareError, BareErrorCode}, structs::{BareServerVersion, WsResponseData}, util};


#[allow(unused)]
pub async fn proxy(
    ConnectInfo(connectinfo): ConnectInfo<SocketAddr>,
    appstate: State<AppState>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Response<Body> {
    tracing::debug!("Recieved a Websocket Upgrade from {connectinfo} to a Bare V2 endpoint");

    // The specification docs as of rn are weird for this. They say Sec-WebSocket-Protocol: bare, ...,
    // But then they say Sec-WebSocket-Protocol: The protocol is the meta ID. Looking at bare-server-node
    // and a previous version of bare-client that supported V2, it is not bare, ... . It's just the ID
    let id = headers
        .get(http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
        .unwrap();
    let ometa = match appstate.wsstore.wsdata.get(id).await {
        Some(v) => v,
        None => {
            tracing::debug!("Failed to Get: ID '{id}' didn't exist");
            return BareError {
                code: BareErrorCode::INVALID_HEADER,
                id: "request.headers.x-bare-id".into(),
                message: format!("ID '{id}' wasn't found in the cache")
            }.into_response()
        }
    };
    let Some(mut meta) = ometa else {
        return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' was found in a cache, but it maps to 'None'. This ID likely belongs to a V1 request")
        }.into_response()
    };

    match meta.version {
        BareServerVersion::V1 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v1 request, not a v2 request.")
        }.into_response(),
        BareServerVersion::V2 => (),
        BareServerVersion::V3 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v3 request, not a v2 request.")
        }.into_response()
    }

    let uri = meta.wremote.to_url();

    let Some(host) = uri.host() else {
        return BareError {
            code: BareErrorCode::CONNECTION_REFUSED,
            id: "request.host".into(),
            message: format!("Host was not found in '{uri}'")
        }.into_response()
    };

    let Some(port) = uri.port_u16().or_else(|| match uri.scheme_str() {
        Some("wss") => Some(443),
        Some("ws") => Some(80),
        _ => None,
    }) else {
        return BareError {
            code: BareErrorCode::CONNECTION_REFUSED,
            id: "request.port".into(),
            message: format!("The port of '{uri}' could not be deduced")
        }.into_response()
    };

    let Ok(dnsres) = appstate.resolver.lookup_socket_with_port(host, port).await else {
        return BareError {
            code: BareErrorCode::CONNECTION_REFUSED,
            id: "request.dns".into(),
            message: format!("DNS failed to find the IP of '{uri}'")
        }.into_response()
    };

    let sockets = dnsres.addrs;

    if let Some(ip) = sockets.iter().map(|x| x.ip()).find(ip::ip_is_not_global) {
        return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.host".into(),
            message: format!("The IP of {uri} is {ip}, which is not a global address")
        }.into_response()
   }
   
   tracing::debug!("Sockets for request {} found: {:?}", meta.wremote.host, sockets);

    let mut forward_request = uri.into_client_request().unwrap();
    forward_request.headers_mut().extend(meta.wheaders.clone());
    forward_request.headers_mut().extend(
        util::getxb::get_x_bare_forward_headers_map(&headers, &meta.wforward_headers)
    );

    tracing::debug!("Client {connectinfo} has requested to connect to {}", forward_request.uri());

    let (tungstenite_socket, remote_res) = match util::ws::connect_async(forward_request, &sockets).await { 
        Ok(t) => t,
        Err(e) => {
            let mut errstr = format!("Couldn't Connect to Remote For: {e:?}");
            if let tokio_tungstenite::tungstenite::error::Error::Http(x) = e {
                let bytes = x.body().as_ref().unwrap();
                let body = std::str::from_utf8(&bytes).unwrap();
                errstr += &format!("\nResponse Err Body {body}");
            }
            tracing::trace!(errstr);
            return Response::builder()
                .status(404)
                .body(errstr.into())
                .unwrap();
        }
    };

    let mut res =  ws.on_upgrade(
        move |axum_session| util::ws::handle_messages(axum_session, tungstenite_socket)
    ).into_response();

    res.headers_mut().insert("Sec-WebSocket-Protocol", id.parse().unwrap()); 
    if let Some(accept) = remote_res.headers().get("Sec-WebSocket-Accept") {
        res.headers_mut().insert("Sec-WebSocket-Accept", accept.clone());
    }
    if let Some(extension) = remote_res.headers().get("Sec-WebSocket-Extensions") {
        res.headers_mut().insert("Sec-WebSocket-Extensions", extension.clone());
    }
    meta.wresponse = Some(WsResponseData {
        status: remote_res.status().as_u16(),
        status_text: remote_res.status().canonical_reason().unwrap_or_default().to_string(),
        headers: remote_res.headers().clone(),
    });
    appstate.wsstore.wsdata.insert(id.to_string(), Some(meta)).await;
    return res;
}


