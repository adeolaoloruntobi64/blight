use std::net::SocketAddr;
use axum::{body::Body, extract::{ConnectInfo, Request, State}, http::HeaderMap, response::Response};
use crate::{appstate::AppState, err::BareError, structs::{BareRemote, BareRemoteData, BareServerInfo, BareServerVersion}, util};

fn parse_req(headers: &HeaderMap, info: BareServerInfo) -> Result<BareRemoteData, BareError> {

    let scheme = util::getxb::get_x_bare_protocol(headers)?;
    let host = util::getxb::get_x_bare_host(headers)?;
    let port = util::getxb::get_x_bare_port(headers)?;
    let pathquery = util::getxb::get_x_bare_path(headers)?;
    let x_bare_headers = util::getxb::get_x_bare_headers(headers, info.clone())?;
    // Mandatory in V1, returns Error on None
    let x_bare_forward_headers = util::getxb::get_x_bare_forward_headers(headers, info.clone())?;
    // Optional in all versions, returns defaults on None
    let x_bare_pass_headers = util::getxb::get_x_bare_pass_headers(headers, info.cache)?;
    let x_bare_pass_statuses = util::getxb::get_x_bare_pass_statuses(headers, info.cache)?;

    Ok(BareRemoteData {
        remote: BareRemote { scheme, host, port, pathquery },
        headers: x_bare_headers,
        forward_headers: x_bare_forward_headers,
        pass_headers: x_bare_pass_headers,
        pass_statuses: x_bare_pass_statuses,
    })
}

pub async fn proxy(
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>
) -> Response<Body> {
    util::req::make_request(
        appstate,
        connectinfo,
        headers,
        request,
        BareServerVersion::V1,
        parse_req
    ).await
}