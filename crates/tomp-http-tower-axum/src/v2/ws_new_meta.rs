use std::collections::HashMap;

use axum::{body::Body, extract::{Query, Request, State}, http::HeaderMap, response::{IntoResponse, Response}};
use crate::{appstate::BareWSStore, err::BareError, structs::{BareRemote, BareServerInfo, BareServerVersion, WsMeta}, util};

fn parse_req(headers: &HeaderMap, cache: bool) -> Result<WsMeta, BareError> {
    let info = BareServerInfo {
        version: BareServerVersion::V2,
        cache
    };
    let scheme = util::getxb::get_x_bare_protocol(headers)?;
    let host = util::getxb::get_x_bare_host(headers)?;
    let port = util::getxb::get_x_bare_port(headers)?;
    let pathquery = util::getxb::get_x_bare_path(headers)?;
    // Headers can be split in V2 due to very popular webservers forbidding very long header values
    let x_bare_headers = util::getxb::get_x_bare_headers(headers, info.clone())?; 
    // Optional in V2, returns returns default on None
    let x_bare_forward_headers = util::getxb::get_x_bare_forward_headers(headers, info)?;

    Ok(WsMeta {
        version: BareServerVersion::V2,
        wremote: BareRemote { scheme, host, port, pathquery },
        wheaders: x_bare_headers,
        wforward_headers: x_bare_forward_headers,
        wresponse: None,
        id: None
    })
}

pub async fn proxy(
    State(BareWSStore { wsdata }): State<BareWSStore>,
    headers: HeaderMap,
    request: Request<Body>
) -> Response<Body> {
    let cache = Query::<HashMap<String, String>>::try_from_uri(request.uri())
        .map(|query| query.contains_key("cache"))
        .unwrap_or(false);

    let mut parsed_req = match parse_req(&headers, cache) {
        Ok(parsed_req) => parsed_req,
        Err(e) => return e.into_response()
    };

    let mut new_id = util::ws::random_hex_string(16);
    while wsdata.contains_key(&new_id) {
        new_id = util::ws::random_hex_string(16);
    }

    parsed_req.id = Some(new_id.clone());
    wsdata.insert(new_id.clone(), Some(parsed_req)).await;

    Response::builder()
        .header("Content-Type", " text/plain")
        .body(Body::from(new_id))
        .unwrap()
}