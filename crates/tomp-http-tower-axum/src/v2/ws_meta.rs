use std::collections::HashMap;

use axum::{body::Body, extract::State, http::{HeaderMap, Response, StatusCode}, response::IntoResponse};
use serde_json::json;

use crate::{appstate::AppState, err::{BareError, BareErrorCode}, structs::BareServerVersion, util};


pub async fn proxy(
    appstate: State<AppState>,
    headers: HeaderMap
) -> Response<Body> {
    let id = match util::getxb::get_x_bare_id(&headers) {
        Ok(id) => id,
        Err(e) => return e.into_response()
    };
    let Some(ometa) = appstate.wsstore.wsdata.get(&id).await else {
        return BareError {
            code: BareErrorCode::INVALID_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("ID '{id}' cannot be found in cache")
        }.into_response()
    };
    let Some(meta) = ometa else {
        return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' was found in a cache, but it maps to 'None'. This ID likely belongs to a V1 request")
        }.into_response()
    };
    match meta.version {
        BareServerVersion::V1 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v1 request, not a v2 request.")
        }.into_response(),
        BareServerVersion::V2 => (),
        BareServerVersion::V3 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v3 request, not a v2 request.")
        }.into_response()
    }
    let Some(wresponse) = meta.wresponse.as_ref() else {
        return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("Response metadata for ID '{id}' is not ready.")
        }.into_response();
    };

    let mut unsplit_response_headers = HashMap::<String, String>::new();
    let remote_response_headers = util::hehs::headermap_to_hashmap(&wresponse.headers);

    unsplit_response_headers.insert("x-bare-status".into(), wresponse.status.to_string());
    unsplit_response_headers.insert("x-bare-status-text".into(), wresponse.status_text.clone());
    unsplit_response_headers.insert(
        "x-bare-headers".into(),
        serde_json::to_string(&remote_response_headers).unwrap()
    );

    let unsplit_response_headers_str = serde_json::to_string(&unsplit_response_headers).unwrap();
    let possibly_split_headers = match util::splitjoin::try_split_x_bare_headers_str(
        &unsplit_response_headers_str
    ) {
        Some(split) => split,
        None => {
            HeaderMap::from_iter(
                [("x-bare-headers", &unsplit_response_headers_str)]
                    .map(|(a,b)| (a.parse::<_>().unwrap(), b.parse::<_>().unwrap()))
            )
        }
    };

    let mut response = if appstate.arcedinfo.extra_meta {
        // This isn't part of the specification. I'm just putting it here
        let mut map = HashMap::new();
        util::getxb::get_x_bare_forward_headers_map(&headers, &meta.wforward_headers, |k, v| {
            map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
        });
        let json = json!({
            "version": "v2",
            "remote": {
                "host": meta.wremote.host,
                "port": meta.wremote.port,
                "path": meta.wremote.pathquery,
                "protocol": meta.wremote.scheme + ":"
            },
            "headers": util::hehs::headermap_to_hashmap(&meta.wheaders),
            "forward_headers": map,
            "response": {
                "status": wresponse.status,
                "status_text": wresponse.status_text,
                "headers": util::hehs::headermap_to_hashmap(&wresponse.headers)
            }
        });
        Response::new(json.to_string().into())
    } else {
        Response::new(Body::empty())
    };
    *response.status_mut() = StatusCode::OK;
    *response.headers_mut() = possibly_split_headers;
    response
}