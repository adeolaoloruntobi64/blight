use axum::http::{HeaderMap, HeaderName, Uri};
use serde::{Deserialize, Serialize};

use crate::err::{BareError, BareErrorCode};

#[derive(Clone, Debug)]
pub struct BareRemote {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    // Uri fragments aren't sent back to the server,
    // so it shouldn't be named 'pathquery'
    pub pathquery: String,
}

impl BareRemote {
    pub fn to_url(&self) -> Uri {
        Uri::builder()
            .scheme(self.scheme.as_str())
            .authority(format!("{}:{}", self.host, self.port))
            .path_and_query(self.pathquery.as_str())
            .build()
            .unwrap()
    }

    pub fn try_from_url(url: &Uri, allow_ws: bool) -> Result<Self, BareError> {
        Ok(Self {
            host: url.host().unwrap().to_string(),
            port: match url.port() {
                Some(x) => x.as_u16(),
                None => match url.scheme_str().unwrap() {
                    // For the non-ws request, we don't want to allow the parsing of ws
                    "http" => 80,
                    "ws" if allow_ws => 80,
                    "https" => 443,
                    "wss" if allow_ws => 443,
                    _ => return Err(BareError {
                        code: BareErrorCode::INVALID_BARE_HEADER,
                        id: "request.header.x-bare-url".into(),
                        message: format!("The port for {url} could not be found or inferred by its scheme")
                    })
                }
            },
            pathquery: url.path_and_query().unwrap().to_string(),
            scheme: url.scheme().unwrap().to_string()
        })
    }
}

#[derive(Clone, Debug)]
pub struct BareRemoteData {
    pub remote: BareRemote,
    pub headers: HeaderMap,
    pub forward_headers: Vec<HeaderName>,
    pub pass_headers: Vec<HeaderName>,
    pub pass_statuses: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct WsResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: HeaderMap
}

#[derive(Clone, Debug)]
pub struct WsMeta {
    pub version: BareServerVersion,
    pub wremote: BareRemote,
	pub wheaders: HeaderMap,
	pub wforward_headers: Vec<HeaderName>,
    pub wresponse: Option<WsResponseData>,
    pub id: Option<String>,   
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BareServerVersion {
    #[serde(rename = "v1")]
    V1,
    #[serde(rename = "v2")]
    V2,
    #[serde(rename = "v3")]
    V3
}

#[derive(Clone, Debug)]
pub struct BareServerInfo {
    pub version: BareServerVersion,
    pub cache: bool
}