
pub const VALID_PROTOCOLS: [&str; 4] = ["http:", "https:", "ws:", "wss:"];

pub const FORBIDDEN_SEND_HEADERS: [&str; 3] = [
	"connection",
	"content-length",
	"transfer-encoding",
];

pub const FORBIDDEN_FORWARD_HEADERS: [&str; 5]= [
	"connection",
	"transfer-encoding",
	"host",
	"origin",
	"referer",
];

pub const FORBIDDEN_PASS_HEADERS: [&str; 9] = [
	"vary",
	"connection",
	"transfer-encoding",
	"access-control-allow-headers",
	"access-control-allow-methods",
	"access-control-expose-headers",
	"access-control-max-age",
	"access-control-request-headers",
	"access-control-request-method",
];

// common defaults
/// DEFAULT_FORWARD_HEADERS[[..2]] for V3 as the SEC-* has been dropped.
/// Headers dropped from V2: sec-websocket-extensions, sec-websocket-key, sec-websocket-version
pub const DEFAULT_FORWARD_HEADERS: [&str; 5] = [
	"accept-encoding",
	"accept-language",
	"sec-websocket-extensions",
	"sec-websocket-key",
	"sec-websocket-version",
];

pub const DEFAULT_PASS_HEADERS: [&str; 3] = [
	"content-encoding",
	"content-length",
	"last-modified",
];

// defaults if the client provides a cache key
pub const DEFAULT_CACHE_FORWARD_HEADERS: [&str; 3] = [
	"if-modified-since",
	"if-none-match",
	"cache-control",
];

pub const DEFAULT_CACHE_PASS_HEADERS: [&str; 2] = ["cache-control", "etag"];

pub const CACHE_NOT_MODIFIED: u16 = 304;

pub const HEADER_BYTES_LIMIT: usize = 3072;

pub const WEBSOCKET_PROTOCOL_VALID_CHARS: &str = "!#$%&'*+-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~";

pub const WEBSOCKET_PROTOCOL_RESERVED_CHAR: char = '%';

pub const NULL_BODY_METHODS: [&str; 2] = ["GET", "HEAD"];
pub const NULL_BODY_STATUS: [u16; 4] = [101, 204, 205, 304];