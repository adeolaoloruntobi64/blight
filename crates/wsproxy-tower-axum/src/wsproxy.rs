use std::{error::Error, net::SocketAddr};

use axum::{RequestExt, body::Body, extract::{ConnectInfo, OriginalUri, Path, Query, Request, State, rejection::QueryRejection}, http::{StatusCode, uri::Authority}, response::Response};
use bytes::BytesMut;
use common::ip;
use fastwebsockets::{
    upgrade::{self, UpgradeFut}, CloseCode, FragmentCollector, Frame, OpCode, Payload, WebSocketError
};
use tokio::{io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader} ,net::{TcpStream, UdpSocket}};

use crate::appstate::AppState;

pub async fn proxy(
    path: Path<String>,
    query: Result<Query<String>, QueryRejection>,
    original: OriginalUri,
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    mut request: Request<Body>,
) -> Response<Body> {
    tracing::debug!("Recieved request from {} to a wsproxy endpoint", connectinfo.0);

    match request.extract_parts::<upgrade::IncomingUpgrade>().await.ok() {
        Some(ws) => {
            let Ok((res, fut)) = ws.upgrade() else {
                return Response::builder()
                    .status(StatusCode::OK)
                    .body("Couldn't create web socket connection".into())
                    .unwrap()
            };
            tokio::spawn(wsproxy(appstate, fut, path, query.ok(), connectinfo));
            Response::from_parts(
                res.into_parts().0,
                Body::empty(),
            )
        },
        None => Response::builder()
            .status(StatusCode::OK)
            .body(format!("Bonjour, Comment ca va? Tu es a '{:?}' at {:?}", path, original).into())
            .unwrap()
    }    
}

pub struct WebSocketStreamWrapper<T: AsyncRead + AsyncWrite + Unpin>(pub FragmentCollector<T>);

pub enum WebSocketFrame {
	Data(BytesMut),
	Close,
	Ignore,
}

pub struct WsProxyClose {
    pub code: u16,
    pub reason: &'static str,
    pub err: Option<Box<dyn Error + Send + Sync>>
}

impl<T: AsyncRead + AsyncWrite + Unpin> WebSocketStreamWrapper<T> {
	pub async fn read(&mut self) -> Result<WebSocketFrame, WebSocketError> {
		let frame = self.0.read_frame().await?;
		Ok(match frame.opcode {
			OpCode::Text | OpCode::Binary => WebSocketFrame::Data(BytesMut::from(&*frame.payload)),
			OpCode::Close => WebSocketFrame::Close,
			_ => WebSocketFrame::Ignore,
		})
	}

	pub async fn write(&mut self, data: &[u8]) -> Result<(), WebSocketError> {
		self.0
			.write_frame(Frame::binary(Payload::Borrowed(data)))
			.await
	}

	pub async fn close(&mut self, code: u16, reason: &[u8]) -> Result<(), WebSocketError> {
		self.0.write_frame(Frame::close(code, reason)).await
	}
}

async fn wsproxy(
    appstate: State<AppState>,
    ws: UpgradeFut,
    path: Path<String>,
    query: Option<Query<String>>,
    connectinfo: ConnectInfo<SocketAddr>,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut ws = ws.await?;
    ws.set_max_message_size(appstate.arcedinfo.max_message_size);
    
    let ws = FragmentCollector::new(ws);
	let mut ws = WebSocketStreamWrapper(ws);
    if let Some(close) = handle_wsproxy(appstate, &mut ws, path, query, connectinfo).await {
        ws.close(close.code, close.reason.as_bytes()).await?;
        match close.err {
            Some(err) => Err(err),
            None => Ok(())
        }
    } else {
        Ok(())
    }
}

async fn handle_wsproxy<T: AsyncRead + AsyncWrite + Unpin>(
    appstate: State<AppState>,
    ws: &mut WebSocketStreamWrapper<T>,
    path: Path<String>,
    query: Option<Query<String>>,
    connectinfo: ConnectInfo<SocketAddr>,
) -> Option<WsProxyClose> {
    let authority = match Authority::try_from(path.as_str()) {
        Ok(a) => a,
        Err(e) => return Some(WsProxyClose {
            code: CloseCode::Error.into(),
            reason: "failed to parse authority",
            err: Some(e.into())
        })
    };

    // https://github.com/ading2210/libcurl.js?tab=readme-ov-file#changing-the-network-transport
    let Some(port) = authority.port_u16() else {
        return Some(WsProxyClose {
            code: CloseCode::Error.into(),
            reason: "failed to get port",
            err: Some(format!("Authority '{authority}' does not contain a port number").into())
        })
    };

    let udp = query.map(|x| x.as_str().contains("udp")).unwrap_or(false);

    if (!appstate.arcedinfo.allow_non_internet_ports && !(port == 80 || port == 443)) || (!appstate.arcedinfo.allow_non_standard_udp && udp) {
        return Some(WsProxyClose {
            code: CloseCode::Error.into(),
            reason: "blocked connection due to port or udp",
            err: Some(format!("Error allowing connection, port or udp => Port: {port}, UDP: {udp}").into())
        });
    }

    tracing::debug!("{:?}: connected (wsproxy): \"{}\"", connectinfo.0, authority);

    let dnsres = match appstate.resolver.lookup_socket_with_port(authority.host(), port).await {
        Ok(d) => d,
        Err(e) => return Some(WsProxyClose {
            code: CloseCode::Error.into(),
            reason: "failed to resolve uri",
            err: Some(e.into())
        })
    };

    let sockets = dnsres.collect::<Vec<_>>();

    if !appstate.arcedinfo.allow_non_global_ip {
        if let Some(ip) = sockets.iter().map(|x| x.ip()).find(ip::ip_is_not_global) {
            return Some(WsProxyClose {
                code: CloseCode::Error.into(),
                reason: "blocked non-global ip",
                err: Some(format!("Non-global IP {ip} found for {authority}").into())
            });
        }
    }
    let to_close = |ret, reason| match ret {
        Ok(true) => Some(WsProxyClose { code: CloseCode::Normal.into(), reason: "", err: None }),
        Ok(false) => None,
        Err(e) => Some(WsProxyClose { code: CloseCode::Error.into(), reason, err: Some(e) }),
    };

    let close = if udp {
        async {
            let stream = async {
                for socket in &sockets {
                    let bindport = if socket.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
                    let Ok(udp) = UdpSocket::bind(bindport).await else { continue };
                    let Ok(()) = udp.connect(socket).await else { continue };
                    return Some(udp);
                }
                None
            }.await;
            let Some(stream) = stream else {
                return Some(WsProxyClose {
                    code: CloseCode::Error.into(),
                    reason: "Could not connect to any of the provided sockets",
                    err: Some(format!("Could not connect to any of {sockets:?}").into()),
                });
            };
            let mut buffer = vec![0u8; 65507];
            let ret = async {
                loop {
                    tokio::select! {
                        x = ws.read() => match x? {
                            WebSocketFrame::Data(data) => {
                                let i = stream.send(&data).await?;
                                if i != data.len() {
                                    return Err(format!("Error sending packets to udp socket: # Bytes Sent: {i}, # Expected Bytes Sent: {}", data.len()).into());
                                }
                            }
                            WebSocketFrame::Close => return Ok(false),
                            WebSocketFrame::Ignore => {}
                        },
                        size = stream.recv(&mut buffer) => {
                            let size = size?;
                            if size == 0 { return Ok(true); }
                            ws.write(&buffer[..size]).await?;
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
                    code: CloseCode::Error.into(),
                    reason: "failed to connect",
                    err: Some(e.into()),
                }),
            };
            let ret = async {
                loop {
                    tokio::select! {
                        x = ws.read() => match x? {
                            WebSocketFrame::Data(data) => {
                                let i = stream.write(&data).await?;
                                if i != data.len() {
                                    return Err(format!("Error sending packets to tcp stream: # Bytes Sent: {i}, # Expected Bytes Sent: {}", data.len()).into());
                                }
                            }
                            WebSocketFrame::Close => { stream.shutdown().await?; return Ok(false); }
                            WebSocketFrame::Ignore => {}
                        },
                        x = stream.fill_buf() => {
                            let x = x?;
                            let len = x.len();
                            if len == 0 { return Ok(true); }
                            ws.write(x).await?;
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