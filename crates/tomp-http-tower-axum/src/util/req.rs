use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use axum::{body::Body, extract::{ConnectInfo, Query, Request, State}, http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode}, response::IntoResponse};
use common::ip;
use reqwest::{Request as ReqwRequest, Response as ReqwResponse, Url};
use sync_wrapper::SyncStream;
use crate::{appstate::{AppState, Requestor}, consts::{NULL_BODY_METHODS, NULL_BODY_STATUS}, dns::TompDnsResolverWrapper, err::{BareError, BareErrorCode}, structs::{BareRemoteData, BareServerInfo, BareServerVersion}, util};

pub fn req_from_bare_request(
    ConnectInfo(connectinfo): &ConnectInfo<SocketAddr>,
    headers: &HeaderMap,
    parsed_req: &BareRemoteData,
    request: Request<Body>,
) -> ReqwRequest {
    let uri = parsed_req.remote.to_url();
    let method = request.method().clone();
    let mut reqw_headers = HeaderMap::new();
    
    reqw_headers.extend(parsed_req.headers.clone());
    reqw_headers.extend(util::getxb::get_x_bare_forward_headers_map(headers, &parsed_req.forward_headers));
    
    let body = if NULL_BODY_METHODS.contains(&method.as_str()) {
        reqwest::Body::default()
    } else {
        let axum_non_sync_body_stream = request.into_body().into_data_stream();
        // Before, you could convert from axum body to reqwest body normally, but that feature
        // was removed by axum making body no longer sync, and maybe reqwest removing From<hyper::Body>.
        // This hurt my head for an hour. Thought of going back to the original version. Was blessed
        // by https://stackoverflow.com/a/78577792. I guess that is what happens when you use 0.x stuff.
        // I JUST REALIZED, FROM WHEN I HAD THIS PROBLEM IT WAS ASKED 7 DAYS AGO. IT'S JUNE 12 AND IT WAS
        // ASKED ON JUNE 4TH! Thank you, random stanger on the internet
        let sync_stream = SyncStream::new(axum_non_sync_body_stream);
        reqwest::Body::wrap_stream(sync_stream)
    };

    tracing::debug!("Creating request from {connectinfo} to send: Method: {method:?}, URL => {uri}");
    
    let mut req = ReqwRequest::new(
        method, Url::parse(&uri.to_string()).unwrap()
    );
    *req.body_mut() = Some(body);
    *req.headers_mut() = reqw_headers;
    req
}

pub async fn handle_request(
    requestor: &Requestor,
    reqw_req: ReqwRequest
) -> Result<ReqwResponse, BareError> {

    tracing::trace!("Request: {reqw_req:?}");

    match requestor.http11client.execute(reqw_req).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let status = if let Some(status) = error.status() {
                status
            } else {
                return Err(BareError {
                    code: BareErrorCode::UNKNOWN,
                    id: "response".into(),
                    message: format!("Failed to get the status of the failed response: {error}")
                })
            };
            let code = match status {
                StatusCode::NOT_FOUND => BareErrorCode::HOST_NOT_FOUND,
                StatusCode::BAD_REQUEST => BareErrorCode::CONNECTION_RESET,
                StatusCode::BAD_GATEWAY => BareErrorCode::CONNECTION_REFUSED,
                StatusCode::GATEWAY_TIMEOUT | StatusCode::REQUEST_TIMEOUT => BareErrorCode::CONNECTION_TIMEOUT,
                _ => BareErrorCode::UNKNOWN
            };
            Err(BareError {
                code,
                id: "response".into(),
                message: "Failed to get response".into()
            })
        }
    }
}

pub fn bare_response_from_res(
    parsed_req: &BareRemoteData,
    response: ReqwResponse,
    bare_info: BareServerInfo,
) -> Response<Body> {
    let mut axum_response = Response::default();
    let mut new_headers = HeaderMap::new();
    let fw_headers = util::hehs::headermap_to_hashmap(response.headers());
    let xb_headers = serde_json::to_string(&fw_headers).unwrap();

    for passh in &parsed_req.pass_headers {
        if let Some(value) = response.headers().get(passh.as_str()) {
            new_headers.insert(
                passh.clone(),
                value.clone(),
            );
        }
    }

    // cache related
    *axum_response.status_mut() = if parsed_req.pass_statuses.contains(&response.status().as_u16()) {
        response.status()
    } else {
        StatusCode::OK
    };

    let nw = &mut new_headers;
    let insert = |key: &str, value: &str, new_headers: &mut HeaderMap| {
        new_headers.insert(
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    };

    if axum_response.status() != StatusCode::NOT_MODIFIED {
        insert("x-bare-status", response.status().as_str(), nw);
        insert("x-bare-status-text", response.status().canonical_reason().unwrap_or("Unknown"), nw);

        match bare_info.version {
            BareServerVersion::V1 => insert("x-bare-headers", &xb_headers, nw),
            BareServerVersion::V2 |
            BareServerVersion::V3 => if let Some(split) = 
                util::splitjoin::try_split_x_bare_headers_str(&xb_headers) {
                    nw.extend(split);
                } else {
                    insert("x-bare-headers", &xb_headers, nw);
                }
        };
    }
    
    *axum_response.headers_mut() = new_headers; 
    *axum_response.body_mut() = if NULL_BODY_STATUS.contains(&axum_response.status().as_u16()){
        Body::empty()
    } else {
        // We don't really care about how many bytes sent
        // Also this could maybe save some memory if what we're sending is big
        // Update: Streaming should be preferred (look at stackoverflow link far above)
        Body::from_stream(response.bytes_stream())
    };
    axum_response
}

pub async fn validate_remote_ip_addr(parsed_req: &BareRemoteData, resolver: &TompDnsResolverWrapper) -> Result<(), BareError> {
    let ips = match resolver.lookup_ip(parsed_req.remote.host.as_str()).await {
        Ok(ips) => ips.iter().collect::<Vec<_>>(),
        Err(e) => return Err(BareError {
            code: BareErrorCode::UNKNOWN,
            id: "request.dns".into(),
            message: format!("DNS could not find '{}': {}", parsed_req.remote.host, e.to_string())
        })
    };

    if let Some(ip) = ips.iter().find(|ip| ip::ip_is_not_global(*ip)) {
         return Err(BareError {
            code: BareErrorCode::UNKNOWN,
            id: "request.ip".into(),
            message: format!(
                "The ip address of the remote '{}' is '{}', which is is not a global ip",
                parsed_req.remote.to_url().to_string(), ip
            )
        })
    }
    
    tracing::debug!("IP's for request {} found: {:?}", parsed_req.remote.host, ips);
    Ok(())
}

pub async fn make_request(
    appstate: State<AppState>,
    connectinfo: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
    version: BareServerVersion,
    parse_req: fn(&HeaderMap, BareServerInfo) -> Result<BareRemoteData, BareError>
) -> Response<Body> {
    let cache = Query::<HashMap<String, String>>::try_from_uri(request.uri())
        .map(|query| query.contains_key("cache"))
        .unwrap_or(false);

    let info = BareServerInfo {
        version,
        cache,
    };

    let parsed_req = match parse_req(&headers, info.clone()) {
        Ok(parsed_req) => parsed_req,
        Err(e) => return e.into_response()
    };

    if appstate.arcedinfo.block_non_global_ips {
        match validate_remote_ip_addr(&parsed_req, &appstate.resolver).await {
            Ok(()) => (),
            Err(e) => return e.into_response()
        }
    }

    let reqw_req = req_from_bare_request(
        &connectinfo, &headers, &parsed_req, request
    );

    let response = match handle_request(&appstate.requestor, reqw_req).await {
        Ok(res) => res,
        Err(e) => return e.into_response()
    };

    let axum_response = bare_response_from_res(&parsed_req, response, info);
    
    let status = axum_response.headers()
        .get("x-bare-status")
        .map(|p| p.to_str().unwrap())
        .unwrap_or("304");

    let status_text = axum_response.headers()
        .get("x-bare-status-text")
        .map(|p| p.to_str().unwrap())
        .unwrap_or("Not Modified");

    tracing::debug!(
        "Received response for {}: Status: {status} {status_text}, URL: {}",
        connectinfo.0,
        parsed_req.remote.to_url()
    );
    
    axum_response
}