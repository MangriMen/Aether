//! Whether a host is public: the address policy of every request the launcher makes to a host
//! someone else named (a plugin, a provider, a pack).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Reject everything that is not a public, routable address.
///
/// Fails closed: anything not positively known to be public is blocked. One exception: most of
/// `198.18.0.0/15` (benchmarking) passes. VPN and proxy clients in TUN mode with fake-ip DNS
/// (Clash, sing-box, …) answer every public name with an address from it and route the
/// connection by that name, so refusing it refused public hosts. Its first four addresses,
/// `198.18.0.0/30`, stay blocked: TUN clients give their own adapter one of them (mihomo and
/// Clash Verge take `198.18.0.1`), and a connection to an address of one's own interface
/// reaches every service of this machine listening on all addresses.
pub(crate) fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, d] = ip.octets();

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
        // 198.18.0.0/30 — where TUN clients put their own interface; the rest of
        // 198.18.0.0/15 (benchmarking, fake-ip) passes: see above.
        || (a == 198 && b == 18 && c == 0 && d < 4)
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

/// The instance metadata services of the clouds, which hand out the machine's credentials:
/// AWS, GCP, Azure and others at `169.254.169.254`, Alibaba at `100.100.100.200`.
const METADATA_V4: [Ipv4Addr; 2] = [
    Ipv4Addr::new(169, 254, 169, 254),
    Ipv4Addr::new(100, 100, 100, 200),
];

/// AWS's metadata service over IPv6 (`fd00:ec2::254`).
const METADATA_V6: Ipv6Addr = Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254);

/// Whether `ip` is no single host the user could mean when they let a plugin reach a local
/// address: unspecified and "this network" (which reach the local machine on some systems),
/// broadcast and multicast; or a cloud metadata service. In whatever IPv6 form carries them.
/// Even a granted host is refused such an address.
pub(crate) fn is_never_grantable(ip: IpAddr) -> bool {
    let is_never_v4 = |v4: Ipv4Addr| {
        v4.octets()[0] == 0
            || v4.is_broadcast()
            || v4.is_multicast()
            || v4.is_unspecified()
            || METADATA_V4.contains(&v4)
    };
    match ip {
        IpAddr::V4(v4) => is_never_v4(v4),
        IpAddr::V6(v6) => match embedded_ipv4(v6) {
            Some(v4) => is_never_v4(v4),
            None => v6.is_unspecified() || v6.is_multicast() || v6 == METADATA_V6,
        },
    }
}

/// `localhost` and its subdomains always mean the local machine (RFC 6761).
pub(crate) fn is_localhost(name: &str) -> bool {
    let name = name.trim_end_matches('.');
    name == "localhost" || name.ends_with(".localhost")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn blocked(ip: &str) -> bool {
        is_blocked_ip(ip.parse().unwrap())
    }

    #[rstest]
    #[case::nat64_embedded_ipv4(
        &["64:ff9b::7f00:1", "64:ff9b::10.0.0.1", "64:ff9b::169.254.169.254"],
        &["64:ff9b::8.8.8.8"]
    )]
    #[case::local_use_nat64_whole_prefix(
        &["64:ff9b:1::10.0.0.1", "64:ff9b:1::8.8.8.8", "64:ff9b:1:a00:1::"],
        &[]
    )]
    #[case::six_to_four_embedded_ipv4(
        // 127.0.0.1, 192.168.1.1, 169.254.169.254, then 8.8.8.8 allowed.
        &["2002:7f00:1::1", "2002:c0a8:101::1", "2002:a9fe:a9fe::"],
        &["2002:808:808::1"]
    )]
    #[case::site_local(&["fec0::1", "feff:ffff::1"], &[])]
    #[case::discard_only(&["100::", "100::ffff:ffff:ffff:ffff"], &[])]
    #[case::ipv4_mapped(
        &["::ffff:127.0.0.1", "::ffff:100.64.0.1"],
        &["::ffff:8.8.8.8"]
    )]
    #[case::ipv4_compatible_or_translated(
        // The last two are deprecated (never routed as IPv4) and SIIT.
        &["::", "::1", "::10.0.0.1", "::8.8.8.8", "::ffff:0:8.8.8.8"],
        &[]
    )]
    #[case::outside_global_unicast(
        &["1::1", "4000::1", "fc00::1", "fd12::1", "fe80::1", "ff02::1"],
        &[]
    )]
    #[case::special_purpose_global_unicast(
        &[
            "2001::1",
            "2001:0:4136:e378:8000:63bf:3fff:fdd2",
            "2001:2::1",
            "2001:10::1",
            "2001:20::1",
            "2001:db8::1",
            "3fff::1",
            "3ffe::1",
        ],
        &[]
    )]
    #[case::public_global_unicast(
        &[],
        &[
            "2606:4700:4700::1111",
            "2001:4860:4860::8888",
            "2a00:1450:4001::1",
            "2001:200::1",
        ]
    )]
    fn should_judge_the_ipv6_address(#[case] blocked_ips: &[&str], #[case] allowed_ips: &[&str]) {
        for ip in blocked_ips {
            assert!(blocked(ip), "{ip}");
        }
        for ip in allowed_ips {
            assert!(!blocked(ip), "{ip}");
        }
    }

    /// TUN-mode VPN clients answer public names from `198.18.0.0/15` (fake-ip): it passes in
    /// every form, while the ranges around it and CGNAT stay blocked.
    #[test]
    fn should_let_the_fake_ip_range_through_in_every_form() {
        for ip in [
            "198.18.0.4",
            "198.18.0.46",
            "198.19.255.254",
            "198.19.255.255",
            "198.17.255.255",
            "198.20.0.1",
            "::ffff:198.18.0.7",
            "64:ff9b::c612:7",
            "2002:c612:7::1",
        ] {
            assert!(!blocked(ip), "{ip}");
        }
        for ip in [
            "100.64.0.1",
            "::ffff:100.64.0.1",
            "64:ff9b::6440:1",
            "2002:6440:1::1",
            "192.0.0.1",
        ] {
            assert!(blocked(ip), "{ip}");
        }
    }

    /// TUN clients give their own adapter an address of `198.18.0.0/30` (mihomo takes
    /// `198.18.0.1`): a connection there reaches this machine.
    #[test]
    fn should_block_the_tun_interface_addresses_in_every_form() {
        for ip in [
            "198.18.0.0",
            "198.18.0.1",
            "198.18.0.2",
            "198.18.0.3",
            "::ffff:198.18.0.1",
            "64:ff9b::c612:1",
            "2002:c612:2::1",
        ] {
            assert!(blocked(ip), "{ip}");
        }
    }

    #[test]
    fn should_never_grant_a_cloud_metadata_address_in_any_form() {
        for ip in [
            "169.254.169.254",
            "100.100.100.200",
            "fd00:ec2::254",
            "::ffff:169.254.169.254",
            "64:ff9b::a9fe:a9fe",
            "2002:a9fe:a9fe::1",
            "::ffff:100.100.100.200",
        ] {
            assert!(is_never_grantable(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["169.254.169.253", "100.100.100.201", "fd00:ec2::253"] {
            assert!(!is_never_grantable(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn should_never_grant_an_address_when_it_is_no_single_host() {
        for ip in [
            "0.0.0.0",
            "0.1.2.3",
            "255.255.255.255",
            "224.0.0.1",
            "::",
            "ff02::1",
            "::ffff:0.0.0.0",
            "::ffff:224.0.0.1",
        ] {
            assert!(is_never_grantable(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "127.0.0.1",
            "192.168.1.10",
            "10.0.0.1",
            "::1",
            "fd12::1",
            "fe80::1",
        ] {
            assert!(!is_never_grantable(ip.parse().unwrap()), "{ip}");
        }
    }
}
