
use std::net::SocketAddr;

use axum::{body::Body, extract::{ConnectInfo, State, WebSocketUpgrade}, http::{self, HeaderMap, HeaderName, HeaderValue, Response}, response::IntoResponse};
use common::ip;
use serde_json::Value;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use crate::{appstate::AppState, err::{BareError, BareErrorCode}, structs::{BareRemote, BareServerVersion, WsMeta, WsResponseData}, util};

fn parse_ws(json: &Value) -> Result<WsMeta, BareError> {
    
    let remote_info = json["remote"].as_object().unwrap();
    let wheaders = json["headers"].as_object().unwrap();
    let wforward_headers = json["forward_headers"].as_array().unwrap();
    let wnew_headers = wheaders
        .into_iter()
        .map(|(key, value)| {
            let nkey = key.parse::<HeaderName>().unwrap();
            let nvalue = HeaderValue::from_str(value.as_str().unwrap()).unwrap();
            (nkey, nvalue)
        }).collect::<HeaderMap>();

    let wnew_forward_headers = wforward_headers
        .into_iter()
        .map(|key| key.as_str().unwrap().parse::<HeaderName>().unwrap())
        .collect::<Vec<_>>();

    let scheme = remote_info["protocol"].as_str().unwrap();
    let host = remote_info["host"].as_str().unwrap();
    let pathquery = remote_info["path"].as_str().unwrap();
    let port= remote_info["port"].as_u64().unwrap() as u16;
    let id = json.get("id")
        .map(|val| val.as_str().unwrap().to_string());
    
    let remote = BareRemote {
        scheme: {
            let mut sc = scheme.to_string();
            sc.pop(); // remove the ':' at the end
            sc
        },
        host: host.to_string(),
        port,
        pathquery: pathquery.to_string()
    };
    Ok(WsMeta {
        version: BareServerVersion::V1,
        wremote: remote.clone(),
        wheaders: wnew_headers.clone(),
        wforward_headers: wnew_forward_headers.clone(),
        wresponse: None,
        id,
    })
}

pub async fn proxy(
    ConnectInfo(connectinfo): ConnectInfo<SocketAddr>,
    appstate: State<AppState>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Response<Body> {
    tracing::debug!("Recieved a Websocket Upgrade from {connectinfo} to a Bare V1 endpoint");

    let (bare, protocol) = headers
        .get(http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
        .unwrap()
        .split_once(",")
        .unwrap();

    assert_eq!(bare, "bare");

    let decoded_protocol = util::ws::decode_protocol(protocol.trim()).unwrap();
    let json = serde_json::from_str::<Value>(&decoded_protocol).unwrap();
    let mut parsed_ws = match parse_ws(&json) {
        Ok(parsed_ws) => parsed_ws,
        Err(e) => return e.into_response(),
    };

    let uri = parsed_ws.wremote.to_url();

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
   
   tracing::debug!("Sockets for request {} found: {:?}", parsed_ws.wremote.host, sockets);

    let mut forward_request = uri.into_client_request().unwrap();
    forward_request.headers_mut().extend(parsed_ws.wheaders.clone());
    forward_request.headers_mut().extend(
        util::getxb::get_x_bare_forward_headers_map(&headers, &parsed_ws.wforward_headers)
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

    res.headers_mut().insert("Sec-WebSocket-Protocol", "bare".parse().unwrap());
    if let Some(accept) = remote_res.headers().get("Sec-WebSocket-Accept") {
        res.headers_mut().insert("Sec-WebSocket-Accept", accept.clone());
    }
    if let Some(extension) = remote_res.headers().get("Sec-WebSocket-Extensions") {
        res.headers_mut().insert("Sec-WebSocket-Extensions", extension.clone());
    }
    
    if let Some(id) = parsed_ws.id.as_ref() {
        tracing::debug!("ID '{id}' found in Request. Caching Metadata...");
        let Some(meta) = appstate.wsstore.wsdata.get(id).await else {
            tracing::debug!("Failed to Cache: ID '{id}' didn't exist");
            return BareError {
                code: BareErrorCode::INVALID_HEADER,
                id: "request.headers.x-bare-id".into(),
                message: format!("ID '{id}' was invalid: wasn't found in cache")
            }.into_response()
        };

        if let Some(imeta) = meta {
            match imeta.version {
                BareServerVersion::V1 => (),
                BareServerVersion::V2 => return BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
                    id: "request.headers.x-bare-id".into(),
                    message: format!("The ID '{id}' maps to a v2 request, not a v1 request.")
                }.into_response(),
                BareServerVersion::V3 => return BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
                    id: "request.headers.x-bare-id".into(),
                    message: format!("The ID '{id}' maps to a v3 request, not a v1 request.")
                }.into_response()
            }
        }

        parsed_ws.wresponse = Some(WsResponseData {
            status: remote_res.status().as_u16(),
            status_text: remote_res.status().canonical_reason().unwrap_or_default().to_string(),
            headers: res.headers().clone(),
        });
        
        appstate.wsstore.wsdata.insert(id.to_string(), Some(parsed_ws)).await;
    }
    return res;
}