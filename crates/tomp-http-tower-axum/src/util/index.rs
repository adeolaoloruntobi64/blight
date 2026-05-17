use axum::{body::Body, extract::State, http::Response};
use memory_stats::memory_stats;
use serde_json::json;

use crate::config::ArcedBareServerInfo;

// https://github.com/tomphttp/specifications/blob/master/BareServer.md#request-server-info
#[allow(clippy::unused_async)]
pub async fn request_server_info(arcedinfo: State<ArcedBareServerInfo>) -> Response<Body> {

    // Memory Usage in megabytes
    let memory_mb = {
        let num_bytes = memory_stats().unwrap().physical_mem as f64;
        let num_megabytes = num_bytes / 1048576.0;
        // num mb to 2 decimal places
        (num_megabytes * 100.0).round() / 100.0
    };

    let json = json!({
        "versions": arcedinfo.inner.supported_versions,
        "language": "Rust",
        "memoryUsage": memory_mb,
        "maintainer": arcedinfo.inner.maintainer,
        "project": arcedinfo.inner.project
    });

    Response::builder()
        .header("Content-Type", "application/json")
        .body(json.to_string().into())
        .unwrap()
}
