use std::net::SocketAddr;

use clap::Parser;


#[derive(Parser, Debug, Clone)]
/// Server Configuration
pub struct Config {
    /// the socket to bind to
    #[arg(long, short = 's', default_value_t = SocketAddr::from(([127, 0, 0, 1], 3000)))]
    pub socket: SocketAddr,

    /// the directory that holds the static frontend files
    #[arg(long, short = 'f', default_value_t = String::from("./client/"))]
    pub frontend_files_dir: String,    

    /// the bare server directory
    #[arg(long, short = 'b', default_value_t = String::from("/bare/"))]
    pub bare_prefix: String,

    /// Whether Bare server websocket responses should have extra metadata
    #[arg(long, short = 'e')]
    pub extra_bare_meta: bool,

    /// whether the server should allow addresses that aren't globally reachable
    #[arg(long, short = 'g')]
    pub allow_non_global_ip: bool,

    /// the wisp server directory
    #[arg(long, short = 'w', default_value_t = String::from("/wisp/"))]
    pub wisp_prefix: String,
    
    /// the wsproxy sevrer directory
    #[arg(long, short = 'x', default_value_t = String::from("/wsproxy/"))]
    pub wsproxy_prefix: String,

    /// whether the server should allow UDP
    #[arg(long, short = 'u')]
    pub allow_udp: bool,

    /// whether the server should allow ports other than 80 or 443
    #[arg(long, short = 'p', alias = "all-ports")]
    pub allow_non_internet_ports: bool,

    /// the max message size for the websockets in bytes
    #[arg(long, short = 'm', default_value_t = 1024 * 1024)]
    pub ws_max_message_size: usize,

    /// path to a file containing `user:password` separated by newlines.
    #[arg(long, short = 'a')]
    pub auth_path: Option<String>,
}