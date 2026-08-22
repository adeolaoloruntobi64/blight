use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_websockets::WebSocketStream;
use crate::consts::*;

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

pub async fn handle_messages<S: AsyncRead + AsyncWrite + Unpin, T: AsyncRead + AsyncWrite + Unpin>(
    mut client_ws: WebSocketStream<S>,
    mut server_ws: WebSocketStream<T>
) {
    loop {
        tokio::select! {
            client_ws_message = client_ws.next() => match client_ws_message {
                Some(Ok(msg)) => {
                    let close = msg.is_close();
                    tracing::trace!("Bare websocket message from client to server, {msg:?}");
                    let res = server_ws.send(msg).await;
                    if res.is_err() || close {
                        break;
                    }
                },
                e => {
                    tracing::trace!("Error retrieveing client ws message: {e:?}");
                    break
                }
            },
            server_ws_message = server_ws.next() => match server_ws_message {
                Some(Ok(msg)) => {
                    let close = msg.is_close();
                    tracing::trace!("Bare websocket message from server to client, {msg:?}");
                    let res = client_ws.send(msg).await;
                    if res.is_err() || close {
                        break;
                    }
                },
                e => {
                    tracing::trace!("Error retrieveing server ws message: {e:?}");
                    break
                }
            },
        }
    }

    let _ = tokio::join!(
        client_ws.close(),
        server_ws.close()
    );
}