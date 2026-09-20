# aether-core-plugin-api

Shared DTOs and the manifest schema for the contract between Aether's core and its Extism
plugins. Both sides depend on this crate, so every change here is a change to the plugin ABI —
see `OPEN_QUESTIONS.md` Q5 for the release order (core → tag of this crate → plugins).

## Networking from a plugin

A plugin has two ways to make an HTTP request. They are **not** equally safe.

### `http_get` — the host function (preferred)

```rust
// plugin side
let response: HostResult<HttpResponseDto> = unsafe { http_get(&HttpRequestDto {
    url: "https://example.com/pack.toml".to_string(),
    headers: HashMap::new(),
}) };
```

The host performs the request itself and applies, in this order:

1. **Scheme check** — only `http` and `https`.
2. **Allowlist check in the host** — the URL's host must match the plugin's `allowed_hosts`
   (manifest entries plus whatever the user added in the plugin's settings). Matching uses the
   same glob semantics as Extism, so the two paths accept exactly the same set of hosts.
3. **Header check** — connection-control and virtual-host headers (`host`, `content-length`,
   `connection`, `transfer-encoding`, …) are rejected.
4. **Address check** — the host name is resolved and **every** returned address must be public.
   Loopback, private, link-local (including the cloud metadata address `169.254.169.254`),
   CGNAT, multicast, documentation and reserved ranges are refused, in both IPv4 and IPv6
   (including the `::ffff:a.b.c.d` form). If a name resolves to a mix of public and non-public
   addresses, the whole request is refused.
5. **Pinned connection** — the request is pinned to the addresses that were just checked, so a
   second DNS answer cannot swap in a private address between the check and the connect
   (DNS rebinding).
6. **No redirects** — a 3xx response is returned as an error, never followed.
7. **Limits** — 30 s total, 10 s to connect, 16 MiB of response body. An oversized body fails
   the call; it is never silently truncated.

The response body is a plain `Vec<u8>`; interpreting it (JSON, TOML, …) is the plugin's job.
Repeated response headers collapse to the last value.

### `http_request` — Extism's built-in (fallback)

Still available for simple cases, but **it follows redirects**, and its `allowed_hosts` check
applies only to the *initial* URL. A permitted host that answers `302` can therefore send the
plugin to any other host, including internal addresses — the host has no say after the first
hop. Treat `http_request` as usable only against endpoints you already trust not to redirect,
and prefer `http_get` for anything that reaches a third-party or user-configurable URL.
