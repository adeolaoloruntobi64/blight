use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Implementation of the unstable IpAddr::is_global
pub const fn ip_is_global(ipaddr: &IpAddr) -> bool {
    match ipaddr {
        IpAddr::V4(v4) => ipv4_is_global(v4),
        IpAddr::V6(v6) => ipv6_is_global(v6),
    }
}

/// !IpAddr::is_global
pub const fn ip_is_not_global(ipaddr: &IpAddr) -> bool {
    !ip_is_global(ipaddr)
}

/// Ipv4Addr::is_global
pub const fn ipv4_is_global(ipv4: &Ipv4Addr) -> bool {
    !(
        ipv4.octets()[0] == 0
        || ipv4.is_private()
        // ipv4.is_shared()
        || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0b1100_0000 == 0b0100_0000)) 
        || ipv4.is_loopback()
        || ipv4.is_link_local()
        || (
            ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 0
            && ipv4.octets()[3] != 9 && ipv4.octets()[3] != 10
        )
        || ipv4.is_documentation()
        // ipv4.is_benchmarking()
        || (ipv4.octets()[0] == 198 && (ipv4.octets()[1] & 0xfe) == 18)
         // ipv4.is_reserved()
        || (ipv4.octets()[0] & 240 == 240 && !ipv4.is_broadcast())
        || ipv4.is_broadcast()
    )
}

/// Ipv6Addr::is_global
pub const fn ipv6_is_global(ipv6: &Ipv6Addr) -> bool {
    !(
        ipv6.is_unspecified()
        || ipv6.is_loopback()
        || matches!(ipv6.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
        || matches!(ipv6.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
        || matches!(ipv6.segments(), [0x100, 0, 0, 0, _, _, _, _])
        || (matches!(ipv6.segments(), [0x2001, b, _, _, _, _, _, _] if b < 0x200)
            && !(
                u128::from_be_bytes(ipv6.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                || u128::from_be_bytes(ipv6.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                || matches!(ipv6.segments(), [0x2001, 3, _, _, _, _, _, _])
                || matches!(ipv6.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                || matches!(ipv6.segments(), [0x2001, b, _, _, _, _, _, _] if b >= 0x20 && b <= 0x3F)
            ))
        || matches!(ipv6.segments(), [0x2002, _, _, _, _, _, _, _])
        // ipv6.is_documentation()
        || ((ipv6.segments()[0] == 0x2001) && (ipv6.segments()[1] == 0xdb8))
        // ipv6.is_unique_local()
        || ((ipv6.segments()[0] & 0xfe00) == 0xfc00)
        // ipv6.is_unicast_link_local()
        || ((ipv6.segments()[0] & 0xffc0) == 0xfe80)
    )
}