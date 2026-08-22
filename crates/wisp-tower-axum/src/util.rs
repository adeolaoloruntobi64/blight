use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{ConnectInfo, State};
use common::ip;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};
use tokio_websockets_axum::WebSocket;
use wisp_mux::extensions::cert::{CertAuthProtocolExtension, CertAuthProtocolExtensionBuilder};
use wisp_mux::extensions::motd::{MotdProtocolExtension, MotdProtocolExtensionBuilder};
use wisp_mux::extensions::udp::UdpProtocolExtension;
use wisp_mux::ws::{TokioWebsocketsTransport, TransportExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::task::JoinSet;
use wisp_mux::{
    extensions::{
        password::{PasswordProtocolExtension, PasswordProtocolExtensionBuilder},
        udp::UdpProtocolExtensionBuilder,
        AnyProtocolExtensionBuilder,
    },
    packet::{CloseReason, StreamType},
    ServerMux, WispV2Handshake,
};

use crate::{appstate::AppState, versions::WispServerVersion};

pub async fn proxy(
    appstate: State<AppState>,
    ws: WebSocket,
    connectinfo: ConnectInfo<SocketAddr>,
    version: WispServerVersion,
) {
    // Upgrade the WebSocket
    tracing::debug!("{:?}: connected", connectinfo.0);
    let (rx, tx) = TokioWebsocketsTransport(ws.inner).split_fast();
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
        if !appstate.arcedinfo.v2_use_cert.is_empty() {
            required.push(CertAuthProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(
                CertAuthProtocolExtensionBuilder::new_server(appstate.arcedinfo.v2_use_cert.clone(), true),
            ));
        }
        if let Some(motd) = &appstate.arcedinfo.v2_use_motd {
            required.push(MotdProtocolExtension::ID);
            exts.push(AnyProtocolExtensionBuilder::new(
                MotdProtocolExtensionBuilder::new_server(motd.clone()),
            ));
        }
        tracing::trace!("Created Wisp V2 handshake with the following extension ids: {required:?}");
        Some(WispV2Handshake::new(exts))
    } else {
        tracing::trace!("Client connected to Wisp V1, no handshake needed");
        None
    };

    let (mux, fut) = match ServerMux::new(rx, tx, appstate.arcedinfo.buffer_size, handshake).await {
        Ok(server) => match server.with_required_extensions(&required).await {
            Ok(res) => res,
            Err(e) => {
                tracing::debug!("Mandating requred extensions failed: {e:?}");
                return;
            },
        },
        Err(e) => {
            tracing::debug!("Creating the mux server failed: {e:?}");
            return;
        },
    };

    tracing::debug!(
        "{:?}: downgraded: {}, extensions: {:?}",
        connectinfo.0,
        mux.was_downgraded(),
        mux.get_extension_ids(),
    );

    tokio::spawn(async move {
        if let Err(e) = fut.await { tracing::debug!("Error while awaiting the mux future: {:?}", e) }
    });

    let mut set = JoinSet::new();
    while let Some((packet, stream)) = mux.wait_for_stream().await {
        tracing::trace!("New packet stream {packet:?} received from {:?}", connectinfo.0);
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
            let sockets = dnsres.addrs;

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
                    let mut muxrx = muxrx.compat();
                    let mut muxtx = muxtx.compat_write();
                    let (mut tcprx, mut tcptx) = tcp.into_split();
                    let ret = tokio::select! {
                        x = tokio::io::copy(&mut muxrx, &mut tcptx) => x,
                        x = tokio::io::copy(&mut tcprx, &mut muxtx) => x,
                    };
                    let reason = if ret.is_ok() { CloseReason::Voluntary } else { CloseReason::Unexpected };
                    return closer.close(reason).await;
                }
                StreamType::Udp => {
                    let closer = stream.get_close_handle();
                    let (mut read, write) = stream.into_split();
                    let mut write = write.into_async_write().compat_write();
                    let bindport = if sockets[0].is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
                    let udp = match UdpSocket::bind(bindport).await {
                        Ok(udp) if udp.connect(sockets.as_slice()).await.is_ok() => udp, 
                        _ => return closer.close(CloseReason::ServerStreamConnectionRefused).await,
                    };
                    // https://oneuptime.com/blog/post/2026-03-20-ipv6-udp-jumbograms/view
                    // 65507 for ipv6, 65527 for ipv6
                    let mut buf = vec![0u8; if sockets[0].is_ipv4() { 65507 } else { 65527 }];
                    let ret= async {
                        loop {
                            tokio::select! {
                                n = udp.recv(&mut buf) => {
                                    write.write_all(&buf[..n?]).await?;
                                }
                                frame = read.next() => match frame {
                                    Some(Ok(data)) => { udp.send(&data).await?; }
                                    Some(Err(e)) => return Err(Error::new(ErrorKind::Other, Box::new(e))),
                                    None => break,
                                }
                            }
                        }
                        Ok(())
                    }.await;
                    let reason = if ret.is_ok() { CloseReason::Voluntary } else { CloseReason::Unexpected };
                    return closer.close(reason).await;
                }
                _ => {
                    return stream.close(CloseReason::ServerStreamInvalidInfo).await;
                }
            }
        });
    }

    // Give it 6-7 seconds for all streams to close
    if tokio::time::timeout(Duration::from_secs(7), async {
        while set.join_next().await.is_some() {
        }
    }).await.is_err() {
        set.abort_all();
        while set.join_next().await.is_some() {}
    }
    tracing::debug!("{:?}: disconnected", connectinfo.0);
}