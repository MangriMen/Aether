//! Whether a host is public: the address policy of every request the launcher makes to a host
//! someone else named (a plugin, a provider, a pack).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Reject everything that is not a public, routable address.
///
/// Fails closed: anything not positively known to be public is blocked.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();

    ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // 0.0.0.0/8 — "this network"
        || a == 0
        // 100.64.0.0/10 — carrier-grade NAT
        || (a == 100 && (64..128).contains(&b))
        // 192.0.0.0/24 — IETF protocol assignments
        || (a == 192 && b == 0 && c == 0)
        // 198.18.0.0/15 — benchmarking
        || (a == 198 && (b == 18 || b == 19))
        // 240.0.0.0/4 — reserved
        || a >= 240
}

/// An allowlist, not a denylist: only global unicast (`2000::/3`) may pass, less the ranges in it
/// that are not public, and the forms that carry an IPv4 address the connection really goes to
/// are judged by that address.
fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    // An IPv4 address wearing an IPv6 costume must be judged as IPv4.
    if let Some(v4) = embedded_ipv4(ip) {
        return is_blocked_ipv4(v4);
    }

    let s = ip.segments();

    // Everything outside 2000::/3 is not public: `::`, `::1`, the deprecated IPv4-compatible
    // `::a.b.c.d` and SIIT `::ffff:0:a.b.c.d` forms, 100::/64 (discard), the local-use NAT64
    // prefix 64:ff9b:1::/48 (where its IPv4 sits depends on a prefix length only the network
    // knows), fc00::/7 (unique local), fe80::/10 (link-local), fec0::/10 (site-local),
    // ff00::/8 (multicast) and whatever IANA has yet to assign.
    if (s[0] & 0xe000) != 0x2000 {
        return true;
    }

    // 2001::/32 — Teredo
    (s[0] == 0x2001 && s[1] == 0)
        // 2001:2::/48 — benchmarking
        || (s[0] == 0x2001 && s[1] == 0x0002 && s[2] == 0)
        // 2001:10::/28 — ORCHID, 2001:20::/28 — ORCHIDv2
        || (s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0010)
        || (s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0020)
        // 2001:db8::/32 — documentation
        || (s[0] == 0x2001 && s[1] == 0x0db8)
        // 3fff::/20 — documentation
        || (s[0] & 0xfff0) == 0x3ff0
        // 3ffe::/16 — the former 6bone, returned to IANA
        || s[0] == 0x3ffe
}

/// The IPv4 address an IPv6 address stands for, where the connection to it reaches that IPv4
/// address: `::ffff:a.b.c.d` (IPv4-mapped, which a dual-stack socket connects to over IPv4),
/// `64:ff9b::a.b.c.d` (the well-known NAT64 prefix) and `2002:AABB:CCDD::/48` (6to4).
fn embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    let v4 = |hi: u16, lo: u16| Ipv4Addr::from((u32::from(hi) << 16) | u32::from(lo));

    if let Some(v4) = ip.to_ipv4_mapped() {
        return Some(v4);
    }
    // 64:ff9b::/96 — NAT64 well-known prefix (RFC 6052)
    if s[..6] == [0x0064, 0xff9b, 0, 0, 0, 0] {
        return Some(v4(s[6], s[7]));
    }
    // 2002::/16 — 6to4 (RFC 3056)
    if s[0] == 0x2002 {
        return Some(v4(s[1], s[2]));
    }
    None
}

/// `localhost` and its subdomains always mean the local machine (RFC 6761).
pub fn is_localhost(name: &str) -> bool {
    let name = name.trim_end_matches('.');
    name == "localhost" || name.ends_with(".localhost")
}
