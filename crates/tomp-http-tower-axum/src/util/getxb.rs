use std::{borrow::Cow, str::FromStr};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Uri};
use serde_json::{Map, Value};
use crate::{consts::*, err::{BareError, BareErrorCode}, structs::{BareServerInfo, BareServerVersion}, util::*};

pub fn get_x_bare_protocol(headers: &HeaderMap) -> Result<String, BareError> {
    match headers.get("x-bare-protocol")  {
        Some(value) => {
            let mut scheme = value.to_str().unwrap().to_string();
            if !VALID_PROTOCOLS.contains(&scheme.as_str()) {
                return Err(BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
					id: "request.headers.x-bare-protocol".into(),
					message: "Header was invalid".into(),
                })
            }
            scheme.pop(); // remove the ":" at the end
            Ok(scheme)
        }
        None => Err(BareError {
                code: BareErrorCode::MISSING_BARE_HEADER,
                id: "request.headers.x-bare-protocol".into(),
                message: "Header was not specified.".into(),
            })
    }
}

pub fn get_x_bare_host(headers: &HeaderMap) -> Result<String, BareError> {
    match headers.get("x-bare-host")  {
        Some(value) => {
            Ok(value.to_str().unwrap().to_string())
        }
        None => Err(BareError {
                code: BareErrorCode::MISSING_BARE_HEADER,
                id: "request.headers.x-bare-host".into(),
                message: "Header was not specified.".into(),
            })
    }
}

pub fn get_x_bare_port(headers: &HeaderMap) -> Result<u16, BareError> {
    match headers.get("x-bare-port")  {
        Some(value) => {
            value.to_str().unwrap().parse::<u16>()
            .map_err(|_| BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
					id: "request.headers.x-bare-port".into(),
					message: "Header was not a valid integer.".into(),
            })
        }
        None => Err(BareError {
            code: BareErrorCode::MISSING_BARE_HEADER,
            id: format!("request.headers.x-bare-port"),
            message: "Header was not specified.".into(),
        })
    }
}

pub fn get_x_bare_path(headers: &HeaderMap) -> Result<String, BareError> {
    match headers.get("x-bare-path")  {
        Some(value) => {
            Ok(value.to_str().unwrap().to_string())
        }
        None => Err(BareError {
            code: BareErrorCode::MISSING_BARE_HEADER,
            id: "request.headers.x-bare-path".into(),
            message: "Header was not specified.".into(),
        })
    }
}

pub fn get_x_bare_id(headers: &HeaderMap) -> Result<String, BareError> {
    match headers.get("x-bare-id")  {
        Some(value) => {
            Ok(value.to_str().unwrap().to_string())
        }
        None => Err(BareError {
                code: BareErrorCode::MISSING_BARE_HEADER,
                id: "request.headers.x-bare-id".into(),
                message: "Header was not specified.".into(),
            })
    }
}

pub fn get_x_bare_url(headers: &HeaderMap) -> Result<Uri, BareError> {
    match headers.get("x-bare-url")  {
        Some(value) => {
            Uri::from_str(value.to_str().unwrap())
                .map_err(|e| BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
                    id: "request.header.x-bare-url".into(),
                    message: format!("There was an Error while parsing the URL: {e:?}")
                })
        }
        None => Err(BareError {
                code: BareErrorCode::MISSING_BARE_HEADER,
                id: "request.headers.x-bare-id".into(),
                message: "Header was not specified.".into(),
            })
    }
}

/// Gets the X-Bare-Headers. This must always exist
pub fn get_x_bare_headers(
    headers: &HeaderMap,
    bare_info: BareServerInfo
) -> Result<HeaderMap, BareError> {
    let mut new_headers = HeaderMap::new();

    let x_bare_headers = match headers.get("x-bare-headers") {
        Some(header) => Cow::Borrowed(header),
        None => match bare_info.version {
            BareServerVersion::V1 => return Err(BareError {
                code: BareErrorCode::MISSING_BARE_HEADER,
                id: "request.headers.x-bare-headers".into(),
                message: "Header was not specified.".into(),
            }),
            BareServerVersion::V2 |
            BareServerVersion::V3 => Cow::Owned(splitjoin::join_x_bare_headers(headers)?),
        }
    };

    let headers_map = match serde_json::from_str::<Map<String, Value>>(
        x_bare_headers.to_str().unwrap()
    ).ok() {
        Some(x) => x,
        None => return Err(BareError {
                code: BareErrorCode::INVALID_BARE_HEADER,
                id: "bare.headers.x-bare-headers".to_string(),
                message: "Header was not an array of Strings.".to_string(),
            })
    };

    for (header, value) in headers_map {
        if FORBIDDEN_SEND_HEADERS.contains(&header.as_str()) {
            return Err(BareError {
                code: BareErrorCode::FORBIDDEN_BARE_HEADER,
                id: "bare.headers.x-bare-headers".to_string(),
                message: format!("A forbidden header was passed: '{header}'."),
            })
        }
        match value {
            Value::String(string) => {
                new_headers.insert(
                    HeaderName::from_str(&header).unwrap(),
                    HeaderValue::from_str(&string).unwrap()
                );
            },
            Value::Array(array) => {
                let mut header_array = Vec::new();
                for item in array {
                    if let Value::String(string) = item {
                        header_array.push(string);
                        continue;
                    } 

                    return Err(BareError {
                        code: BareErrorCode::INVALID_BARE_HEADER,
                        id: "bare.headers.x-bare-headers".into(),
                        message: "Header was not an array of Strings.".into()
                    })
                }
                new_headers.insert(
                    HeaderName::from_str(&header).unwrap(),
                    HeaderValue::from_str(&serde_json::to_string(&header_array).unwrap()).unwrap(),
                );
            }
            _ => {
                return Err(BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
					id: "bare.headers.x-bare-headers".into(),
					message: "Header was not a String or Array of Strings.".into()
                })
            }
        }
    }
    Ok(new_headers)
}

/// Gets the X-Bare-Forward-Headers. Returns an Empty HeaderMap if there is none
pub fn get_x_bare_forward_headers(
    headers: &HeaderMap,
    bare_info: BareServerInfo
) -> Result<Vec<HeaderName>, BareError> {
    let mut forward_headers = Vec::new();

    match bare_info.version {
        BareServerVersion::V1 => (),
        BareServerVersion::V2 => {
            forward_headers.extend(DEFAULT_FORWARD_HEADERS.map(|i| i.parse::<_>().unwrap()));
            if bare_info.cache {
                forward_headers.extend(DEFAULT_CACHE_FORWARD_HEADERS.map(|i| i.parse::<_>().unwrap()));
            }
        }
        BareServerVersion::V3 => {
            // Headers dropped from V2: sec-websocket-extensions, sec-websocket-key, sec-websocket-version
            forward_headers.extend(DEFAULT_FORWARD_HEADERS[..2].iter().map(|i| i.parse::<_>().unwrap()));
            if bare_info.cache {
                forward_headers.extend(DEFAULT_CACHE_FORWARD_HEADERS.map(|i| i.parse::<_>().unwrap()));
            }
        }
    }

    let x_bare_forward_headers = match headers.get("x-bare-forward-headers") {
        Some(header) => header.to_str().unwrap(),
        None => match bare_info.version {
                BareServerVersion::V1 => return Err(BareError {
                    code: BareErrorCode::MISSING_BARE_HEADER,
                    id: "request.headers.x-bare-forward-headers".into(),
                    message: "Header was not specified.".into(),
                }),
                BareServerVersion::V2 | BareServerVersion::V3 => return Ok(forward_headers)
            }
    };

    // V1 sends a JSON-serialized array, V2 and V3 don't. Never realized until now, when
    // Testing a V2 transport
    match bare_info.version {
        BareServerVersion::V1 => {
            let headers_forward_vec = match serde_json::from_str::<Vec<Value>>(
                x_bare_forward_headers
            ).ok() {
                Some(x) => x,
                None => return Err(BareError {
                        code: BareErrorCode::INVALID_BARE_HEADER,
                        id: "bare.headers.x-bare-forward-headers".to_string(),
                        message: "Header was not an array of Strings.".to_string(),
                    })
            };
        
            for value in headers_forward_vec {
                if let Value::String(header) = value {
                    if FORBIDDEN_FORWARD_HEADERS.contains(&header.as_str()) {
                        return Err(BareError {
                            code: BareErrorCode::FORBIDDEN_BARE_HEADER,
                            id: "bare.headers.x-bare-forward-headers".to_string(),
                            message: format!("A forbidden header was passed: '{header}'."),
                        })
                    }
                    forward_headers.push(header.parse::<_>().unwrap());
                }
            }
        },
        BareServerVersion::V2 | BareServerVersion::V3 => {
            // Remember to trim and filter. Lesson learned from x_bare_pass_headers
            for value in x_bare_forward_headers.trim().split(",").filter(|x| !x.is_empty()).map(str::trim) {
                if FORBIDDEN_PASS_HEADERS.contains(&value) {
                    return Err(BareError {
                        code: BareErrorCode::FORBIDDEN_BARE_HEADER,
                        id: "request.headers.x-bare-pass-headers".into(),
                        message: format!("A forbidden header was passed: '{value}'."),
                    });
                }
                forward_headers.push(HeaderName::from_str(value).unwrap());
            }        
        }
    }

    Ok(forward_headers)
}

/// Gets the X-Bare-Forward-Headers. Returns an Empty HeaderMap if there is none
/// rename to get_map_from_keys when reqwest uses hyper 1.0
pub fn get_x_bare_forward_headers_map(
    headers: &HeaderMap,
    forward_headers: &Vec<HeaderName>,
) -> HeaderMap {
    let mut forward_headers_map = HeaderMap::new();

    for header in forward_headers {
        if let Some(value) = headers.get(header) {
            forward_headers_map.insert(header.clone(), value.clone());
        }
    }

    forward_headers_map
}

/// Gets X-Bare-Pass-Headers. Since this is an optional header in all versions,
/// it returns an Empty Vec when it isn't present.
pub fn get_x_bare_pass_headers(headers: &HeaderMap, cache: bool) -> Result<Vec<HeaderName>, BareError> {
    let mut pass_headers = Vec::new();

    pass_headers.extend(DEFAULT_PASS_HEADERS.map(|x| x.parse::<HeaderName>().unwrap()));

    if cache {
        pass_headers.extend(DEFAULT_CACHE_PASS_HEADERS.map(|x| x.parse::<HeaderName>().unwrap()));
    }

    let x_bare_pass_headers = match headers.get("x-bare-pass-headers") {
        Some(header) => header.to_str().unwrap(),
        None => return Ok(pass_headers)
    };

    // Gotta trim incase it's an emptry string. The split would return the 
    // empty string and the creation of the HeaderName would error. Filter out empty strings too
    for value in x_bare_pass_headers.trim().split(",").filter(|x| !x.is_empty()).map(str::trim) {
        if FORBIDDEN_PASS_HEADERS.contains(&value) {
            return Err(BareError {
                code: BareErrorCode::FORBIDDEN_BARE_HEADER,
                id: "request.headers.x-bare-pass-headers".into(),
                message: format!("A forbidden header was passed: '{value}'."),
            });
        }
        pass_headers.push(HeaderName::from_str(value).unwrap());
    }

    Ok(pass_headers)
}

pub fn get_x_bare_pass_statuses(headermap: &HeaderMap, cache: bool) -> Result<Vec<u16>, BareError> {
    let mut status_codes = Vec::new();

    if cache {
        status_codes.push(CACHE_NOT_MODIFIED);
    }

    if let Some(x_bare_pass_status) = headermap.get("x-bare-pass-status") {
        // Make sure to trim. Lesson learned from x_bare_pass_headers
        let statuses = x_bare_pass_status.to_str().unwrap().trim().split(',').filter(|x| !x.is_empty());
        for code in statuses {
            let icode = code.trim().parse::<u16>()
                .or(Err(BareError {
                    code: BareErrorCode::INVALID_BARE_HEADER,
					id: "request.headers.x-bare-pass-status".into(),
					message: format!("Array contained non-number value '{code}."),
                }))?;
            status_codes.push(icode);
        }

    }
    Ok(status_codes)
}