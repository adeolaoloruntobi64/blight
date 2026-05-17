use std::net::SocketAddr;
use axum::{body::Body, extract::{ConnectInfo, Request, State}, http::{HeaderMap, Response}};
use crate::{appstate::AppState, err::BareError, structs::{BareRemote, BareRemoteData, BareServerInfo, BareServerVersion}, util};

fn parse_req(headers: &HeaderMap, info: BareServerInfo) -> Result<BareRemoteData, BareError> {

    let url = util::getxb::get_x_bare_url(headers)?;
    // Headers can be split in V3 due to very popular webservers forbidding very long header values
    let x_bare_headers = util::getxb::get_x_bare_headers(headers, info.clone())?;
    // Optional in V3, returns returns empty containers on None
    let x_bare_forward_headers = util::getxb::get_x_bare_forward_headers(headers, info.clone()).unwrap_or_default();
    // Optional in all versions, returns empty containers on None
    let x_bare_pass_headers = util::getxb::get_x_bare_pass_headers(headers, info.cache)?;
    let x_bare_pass_statuses = util::getxb::get_x_bare_pass_statuses(headers, info.cache)?;

    Ok(BareRemoteData {
        remote: BareRemote::try_from_url(&url, false)?,
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
        BareServerVersion::V3,
        parse_req
    ).await
}