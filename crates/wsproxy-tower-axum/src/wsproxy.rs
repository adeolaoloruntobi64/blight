use std::{error::Error, net::SocketAddr};

use axum::{body::Body, extract::{ConnectInfo, OriginalUri, Path, Query, State, rejection::QueryRejection}, http::{StatusCode, uri::Authority}, response::Response};
use common::ip::{self, UDP_BIND_IPV4, UDP_BIND_IPV6};
use futures_util::StreamExt;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader} ,net::{TcpStream, UdpSocket}};
use tokio_websockets::{CloseCode, Limits, Message};
use tokio_websockets_axum::{OptionalWebSocketUpgrade, WebSocket};

use crate::appstate::AppState;

pub async fn proxy(
    path: Path<String>,
    query: Result<Query<String>, QueryRejection>,
    original: OriginalUri,
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    ws: OptionalWebSocketUpgrade,
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a wsproxy endpoint", connectinfo.0);
    match ws.0 {
        Some(ws) => {
            ws.limits(
                    Limits::default().max_payload_len(Some(appstate.arcedinfo.max_message_size))
                ).on_upgrade(move |socket| async move {
                    let _ = wsproxy(appstate, socket, path, query.ok(), connectinfo).await;
                })
        },
        None => {
             Response::builder()
                .status(StatusCode::OK)
                .body(format!("Bonjour, Comment ca va? Tu es a '{:?}' at {:?}", path, original).into())
                .unwrap()
        }
    }
}


pub struct WsProxyClose {
    pub code: CloseCode,
    pub reason: &'static str,
    pub err: Option<Box<dyn Error + Send + Sync>>
}

async fn wsproxy(
    appstate: State<AppState>,
    mut ws: WebSocket,
    path: Path<String>,
    query: Option<Query<String>>,
    connectinfo: ConnectInfo<SocketAddr>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    match handle_wsproxy(appstate, &mut ws, path, query, connectinfo).await {
        Some(close) => {
            ws.send(Message::close(Some(close.code), close.reason)).await?;
            ws.close().await?;
            match close.err {
                Some(err) => Err(err),
                None => Ok(())
            }
        }
        None => Ok(())
    }
}

async fn handle_wsproxy(
    appstate: State<AppState>,
    ws: &mut WebSocket,
    path: Path<String>,
    query: Option<Query<String>>,
    connectinfo: ConnectInfo<SocketAddr>,
) -> Option<WsProxyClose> {
    let authority = match Authority::try_from(path.as_str()) {
        Ok(a) => a,
        Err(e) => return Some(WsProxyClose {
            code: CloseCode::INTERNAL_SERVER_ERROR,
            reason: "failed to parse authority",
            err: Some(e.into())
        })
    };

    // https://github.com/ading2210/libcurl.js?tab=readme-ov-file#changing-the-network-transport
    let Some(port) = authority.port_u16() else {
        return Some(WsProxyClose {
            code: CloseCode::INTERNAL_SERVER_ERROR,
            reason: "failed to get port",
            err: Some(format!("Authority '{authority}' does not contain a port number").into())
        })
    };

    let udp = query.map(|x| x.as_str().contains("udp")).unwrap_or(false);

    if (!appstate.arcedinfo.allow_non_internet_ports && !(port == 80 || port == 443)) || (!appstate.arcedinfo.allow_non_standard_udp && udp) {
        return Some(WsProxyClose {
            code: CloseCode::INTERNAL_SERVER_ERROR,
            reason: "blocked connection due to port or udp",
            err: Some(format!("Error allowing connection, port or udp => Port: {port}, UDP: {udp}").into())
        });
    }

    tracing::debug!("{:?}: connected (wsproxy): \"{}\"", connectinfo.0, authority);

    let dnsres = match appstate.resolver.lookup_socket_with_port(authority.host(), port).await {
        Ok(d) => d,
        Err(e) => return Some(WsProxyClose {
            code: CloseCode::INTERNAL_SERVER_ERROR,
            reason: "failed to resolve uri",
            err: Some(e.into())
        })
    };

    let sockets = dnsres.addrs;

    if !appstate.arcedinfo.allow_non_global_ip {
        if let Some(ip) = sockets.iter().map(|x| x.ip()).find(ip::ip_is_not_global) {
            return Some(WsProxyClose {
                code: CloseCode::INTERNAL_SERVER_ERROR,
                reason: "blocked non-global ip",
                err: Some(format!("Non-global IP {ip} found for {authority}").into())
            });
        }
    }
    let to_close = |ret, reason| match ret {
        Ok(true) => Some(WsProxyClose { code: CloseCode::NORMAL_CLOSURE, reason: "Closed Successfully", err: None }),
        Ok(false) => None,
        Err(e) => Some(WsProxyClose { code: CloseCode::INTERNAL_SERVER_ERROR, reason, err: Some(e) }),
    };
    let close = if udp {
        async {
            let stream = async {
                // Hypothetically, theoretically, in a parallel universe, considering the absolute worst case scenario,
                // bind() can fail per address type (work for ipv6 but not ipv4). Try each candidate until one
                // binds. connect() itself doesn't really "fail", or at least not meaningfully. It basically always
                // succeeds
                for socket in &sockets {
                    let bindport = if socket.is_ipv4() { UDP_BIND_IPV4 } else { UDP_BIND_IPV6 };
                    let Ok(udp) = UdpSocket::bind(bindport).await else { continue };
                    let Ok(()) = udp.connect(socket).await else { continue };
                    return Some(udp);
                }
                None
            }.await;
            let Some(stream) = stream else {
                return Some(WsProxyClose {
                    code: CloseCode::INTERNAL_SERVER_ERROR,
                    reason: "Could not connect to any of the provided sockets",
                    err: Some(format!("Could not connect to any of {sockets:?}").into()),
                });
            };
            // https://oneuptime.com/blog/post/2026-03-20-ipv6-udp-jumbograms/view
            // 65507 for ipv6, 65527 for ipv6
            let size = if let Ok(addr) = stream.local_addr() && addr.is_ipv4() {
                65507
            } else {
                65527
            };
            let mut buffer = vec![0u8; size];
            let ret = async {
                loop {
                    tokio::select! {
                        x = ws.next() => match x {
                            Some(Ok(msg)) => {
                                if msg.is_binary() {
                                    let data = msg.as_payload();
                                    let i = stream.send(data).await?;
                                    if i != data.len() {
                                        return Err(format!("Error sending packets to udp socket: # Bytes Sent: {i}, # Expected Bytes Sent: {}", data.len()).into());
                                    }
                                } else if msg.is_close() {
                                    return Ok(false);
                                }
                            }
                            Some(Err(e)) => return Err(e.into()),
                            None => return Ok(false),
                        },
                        size = stream.recv(&mut buffer) => {
                            let size = size?;
                            if size == 0 { return Ok(true); }
                            ws.send(Message::binary(buffer[..size].to_vec())).await?;
                        }
                    }
                }
            }.await;
            to_close(ret, "Failed to finish transferring UDP packets")
        }.await
    } else {
        async {
            let mut stream = match TcpStream::connect(sockets.as_slice()).await {
                Ok(s) => BufReader::new(s),
                Err(e) => return Some(WsProxyClose {
                    code: CloseCode::INTERNAL_SERVER_ERROR,
                    reason: "failed to connect",
                    err: Some(e.into()),
                }),
            };

            let ret = async {
                loop {
                    tokio::select! {
                        x = ws.next() => match x {
                            Some(Ok(msg)) => {
                                if msg.is_binary() {
                                    let data = msg.as_payload();
                                    let i = stream.write(data).await?;
                                    if i != data.len() {
                                        return Err(format!("Error sending packets to tcp stream: # Bytes Sent: {i}, # Expected Bytes Sent: {}", data.len()).into());
                                    }
                                } else if msg.is_close() {
                                    stream.shutdown().await?;
                                    return Ok(false);
                                }
                                // ignore ping/pong/text
                            }
                            Some(Err(e)) => return Err(e.into()),
                            None => { stream.shutdown().await?; return Ok(false); }
                        },
                        x = stream.fill_buf() => {
                            let x = x?;
                            let len = x.len();
                            if len == 0 { return Ok(true); }
                            let v = x.to_vec();
                            ws.send(Message::binary(v)).await?;
                            stream.consume(len);
                        }
                    }
                }
            }.await;
            to_close(ret, "Failed to finish transferring TCP packets")
        }.await
    };
    tracing::debug!("{:?}: disconnected (wsproxy)", connectinfo.0);
    close
}