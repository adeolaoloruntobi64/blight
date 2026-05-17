use axum::{body::Body, extract::State, http::Response};
use crate::{appstate::AppState, util};

pub async fn proxy(
    appstate: State<AppState>,
) -> Response<Body> {
    let mut new_id = util::ws::random_hex_string(16);
    while appstate.wsstore.wsdata.contains_key(&new_id) {
        new_id = util::ws::random_hex_string(16);
    }
    appstate.wsstore.wsdata.insert(new_id.clone(), None).await;
    Response::builder()
        .header("Content-Type", " text/plain")
        .body(Body::from(new_id))
        .unwrap()
}