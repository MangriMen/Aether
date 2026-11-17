# aether-core-plugin-api

Shared DTOs and the manifest schema for the contract between Aether's core and its Extism
plugins. Both sides depend on this crate, so every change here is a change to the plugin ABI.
Release order: core → tag of this crate → plugins.

## Declaring host functions — `api.hostFunctions`

List every host function the plugin imports, by its import name:

```json
"api": {
  "version": "0.5.0",
  "hostFunctions": ["log", "get_id", "http_get", "write_instance_file"]
}
```

The host links **only** these into the plugin:

* a name the host does not offer → the plugin is not enabled and is marked incompatible
  ("declares host functions this launcher does not provide: `…`"); usually it was built for a
  newer launcher. Force-enabling does not get around this — it skips only the API version check;
* a function the wasm imports but the list leaves out → the plugin fails to load ("imports host
  function `…`, but its manifest does not declare it in `api.hostFunctions`"). Every import has
  to be satisfied when the module is instantiated, so this shows up on enable, not on the first
  call;
* a declared function the wasm never imports is harmless;
* `[]` declares that the plugin calls no host function at all.

**The field is required.** A manifest without it — a plugin built before the field existed —
is marked incompatible ("does not declare `api.hostFunctions`, which this launcher requires"),
and force-enabling does not get around that either: rebuild the plugin against the current
version of this crate.

Names are matched exactly (case included), without the `extism:host/user` namespace. Extism's
own built-ins (`http_request`, `config_get`, `var_get`, …) live in a different namespace and are
not covered by the list.

With an Extism Rust PDK, the list is the set of functions in your `#[host_fn] extern
"ExtismHost"` blocks that the plugin actually calls.

## Restarts lose the plugin's memory

The host restarts a plugin to apply changed settings — folders are mounted only when an instance
is built, so granting a folder means a new instance. A restart drops the old wasm instance with
everything in its memory: statics, caches, anything kept between calls. The new instance gets
the `Loaded` event like on any start, after the old one got `Unloaded`. Keep nothing in memory
that cannot be rebuilt; what must survive belongs on disk (`/cache`, the instance through
`write_instance_file`).

A restart is refused while a call into the plugin is under way or waiting, so it never cuts a
call short.

## `run_command` output

`OutputDto` carries `stdout` and `stderr` as raw bytes, each capped at 1 MiB. A stream that hit
the cap is cut there and flagged — `stdoutTruncated` / `stderrTruncated` — with nothing added
to the bytes themselves.

A command that exits with a non-zero code is an error, not an `OutputDto`: the call fails with
`HostError::CoreError("Command execution failed")`, and the command's `stderr` is only written
to the launcher's log. So `status` of a returned `OutputDto` is always `0`: the code of a command
that succeeded. A command that runs past 15 minutes is killed and fails the call too.

## `run_command` needs the user's consent

Besides the program and folder checks, the user is asked before a command runs: *allow once*,
*always*, or *deny*. The question shows the Java launcher, the working directory, the jar and the
arguments; an unanswered question is refused after 5 minutes. While it waits, the call into the
plugin is still running, so the plugin is busy for that long (a stop or restart cancels the
question first).

A refused command fails the call with `HostError::PluginError(CommandNotApproved { plugin_id,
reason })` where `reason` is one of `denied`, `timedOut`, `cancelled` or `jarChanged` (on the
wire: `{"type": "pluginError", "data": {"type": "commandNotApproved", "data": {"pluginId": …,
"reason": "denied"}}}`). *Deny* is remembered until the plugin is stopped or reloaded: the same
command is refused again without a new question.

To let the user see why, report the error as a failure envelope (see "Reporting a typed failure"
below), passing the `HostError` as `host`: the launcher then tells the user that the command was
not allowed, and why. A plugin that turns the error into plain text still fails, but the user
only learns that the plugin reported an error. When a plugin calls another through the host and
the other one reported a refusal this way, the refusal passes through as
`PluginCallErrorDto::CommandNotApproved { reason, details }` inside `FunctionCallFailed`, and a
plugin may pass that on the same way.

*Always* is offered only for `java [-Xmx/-Xms/-Xss…] -jar #/cache/<file>.jar <args>`: the jar must be
a file in the plugin's `/cache` whose manifest has no `Class-Path`, and every option before `-jar`
must be a memory size, `-Dfile.encoding=…` or `-Djava.awt.headless=…`. What is remembered is the jar's
content (sha256) and those options, not the arguments after the jar, so the user trusts the tool
and the plugin's choice of arguments; a new build of the plugin starts over. Anything else
(`@argfile`, `-javaagent`, `-cp`, a jar outside `/cache`) can be allowed once only.

## Wire format

Every structured value that crosses the boundary, in both directions, is **msgpack with structs
written as maps keyed by field name (camelCase)**; byte fields (`Vec<u8>`) are msgpack `bin`, and enums have
their JSON shape: a plain enum is its value as a string, an enum with data is `{"type": …, "data": …}`
(`data` absent for a variant without any), and a result is `{"status": …, "data": …}`. The values
and tags are `camelCase` (`dataPack`, `resourcePack`, `shaderPack`, `timedOut`), with one exception: the loader is `neoforge`, not
`neoForge`. A hash is `{"algorithm": "sha1", "hex": …}`. Arrays are not part
of the contract: they make the field order carry the meaning, and an adjacently tagged enum such
as `HostError` cannot be read back from one. A reader ignores a key it does not know and gives a
missing key its default when the DTO has one, so a field can be added without breaking the other
side.

Only four plain enums the launcher sends have a fallback: `ModLoaderDto`, `ContentTypeDto`,
`CommandRefusalDto` and `InstanceInstallStageDto`. A value a plugin built for an older API does
not know reads as `unknown` there, so the call still decodes; a plugin must not answer `unknown`,
the launcher refuses it. Every other enum is strict (an enum with data, `{"type", "data"}`, cannot
have a fallback at all, and `HashAlgorithmDto` and the host error reasons have none): a new
value or variant of it breaks a plugin built for an older API, and `api.version` is matched as a
caret requirement, so on 0.x it needs a new minor version.

A host function returns `{"status": "ok", "data": <value>}` or `{"status": "err", "data":
<HostError>}`; `data` is always there (`nil` for `()`). The arguments and answers of a plugin's exports are the DTOs as maps.

Every host function takes one structured argument and answers a `{"status", "data"}` result, except
`get_id`, whose answer is the plugin's id as a raw UTF-8 string (not wrapped in a result). The
host functions, with the properties of `schemas/plugin-wire.schema.json` that describe their
argument and the `data` of an `ok` answer:

| Host function | Argument | `data` |
| --- | --- | --- |
| `log` | `log` | none |
| `run_command` | `command` | `output` |
| `get_java`, `install_java` | `javaVersion` | `java` |
| `instance_create` | `newInstance` | `instanceCreated` |
| `instance_get_dir`, `instance_plugin_get_dir` | `instanceId` | `instanceDir` |
| `list_content` | `instanceId` | `listContentResult` |
| `enable_contents`, `disable_contents` | `changeContents` | none |
| `write_instance_file` | `writeInstanceFile` | none |
| `read_instance_file` | `readInstanceFile` | `instanceFileBytes` |
| `http_get` | `httpRequest` | `httpResponse` |
| `http_post_json` | `httpPostJsonRequest` | `httpResponse` |
| `download_to_cache` | `downloadToCache` | `cachedFile` |

What the host sends to a plugin's exports and what it expects back (the names of the exports are
the `handler` values of the capabilities; the properties are those of `schemas/plugin-wire.schema.json`):

| Export | Input | Answer |
| --- | --- | --- |
| an event handler | `internalEvent` | none |
| an importer | `importInstance` (`{importerId, source}`; `source` is `{"type": "url", "data": {"url": …}}` or `{"type": "file", "data": {"path": …}}`) | none |
| an updater | `updateInstanceParams` | none |
| `search` | `contentSearchParams` | `contentSearchResult` |
| `getContent` | `getContentParams` | `contentItem` |
| `listVersion` | `listVersionsParams` | `listVersionsResult` |
| `getVersion` | `getVersionParams` | `contentVersion` |
| `resolveFile` | `resolveFileParams` | `resolvedFile` |
| `resolveInstallPlan` | `resolveInstallPlanParams` | `installPlan` |
| `resolvePackMetadata` | `packSource` | `packMetadata` |
| `getContentsMetadata` | `getContentsMetadataParams` | `getContentsMetadataResult` |
| `getVersionsMetadata` | `getVersionsMetadataParams` | `getVersionsMetadataResult` |
| `getAuthors` | `getAuthorsParams` | `getAuthorsResult` |
| `checkCompatibility` | `checkCompatibilityParams` | `checkCompatibilityResult` |

[`schemas/plugin-wire.schema.json`](src/v0/schemas/plugin-wire.schema.json) describes every value
that crosses the boundary (one property per type, the types under `$defs`; a `bin` is a string
with `x-msgpack: "bin"`, since JSON has none). The schema leaves unknown keys open on purpose; a
test checks that what the codec writes is listed in it.

**With Rust**, `aether_core_plugin_api::v0::wire` is the codec and `aether-core-plugin-pdk` has
`Wire<T>` for Extism in place of `Msgpack<T>`.

## Reporting a typed failure (any language)

A plugin call that fails reaches the host as text. To let the host tell *why* — a refused command
is the case today — the plugin sets that text to a failure envelope:

```
aether-plugin-failure:{"message":"...","host":{...}}
```

The marker, directly followed by one line of JSON that follows
[`schemas/plugin-failure.schema.json`](src/v0/schemas/plugin-failure.schema.json), is the
whole string given to `error_set`; return a non-zero code after it. The host finds the marker
anywhere in the string, so a prefix a framework adds is fine, and it ignores what follows the
JSON. A string without a valid envelope is an ordinary failure with that text as its cause.

`host` is the `HostError` a host function returned to the plugin, passed back untouched.

**With Rust**, use `aether-core-plugin-pdk`. Declare a host function as returning
`HostReply<T>` and read it with `.into_result()?` inside a function that returns
`aether_core_plugin_pdk::Result`. `#[plugin_fn]` itself must return `FnResult`, so call that
function from it with `?`, which turns the error into the envelope. The crate documentation has
an example.

**In another language**, do the steps by hand:

1. Decode the msgpack response of the host function (see "Wire format"): `{"status": "ok",
   "data": <value>}` or `{"status": "err", "data": <HostError>}`.
2. On `err`, build `{"message": <any text>, "host": <that HostError>}`, prefix it with the
   marker and call `error_set` with it.
3. Return a non-zero code.

What the host makes of `host` is described where the error is documented: for `run_command`, see
"`run_command` needs the user's consent" above.

## Ids

`id` in the manifest (and every capability id and platform name) must match
`[a-z0-9][a-z0-9_-]*`: lowercase ASCII letters, digits, `_` and `-`, not starting with `_` or
`-`. The plugin id names the plugin's directory, and `:` and the provider separator are left out
so that a provider id always parses back into the same plugin and capability. A manifest with
another id does not load. `aether` is reserved for the providers built into the launcher.

## Folders in the sandbox

Besides the two mounts every plugin gets (`/cache`, read-write; `/instances`, read-only), a
plugin sees the folders the user allowed for it in its settings, each under a path of its own
in the sandbox (`/mnt/d/packs`).

A mount point is `/` and one or more plain `/`-separated names: no empty, `.` or `..` part, no
`\` or NUL. It must not be `/` itself nor overlap `/cache` or `/instances`. The host side must
be an absolute path to an existing folder that is not a whole drive and does not overlap the
folders the launcher keeps for itself (instances, plugins, caches, game files, settings).

A folder that breaks these rules is **not mounted**. The launcher does not delete it from the
settings file: it moves it to `ignoredPaths` with the reason, shows it in the plugin's settings
with a warning, and the user can edit or remove it. Check what is mounted by trying to open the
path; a plugin cannot tell an ignored folder from one that was never granted.

## Networking from a plugin

A plugin has two ways to make an HTTP request. They are **not** equally safe.

### `http_get` — the host function (preferred)

```rust
// plugin side
let response: HostReply<HttpResponseDto> = unsafe { http_get(Wire(HttpRequestDto {
    url: "https://example.com/pack.toml".to_string(),
    headers: HashMap::new(),
})) }?;
```

The host performs the request itself and applies, in this order:

1. **Scheme check** — only `http` and `https`.
2. **Allowlist check in the host** — the URL's host must match the plugin's `allowedHosts`
   (manifest entries plus whatever the user added in the plugin's settings). Matching uses
   glob semantics.
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
6. **Checked redirects** — `301`, `302`, `303`, `307` and `308` are followed, **at most 5**
   times; a sixth redirect fails the call ("Too many redirects: more than 5"). Every hop goes
   through steps 1, 2, 4 and 5 again **before** anything is sent to the new host: the target must be
   `http`/`https`, its host must be in `allowedHosts`, all its addresses must be public, and
   the connection is pinned to them. On top of that:
   * `https` → `http` is refused ("Redirect from `…` to `…` would downgrade https to http");
   * a relative `Location` is resolved against the URL that answered; a missing or
     unparseable one fails the call;
   * your headers are sent only while the chain stays on the **origin** (scheme, host and
     port) you asked for. Once a hop leaves it, they are dropped for the rest of the chain —
     even if a later hop comes back — so an API key never reaches a CDN or a third party;
   * the redirect's own body is never read; `status`, `headers` and `body` describe the final
     response. Any other 3xx (`300`, `304`, …) is an error, not a response.

   These two failures have no kind of their own: like every error of `http_get`,
   `http_post_json` and `download_to_cache` that the host raises itself (a refused host, address
   or header, an oversized body, …) they arrive as `HostError::CoreError(String)`, the text being
   all there is to match on.

   Downloads from GitHub releases need this: `/releases/latest/download/<file>` answers `302`
   to `/releases/download/<tag>/<file>`, which answers `302` to a signed, expiring URL on
   `release-assets.githubusercontent.com`. Put **both** `github.com` and
   `release-assets.githubusercontent.com` in `allowedHosts`; without the second, the call fails
   at that hop with "not listed in the plugin's allowedHosts".
7. **Limits** — 30 s total for the whole chain of hops, 10 s to connect, 16 MiB of response
   body. An oversized body fails the call; it is never silently truncated.

**Rule for plugin authors: use `https://`, and do not expect headers to cross origins.** A key
sent to an `http://` URL is exposed on the very first request, and your headers are never
carried to another origin (scheme, host or port) — not even `http://a` → `https://a`.

**Behind a proxy.** If the user has a proxy configured — the proxy environment variables
(`HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY`, `NO_PROXY`) or, failing those, the OS settings, the
same source the launcher's own requests use — the host functions go through it. The proxy
then resolves host names and connects itself, so steps 4 and 5 cannot apply to a host *name*:
it is passed to the proxy unresolved. Everything else stays: the scheme check, `allowedHosts`
on every hop, the redirect limit and rules, the header drop, and the address check of a host
given as an IP literal. The proxy's own address is not filtered — the user set it up, not the
plugin. The launcher logs once that plugin requests go through the proxy.

The response body is a `Vec<u8>` sent as a msgpack `bin` (so a plugin decodes it with a copy, not
an instruction per byte — a multi-megabyte body would otherwise use up the plugin's fuel); the same
holds for the bytes of `write_instance_file`/`read_instance_file` and for the streams of
`run_command`. Interpreting the body (JSON, TOML, …) is the plugin's job. For anything large use
`download_to_cache`, which keeps the body out of the plugin's memory.
Repeated response headers collapse to the last value.

### `http_post_json` — JSON POST

`http_post_json(HttpPostJsonRequestDto { url, headers, body }) -> HostResult<HttpResponseDto>`
goes through the same pipeline as `http_get` (shared code, so the policy cannot drift): scheme,
`allowedHosts`, public-address check, pinned resolution, 30 s / 16 MiB response limits.
Differences:

* **No redirects** — any 3xx is an error, never followed: what to do with a POST body on
  `301`/`302`/`303` is not the host's guess to make.
* `body` is an already-serialized JSON string; invalid JSON or more than 4 MiB is rejected.
* The host always sends `Content-Type: application/json`.
* Headers are a **fixed allowlist** — `accept`, `accept-language`, `authorization`,
  `user-agent`, `x-api-key`. Anything else (including `Content-Type`, `Host`, `Cookie`) is
  silently dropped, not an error.

### `download_to_cache` — files into `/cache`

`download_to_cache(DownloadToCacheParamsDto { url, hash }) -> HostResult<CachedFileDto>` is for
anything big — mod jars, resource and shader packs, modpack archives. The host downloads the
file straight to disk, so the bytes never pass through the plugin's memory, and returns a path
the plugin can open through WASI:

```rust
// plugin side
let file: HostReply<CachedFileDto> = unsafe { download_to_cache(Wire(DownloadToCacheParamsDto {
    url: "https://cdn.modrinth.com/data/AANobbMI/versions/x/sodium.jar".to_string(),
    hash: FileHashDto::sha1("…40 hex characters…"),
})) }?;
// file.path == "/cache/downloads/sha1/<hex digest>"
```

* **Network policy** — the same code as `http_get`: scheme, `allowedHosts` in the host,
  public addresses only, pinned resolution, **at most 5 redirects, each hop checked** like the
  first request (see `http_get`, step 6). Any other non-2xx final status is an error. No custom
  headers.
* **Hash** — required: `sha1`, `sha256` or `sha512` (no md5). It must be a well-formed hex
  digest of that algorithm (case does not matter). The body is hashed while it streams; on a
  mismatch the call fails and **nothing** is left in the cache, not even a partial file.
* **Name** — chosen by the host from the hash: `/cache/downloads/<algorithm>/<lowercase hex>`.
  The plugin cannot pick it. Calling again with the same hash returns the cached file without
  a network request, after re-hashing it — a file under that name that no longer matches is
  replaced.
* **Limits** — 1 GiB per file; 10 s to connect, 30 s without data, 30 min per attempt (all
  hops included). Transient failures (connection errors, 5xx, 429) are retried up to 3 times;
  a retry starts over from your URL and checks every hop again, so an expired signed CDN URL is
  simply fetched anew. A refused hop is not retried. Downloads share the launcher's global
  limit on concurrent downloads.

### `http_request` — Extism's built-in (disabled)

The network is reachable only through the host functions above (`http_get`, `http_post_json`,
`download_to_cache`). Extism's own `http_request` (`extism_pdk::http::request`) is switched off:
the core gives Extism no host list, so it refuses every call. It would check only the initial
URL and follow redirects unchecked, so a permitted host could send the plugin anywhere.

The hosts a plugin may reach are read on every host function call, from the manifest plus the
user's current plugin settings, so a host added or removed in the settings applies to the very
next request without reloading the plugin.
