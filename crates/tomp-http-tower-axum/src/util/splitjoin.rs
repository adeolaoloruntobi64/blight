use std::str::FromStr;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use crate::{consts::*, err::{BareError, BareErrorCode}};

// for v2 and v3
// https://github.com/tomphttp/specifications/blob/master/BareServerV2.md#split-headers
// This function returns all the x-bare-headers-<y> if they're any to be made.
// None means none to be inserted
pub fn try_split_x_bare_headers_str(x_bare_headers: &str) -> Option<HeaderMap> {
    let mut new_headers = HeaderMap::new();

    if x_bare_headers.len() <= HEADER_BYTES_LIMIT {
        return None;
    }

    for (index, chunk) in x_bare_headers.as_bytes().chunks(HEADER_BYTES_LIMIT).enumerate() {
        let mut bytes = vec![';' as u8];
        bytes.extend_from_slice(chunk);
        new_headers.insert(
            HeaderName::from_str(&format!("x-bare-headers-{index}")).unwrap(),
            HeaderValue::from_bytes(bytes.as_slice()).unwrap()
        );
    }

    Some(new_headers)
}

// This converts all of the x-bare-headers-<y> into a singular header
// and returns that value
pub fn join_x_bare_headers(headers: &HeaderMap) -> Result<HeaderValue, BareError> {
    let mut i = 0;
    let mut x_bare_headers = Vec::new();

    while let Some((x_bare_header, name)) = {
        let name = format!("x-bare-headers-{i}");
        headers.get(&name).map(|x| (x, name))
    } {
        let str = x_bare_header.to_str().unwrap();
        let (semicolon, rest) = str.split_at(1);
        if semicolon != ";" {
            return Err(BareError {
                code: BareErrorCode::INVALID_BARE_HEADER,
                id: format!("request.headers.{name}"),
                message: "Value didn't begin with semi-colon.".into(),
            });
        }
        x_bare_headers.push(rest.to_string());
        i += 1;
    }
    Ok(HeaderValue::from_str(&x_bare_headers.join(",")).unwrap())
}