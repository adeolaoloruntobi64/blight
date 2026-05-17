use axum::{body::Body, extract::State, http::{HeaderMap, Response}, response::IntoResponse};
use serde_json::{json, Map, Value};
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
            message: format!("Metadata for ID '{id}' is not ready.")
        }.into_response();
    };
    match meta.version {
        BareServerVersion::V1 => (),
        BareServerVersion::V2 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v2 request, not a v1 request.")
        }.into_response(),
        BareServerVersion::V3 => return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("The ID '{id}' maps to a v3 request, not a v1 request.")
        }.into_response()
    }
    let Some(wresponse) = meta.wresponse.as_ref() else {
        return BareError {
            code: BareErrorCode::INVALID_BARE_HEADER,
            id: "request.headers.x-bare-id".into(),
            message: format!("Response metadata for ID '{id}' is not ready.")
        }.into_response();
    };
    
    let mut meta_map = Map::from_iter([(
        "headers".into(), serde_json::to_value(util::hehs::headermap_to_hashmap(&meta.wheaders)).unwrap()
    )]);

    // Only "headers" is required
    if appstate.arcedinfo.extra_meta {
        meta_map.insert("version".into(), "v1".into());
        meta_map.insert("remote".into(), json!({
                "host": meta.wremote.host,
                "port": meta.wremote.port,
                "path": meta.wremote.pathquery,
                "protocol": meta.wremote.scheme + ":"
        }));
        meta_map.insert(
            "forward_headers".into(), 
            serde_json::to_value(util::hehs::headermap_to_hashmap(
                &util::getxb::get_x_bare_forward_headers_map(&headers, &meta.wforward_headers)
            )).unwrap()
        );
        meta_map.insert("response".into(), json!({
            "status": wresponse.status,
            "status_text": wresponse.status_text,
            "headers": util::hehs::headermap_to_hashmap(&wresponse.headers)
        }));
    }

    appstate.wsstore.wsdata.remove(&id).await;

    Response::builder()
        .header("Content-Type", "application/json")
        .body(Value::Object(meta_map).to_string().into())
        .unwrap()
}