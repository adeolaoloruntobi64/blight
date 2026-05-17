use std::collections::HashMap;
use axum::http::HeaderMap;

pub fn headermap_to_hashmap(headermap: &HeaderMap) -> HashMap<String, String> {
    headermap.iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
        .collect()
}