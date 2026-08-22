use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use axum::{extract::{ConnectInfo, State}, http::{self, HeaderMap, HeaderName, HeaderValue, Uri}};
use common::ip;
use futures_util::StreamExt;
use serde_json::{json, Map, Value};
use tokio::net::TcpStream;
use tokio_websockets::{ClientBuilder, CloseCode, Message};
use tokio_websockets_axum::WebSocket;
use crate::{appstate::AppState, err::{BareError, BareErrorCode}, structs::{BareRemote, BareServerVersion, WsMeta, WsResponseData}, util};

fn parse_ws(json: &Value) -> Result<WsMeta, BareError> {
    // I don't understand why r# variables are frowned upon. I prefer them much more
    // that type_, or even worse, type__. How do people manage?
    let r#type = json["type"].as_str().unwrap();
    let remote = json["remote"].as_str().unwrap();
    let wheaders = json["headers"].as_object().unwrap();
    let wforward_headers = json["forwardHeaders"].as_array().unwrap();

    assert_eq!(r#type, "connect");
    
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

    let url = Uri::from_str(remote)
        .map_err(|e| BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.header.x-bare-url".into(),
            message: format!("There was an Error while parsing the URL: {e:?}")
        })?;

    Ok(WsMeta {
        version: BareServerVersion::V3,
        wremote: BareRemote::try_from_url(&url, true)?,
        wheaders: wnew_headers,
        wforward_headers: wnew_forward_headers,
        wresponse: None,
        id: None,
    })
}

pub async fn proxy(
    mut client_ws: WebSocket,
    appstate: State<AppState>,
    headers: HeaderMap,
    ConnectInfo(connectinfo): ConnectInfo<SocketAddr>
) {
    tracing::debug!("Recieved a Websocket Upgrade from {connectinfo} to a Bare V3 endpoint");

    let wmessage = client_ws.next().await.unwrap().unwrap();
    let text = wmessage.as_text().unwrap();
    let json = serde_json::from_str::<Value>(&text).unwrap();

    let protocols = json["protocols"].as_array()
        .unwrap().into_iter().map(|val| val.as_str().unwrap())
        .collect::<Vec<_>>();

    let oprotocols_str = if !protocols.is_empty() {
        let mut protocols_str = serde_json::to_string(&protocols).unwrap();
        protocols_str.remove(0); // Remove the [
        protocols_str.pop(); // Remove the ]
        Some(protocols_str)
    } else {
        None
    };
    
    let mut parsed_ws = match parse_ws(&json) {
        Ok(parsed_ws) => parsed_ws,
        Err(_maybe_make_cache_at_v3_ws_err_later_if_i_decide_to_e) => {
            // https://docs.konghq.com/hub/kong-inc/websocket-size-limit/#for-control-frames
            // All control frames (ping, pong, and close) have a max payload size of 125 bytes
            // https://datatracker.ietf.org/doc/html/rfc6455#section-5.5
            // Currently defined opcodes for control frames include 0x8 (Close), 0x9 (Ping),
            // and 0xA (Pong) ... All control frames MUST have a payload length of 125 bytes or less
            // and MUST NOT be fragmented.
            
            // In the future, I might want to make a sort of system where the error is stored
            // in like v3/ws-err, so an ID would be generated here, and a message would be sent
            // back starting with "X-Bare-Error-ID: <random_hex(16)>" and send that
            // errs.insert(id, e.into_json())
            let _ = client_ws.send(Message::close(
                Some(CloseCode::INTERNAL_SERVER_ERROR),
                "There was a problem while parsing the connect info sent"
            )).await;
            let _ = client_ws.close().await;
            return;
        },
    };
    
    let uri = parsed_ws.wremote.to_url();

    let Some(host) = uri.host() else {
        let _ = client_ws.send(Message::close(
            Some(CloseCode::INTERNAL_SERVER_ERROR),
            "Host was not found in uri"
        )).await;
        let _ = client_ws.close().await;
        return
    };

    let Some(port) = uri.port_u16().or_else(|| match uri.scheme_str() {
        Some("wss") => Some(443),
        Some("ws") => Some(80),
        _ => None,
    }) else {
        let _ = client_ws.send(Message::close(
            Some(CloseCode::INTERNAL_SERVER_ERROR),
            "The port for the uri could not be deduced"
        )).await;
        let _ = client_ws.close().await;
        return;
    };

    let Ok(dnsres) = appstate.resolver.lookup_socket_with_port(host, port).await else {
        let _ = client_ws.send(Message::close(
            Some(CloseCode::INTERNAL_SERVER_ERROR),
            "DNS Failed to find the IP of the url"
        )).await;
        let _ = client_ws.close().await;
    return;
    };

    let sockets = dnsres.addrs;

    if let Some(ip) = sockets.iter().map(|x| x.ip()).find(ip::ip_is_not_global) {
        let _ = client_ws.send(Message::close(
            Some(CloseCode::INTERNAL_SERVER_ERROR),
            &format!("the IP of the URI's host is {ip}, which is not a global address")
        )).await;
        let _ = client_ws.close().await;
        return
   }
   
   tracing::debug!("Sockets for request {} found: {:?}", parsed_ws.wremote.host, sockets);
    
    let mut builder = Some(ClientBuilder::from_uri(uri.clone()));
    let mut insert = |key: HeaderName, value: HeaderValue| {
        let current = builder
            .take()
            .expect("builder was unexpectedly consumed");
        builder = Some(
            current
                .add_header(key, value)
                .expect("failed to add WebSocket header"),
        );
    };
    parsed_ws.wheaders.iter()
        .for_each(|(k, v)| insert(k.clone(), v.clone()));
   
   if let Some(protocols_str) = oprotocols_str {
        // Looking at firefox req headers and js docs, [] is passed if no protocol is
        // given, but if it's [], Sec-WebSocket-Protocol is not sent. Basically, if there is
        // no protocol, then don't add this header.
        // Tested with postman. wss://echo-websocket.hoppscotch.io doesn't work if Sec-WebSocket-Protocol
        // is set to nothing.
        insert(
            HeaderName::from_static("Sec-WebSocket-Protocol"),
            protocols_str.parse::<_>().unwrap()
        );
    }
    
    util::getxb::get_x_bare_forward_headers_map(
        &headers,
        &parsed_ws.wforward_headers,
        insert,
    );

    let builder = builder.expect("builder was unexpectedly consumed");
    tracing::debug!("Client {connectinfo} has requested to connect to {}", uri);

    let stream = match TcpStream::connect(sockets.as_slice()).await {
        Ok(s) => s,
        Err(e) => {
            let errstr = format!("Couldn't Connect to Remote For: {e:?}");
            tracing::trace!(errstr);
            let _ = client_ws.send(Message::close(
                Some(CloseCode::INTERNAL_SERVER_ERROR),
                &errstr
            )).await;
            let _ = client_ws.close().await;
            return;
        }
    };
    let (server_ws, remote_res) = match builder.connect_on(stream).await {
        Ok(t) => t,
        Err(e) => {
            let errstr = format!("Couldn't Communicate with the remote server: {e:?}");
            tracing::trace!(errstr);
            let _ = client_ws.send(Message::close(
                Some(CloseCode::INTERNAL_SERVER_ERROR),
                &errstr
            )).await;
            let _ = client_ws.close().await;
            return;
        }
    };

    let res_protocol = remote_res
        .headers()
        .get(http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let res_set_cookies: Vec<&str> = remote_res
        .headers()
        .get_all(http::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect();

    let wresponse = WsResponseData {
        status: remote_res.status().as_u16(),
        status_text: remote_res.status().canonical_reason().unwrap_or_default().to_string(),
        headers: remote_res.headers().clone(),
    };

    let wresponse_fmt = if appstate.arcedinfo.extra_meta {
        Some(json!({
            "status": wresponse.status,
            "status_text": wresponse.status_text,
            "headers": util::hehs::headermap_to_hashmap(&wresponse.headers)
        }))
    } else {
        None
    };

    parsed_ws.wresponse = Some(wresponse);
    let mut msg = Map::from_iter([
        ("type".into(), "open".into()),
        ("protocol".into(), res_protocol.into()),
        ("setCookies".into(), res_set_cookies.into()),
    ]);

    let mut map = HashMap::new();
    util::getxb::get_x_bare_forward_headers_map(&headers, &parsed_ws.wforward_headers, |k, v| {
        map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
    });

    if let Some(wresponse_fmt) = wresponse_fmt {
        let other_metadata = json!({
            "version": "v3",
            "remote": parsed_ws.wremote.to_url().to_string(),
            "headers": util::hehs::headermap_to_hashmap(&parsed_ws.wheaders),
            "forward_headers": map,
            "response": wresponse_fmt
        });
        msg.insert("extraMeta".into(), other_metadata.into());
    }
    
    let _ = client_ws.send(Message::text(Value::Object(msg).to_string())).await;
    util::ws::handle_messages(client_ws.inner, server_ws).await
}