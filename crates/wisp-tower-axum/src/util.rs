use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use bytes::Bytes;
use common::ip;
use fastwebsockets::{Frame, Payload};
use fastwebsockets::upgrade::UpgradeFut;
use futures::{channel::mpsc, StreamExt};
use wisp_mux::extensions::cert::{CertAuthProtocolExtension, CertAuthProtocolExtensionBuilder};
use wisp_mux::extensions::motd::{MotdProtocolExtension, MotdProtocolExtensionBuilder};
use wisp_mux::extensions::udp::UdpProtocolExtension;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use wisp_mux::{
    extensions::{
        password::{PasswordProtocolExtension, PasswordProtocolExtensionBuilder},
        udp::UdpProtocolExtensionBuilder,
        AnyProtocolExtensionBuilder,
    },
    packet::{CloseReason, StreamType},
    ServerMux, WispError, WispV2Handshake,
};

use crate::{appstate::AppState, versions::WispServerVersion};

struct FwsWrite(mpsc::UnboundedSender<Bytes>);

impl futures::Sink<Bytes> for FwsWrite {
    type Error = WispError;
    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), WispError>> {
        Poll::Ready(Ok(()))
    }
    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), WispError> {
        self.0.unbounded_send(item).map_err(|e| {
            WispError::WsImplError(Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                e.to_string(),
            )))
        })
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), WispError>> {
        Poll::Ready(Ok(()))
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), WispError>> {
        self.0.close_channel();
        Poll::Ready(Ok(()))
    }
}

pub async fn proxy(
    appstate: State<AppState>,
    ws: UpgradeFut,
    connectinfo: ConnectInfo<SocketAddr>,
    version: WispServerVersion,
) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // Upgrade the WebSocket
    let mut ws = ws.await?;
    ws.set_max_message_size(appstate.arcedinfo.max_message_size);

    tracing::debug!("{:?}: connected", connectinfo.0);
    let (ws_rx, mut ws_tx) = ws.split(tokio::io::split);
    let (write_tx, mut write_rx) = mpsc::unbounded::<Bytes>();
    let pong_tx = write_tx.clone();
    let write_transport = FwsWrite(write_tx);
    let read_transport = Box::pin(futures::stream::unfold(
        (ws_rx, pong_tx),
        |(mut rx, ptx)| async move {
            loop {
                match rx.read_frame(&mut |_frame| async { Ok::<_, std::io::Error>(()) }).await {
                    Ok(frame) => match frame.opcode {
                        fastwebsockets::OpCode::Binary => {
                            let bytes = Bytes::copy_from_slice(&frame.payload);
                            return Some((Ok(bytes), (rx, ptx)));
                        }
                        fastwebsockets::OpCode::Ping => {
                            let _ = ptx.unbounded_send(Bytes::copy_from_slice(&frame.payload));
                            continue;
                        }
                        fastwebsockets::OpCode::Close => return None,
                        _ => continue,
                    },
                    Err(e) => return Some((Err(WispError::WsImplError(Box::new(e))), (rx, ptx))),
                }
            }
        },
    ));
    tokio::spawn(async move {
        while let Some(bytes) = write_rx.next().await {
            let frame = Frame::binary(Payload::Borrowed(bytes.as_ref()),);
            if ws_tx.write_frame(frame).await.is_err() {
                break;
            }
        }
    });
    let mut required = Vec::new();
    let handshake = if version == WispServerVersion::V2 {
        let mut exts: Vec<AnyProtocolExtensionBuilder> = Vec::new();
        if appstate.arcedinfo.v2_allow_udp {
            required.push(UdpProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(UdpProtocolExtensionBuilder));
        }
        if let Some(auth) = &appstate.arcedinfo.v2_use_auth {
            required.push(PasswordProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(
                PasswordProtocolExtensionBuilder::new_server(auth.clone(), true),
            ));
        }
        if let Some(motd) = &appstate.arcedinfo.v2_use_motd {
            required.push(MotdProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(
                MotdProtocolExtensionBuilder::new_server(motd.clone()),
            ));
        }
        if !appstate.arcedinfo.v2_use_cert.is_empty() {
            required.push(CertAuthProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(
                CertAuthProtocolExtensionBuilder::new_server(appstate.arcedinfo.v2_use_cert.clone(), true),
            ));
        }
        Some(WispV2Handshake::new(exts))
    } else {
        None
    };

    let (mux, fut) = ServerMux::new(read_transport, write_transport, 1024, handshake)
        .await?
        .with_required_extensions(&required)
        .await?;

    tracing::debug!(
        "{:?}: downgraded: {}, extensions: {:?}",
        connectinfo.0,
        mux.was_downgraded(),
        mux.get_extension_ids(),
    );

    tokio::spawn(async move {
        if let Err(e) = fut.await { tracing::debug!("err in mux: {:?}", e) }
    });

    let mut set = JoinSet::new();
    while let Some((packet, stream)) = mux.wait_for_stream().await {
        let appstate = appstate.clone();
        let version = version.clone();
        
        set.spawn(async move {
            if !appstate.arcedinfo.allow_non_internet_ports
            && !(packet.port == 80 || packet.port == 443) {
                return stream.close(CloseReason::ServerStreamBlockedAddress).await;
            }
            if packet.stream_type == StreamType::Udp && !(version == WispServerVersion::V2 && appstate.arcedinfo.v2_allow_udp) {
                return stream.close(CloseReason::ServerStreamBlockedAddress).await;
            }
            if matches!(packet.stream_type, StreamType::Other(_)) {
                return stream.close(CloseReason::ServerStreamInvalidInfo).await;
            }

            let Ok(dnsres) = appstate
                .resolver
                .lookup_socket_with_port(&packet.host, packet.port)
                .await
            else {
                return stream.close(CloseReason::ServerStreamConnectionRefused).await;
            };
            let sockets = dnsres.collect::<Vec<_>>();

            if !appstate.arcedinfo.allow_non_global_ip
            && sockets.iter().map(|x| x.ip()).any(|x| ip::ip_is_not_global(&x)) {
                return stream.close(CloseReason::ServerStreamBlockedAddress).await;
            }

            match packet.stream_type {
                StreamType::Tcp => {
                    let closer = stream.get_close_handle();
                    let Ok(tcp) = TcpStream::connect(sockets.as_slice()).await else {
                        return closer.close(CloseReason::ServerStreamConnectionRefused).await;
                    };
                    
                    let (muxrx, muxtx) = stream.into_async_rw().into_split();
                    let mut muxrx = tokio_util::compat::FuturesAsyncReadCompatExt::compat(muxrx);
                    let mut muxtx = tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(muxtx);
                    
                    let (mut tcprx, mut tcptx) = tcp.into_split();
                    
                    let ret = tokio::select! {
                        x = tokio::io::copy(&mut muxrx, &mut tcptx) => x.map(|_| ()),
                        x = tokio::io::copy(&mut tcprx, &mut muxtx) => x.map(|_| ()),
                    };
                    
                    let reason = if ret.is_ok() { CloseReason::Voluntary } else { CloseReason::Unexpected };
                    return closer.close(reason).await;
                }
                StreamType::Udp => {
                    let closer = stream.get_close_handle();
                    let (mut read, write) = stream.into_split();
                    let mut write = tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(
                        write.into_async_write()
                    );
                    
                    let bindport = if sockets[0].is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
                    let Ok(udp) = tokio::net::UdpSocket::bind(bindport).await else {
                        return closer.close(CloseReason::ServerStreamConnectionRefused).await;
                    };
                    
                    if udp.connect(sockets.as_slice()).await.is_err() {
                        return closer.close(CloseReason::ServerStreamConnectionRefused).await;
                    }
                    
                    let mut buf = vec![0u8; 65507];
                    let ret: std::io::Result<()> = async {
                        loop {
                            tokio::select! {
                                n = udp.recv(&mut buf) => {
                                    use tokio::io::AsyncWriteExt;
                                    write.write_all(&buf[..n?]).await?;
                                }
                                frame = read.next() => match frame {
                                    Some(Ok(data)) => { udp.send(&data).await?; }
                                    Some(Err(e)) => return Err(std::io::Error::new(std::io::ErrorKind::Other, Box::new(e))),
                                    None => break,
                                }
                            }
                        }
                        Ok(())
                    }.await;
                    let reason = if ret.is_ok() { CloseReason::Voluntary } else { CloseReason::Unexpected };
                    return closer.close(reason).await;
                }
                _ => { return stream.close(CloseReason::ServerStreamInvalidInfo).await; }
            }
        });
    }

    // Give it 6-7 seconds for all streams to close
    if tokio::time::timeout(std::time::Duration::from_secs(7), async {
        while set.join_next().await.is_some() {}
    }).await.is_err() {
        set.abort_all();
        while set.join_next().await.is_some() {}
    }
    tracing::debug!("{:?}: disconnected", connectinfo.0);
    Ok(())
}