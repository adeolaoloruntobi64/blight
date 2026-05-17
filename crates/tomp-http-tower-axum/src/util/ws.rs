use std::net::SocketAddr;

use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::{client::IntoClientRequest, handshake::client::{Request, Response}, protocol::WebSocketConfig, Error}, Connector, MaybeTlsStream, WebSocketStream};
use crate::{consts::*, util};

pub fn random_hex_string(byte_length: usize) -> String {
    let mut bytes = vec![0u8; byte_length];
    rand::rng().fill_bytes(&mut bytes);
    
    bytes.iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<String>()
}

pub fn decode_protocol(protocol: &str) -> Result<String, String> {
	let mut result = String::new();
    let mut chars = protocol.chars();
    while let Some(char) = chars.next()  {
        if char == WEBSOCKET_PROTOCOL_RESERVED_CHAR {
            let (Some(hex1), Some(hex2)) = (chars.next(), chars.next()) else {
                return Err("Not enough chars after %".into());
            };
            let (Some(n1), Some(n2)) = (hex1.to_digit(16), hex2.to_digit(16)) else {
                return Err(format!("Couldn't convert char to hex: {hex1} or {hex2}"));
            };
            result.push(char::from_u32(n1 * 16 + n2).unwrap());
        } else if WEBSOCKET_PROTOCOL_VALID_CHARS.contains(char) {
            result.push(char);
        } else {
            return Err(format!("Invalid Char Found: {char}"))
        }
    }
	return Ok(result);
}

pub async fn handle_messages(
    mut axum_session: WebSocket,
    mut tungstenite_socket: WebSocketStream<MaybeTlsStream<TcpStream>>
) {
    loop {
        tokio::select! {
            someok_axum_session_msg = axum_session.next() => {
                match someok_axum_session_msg {
                    Some(Ok(axum_session_msg)) => {
                        let tungstenite_session_msg = util::axtu::axum_to_tungstenite(axum_session_msg);
                        let close = tungstenite_session_msg.is_close();
                        tracing::trace!("From Axum, {tungstenite_session_msg:?}");
                        let res = tungstenite_socket.send(tungstenite_session_msg).await;
                        if res.is_err() || close {
                            break;
                        }
                    },
                    e => {
                        tracing::trace!("Breaking from Axum Session: {e:?}");
                        break
                    }
                }
                
            },
            someok_tungstenite_socket_msg = tungstenite_socket.next() => {
                match someok_tungstenite_socket_msg {
                    Some(Ok(tungstenite_socket_msg)) => {
                        let close = tungstenite_socket_msg.is_close();
                        tracing::trace!("From Tungstenite, {tungstenite_socket_msg:?}");
                        let axum_socket_msg = util::axtu::tungstenite_to_axum(tungstenite_socket_msg);
                        let res = axum_session.send(axum_socket_msg).await;
                        if res.is_err() || close {
                            break;
                        }
                    },
                    e => {
                        tracing::trace!("Breaking from Tungstenite Socket: {e:?}");
                        break;
                    }
                }
                
            }
        }
    }
    let _ = tungstenite_socket.close(None).await;
    let _ = axum_session.close().await;
}

/// Connect to a given URL.
pub async fn connect_async<R>(
    request: R,
    sockets: &[SocketAddr]
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(request, None, false, sockets).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration.
/// Please refer to `connect_async()` for more details. `disable_nagle` specifies if
/// the Nagle's algorithm must be disabled, i.e. `set_nodelay(true)`. If you don't know
/// what the Nagle's algorithm is, better leave it set to `false`.
pub async fn connect_async_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    sockets: &[SocketAddr]
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_tls_with_config(request.into_client_request()?, config, disable_nagle, None, sockets).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration,
/// and a TLS connector to use. Please refer to `connect_async()` for more details.
/// `disable_nagle` specifies if the Nagle's algorithm must be disabled, i.e.
/// `set_nodelay(true)`. If you don't know what the Nagle's algorithm is, better
/// leave it to `false`.
pub async fn connect_async_tls_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    connector: Option<Connector>,
    sockets: &[SocketAddr]
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect(request.into_client_request()?, config, disable_nagle, connector, sockets).await
}

async fn connect(
    request: Request,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    connector: Option<Connector>,
    sockets: &[SocketAddr]
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error> {

    let socket = TcpStream::connect(sockets).await.map_err(Error::Io)?;

    if disable_nagle {
        socket.set_nodelay(true)?;
    }
    
    tokio_tungstenite::client_async_tls_with_config(request, socket, config, connector).await
}
