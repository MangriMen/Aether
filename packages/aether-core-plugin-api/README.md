# aether-core-plugin-api

Shared DTOs and the manifest schema for the contract between Aether's core and its Extism
plugins. Both sides depend on this crate, so every change here is a change to the plugin ABI.
Release order: core → tag of this crate → plugins.

## The plugin's id and the API version

The host puts two keys in the Extism config of every plugin; the plugin reads them with
`extism_pdk::config::get` (the pdk wraps them as `plugin_id()` and `host_api_version()`):

| Key | Value |
| --- | --- |
| `aether.pluginId` | the id of the plugin, as the host knows it |
| `aether.apiVersion` | the version of this API the host implements, `MAJOR.MINOR.PATCH` |

`api.version` of the manifest is a [SemVer](https://semver.org) requirement checked against that
version: `0.5.0` and `^0.5` accept 0.5.x, a plugin that needs a function or a field added in a
later version writes `^0.5.2` (`>=0.5.2` has no upper bound and would accept a 0.6 host). While the major version is 0, a new minor is a breaking change
(a changed or removed function, field or value, or a new value of an enum a plugin reads), and a
patch only adds what an older plugin never meets. There are no optional features: a plugin
either needs the version or it does not load.

## Declaring host functions — `api.hostFunctions`

List every host function the plugin imports, by its import name:

```json
"api": {
  "version": "0.5.0",
  "hostFunctions": ["log", "http_get", "write_instance_file"]
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

With `aether-core-plugin-pdk`, the list is the set of `pdk::host` functions the plugin actually
calls (the linker drops the imports of the others); with your own `#[host_fn] extern
"ExtismHost"` blocks, it is the set of functions in them that the plugin calls.

## Restarts lose the plugin's memory

The host restarts a plugin to apply changed settings — folders are mounted only when an instance
is built, so granting a folder means a new instance. A restart drops the old wasm instance with
everything in its memory: statics, caches, anything kept between calls. The new instance gets
the `Loaded` event like on any start, after the old one got `Unloaded`. Keep nothing in memory
that cannot be rebuilt; what must survive belongs on disk (`/cache`, the instance through
`write_instance_file`).

A restart is refused while a call into the plugin is under way or waiting, so it never cuts a
call short.

## `run_command` input

`CommandDto` is `{program, args, currentDir}`. The host checks it before it asks the user (see
the consent below); what it refuses fails the call as `accessDenied`.

* **`program`** must be a Java launcher, `java` or `javaw` (`java.exe`/`javaw.exe` on Windows),
  in the `bin` folder of a Java runtime registered with the launcher: the `path` of a `JavaDto`
  that `get_java` or `install_java` returned (a host path, passed as it is), or the other
  launcher next to it. It is judged on the canonical path, with links and `..` resolved, and that
  canonical path is what runs. A shell, any other binary, a jar or a file in `/cache` is refused,
  and with no Java registered nothing runs at all.
* **`currentDir`** is mandatory. It must be an existing path inside the plugin's `/cache` or
  `/instances` (on the canonical path), written in the `#` form below (`#/instances/<id>`) or as
  the host path. A folder the user granted in the plugin's settings does not count.
* **`#<mount path>`**: a value that starts with `#` (a whole argument, `program` or `currentDir`)
  names a file or folder in the sandbox and is replaced by its host path:
  `#/cache/<file>.jar` becomes the path of that file in the plugin's cache, `#/instances/<id>`
  that of the instance. After the `#` comes a plain path under `/cache` or `/instances` (`/` and
  names: no empty, `.` or `..` part, no `\`, no trailing `/`, no device name such as `nul`), and
  what it names must exist and stay inside that mount once links are resolved; otherwise the
  call is refused. Only a value that starts with `#` is resolved: `--file=#/cache/x` is passed as
  written.
* **`/mnt/<drive>/…`**: a value that starts with `/mnt/` is turned back into the drive path it
  spells on Windows (`/mnt/d/packs/x` → `D:\packs\x`), by its text alone, whatever is mounted
  there; on other systems it is passed as written.
* Every other argument is passed as written. The host does not check arguments: a URL, a flag or
  a host path reaches the program unchanged.
* The process gets its own minimal environment, not the launcher's: `COMSPEC`,
  `NUMBER_OF_PROCESSORS`, `OS`, `PATH`, `PATHEXT`, `PROCESSOR_ARCHITECTURE`, `SystemDrive`,
  `SystemRoot`, `TEMP`, `TMP` and `windir` on Windows; `HOME`, `LANG`, `LC_ALL`, `PATH`, `TMPDIR`
  and `TZ` elsewhere, each copied from the launcher when it has it. A plugin cannot set a
  variable. Standard input is empty.

## `run_command` output

`OutputDto` carries `stdout` and `stderr` as raw bytes, each capped at 1 MiB. A stream that hit
the cap is cut there and flagged — `stdoutTruncated` / `stderrTruncated` — with nothing added
to the bytes themselves.

`exitCode` is the code the program exited with, and `null` when it has none (it was killed by a
signal). A non-zero code is the program's answer, not a failed call: the plugin gets the
`OutputDto` with the code, `stdout` and `stderr`, and decides what it means (`OutputDto::success()`
of the api crate is `exitCode == 0`). The call fails only when the command did not run to its end:
the host or the user refused it, it could not start (`internal`), or it ran past 15 minutes and
was killed (`timedOut`).

## `run_command` needs the user's consent

Besides the program and folder checks, the user is asked before a command runs: *allow once*,
*always*, or *deny*. The question shows the Java launcher, the working directory, the jar and the
arguments; an unanswered question is refused after 5 minutes. While it waits, the call into the
plugin is still running, so the plugin is busy for that long (a stop or restart cancels the
question first).

A refused command fails the call with `HostError::CommandNotApproved { reason, message }` where
`reason` is one of `denied`, `timedOut`, `cancelled` or `jarChanged` (on the wire: `{"type":
"commandNotApproved", "data": {"reason": "denied", "message": …}}`). *Deny* is remembered until
the plugin is stopped or reloaded: the same command is refused again without a new question.

To let the user see why, report the error as a failure envelope (see "Reporting a typed failure"
below), passing the `HostError` as `host`: the launcher then tells the user that the command was
not allowed, and why, with the envelope's `message` as the plugin's words. A call of an importer
or an updater, or of a content provider while content is installed or an instance is updated, is
told this way too; where the launcher only logs the failure (`handle_event`) or shows no toast
(searching and browsing content), it is not shown. A plugin that turns
the error into plain text still fails, but the user only learns that the plugin reported an
error. When a plugin calls another through the host and the other one reported a refusal this
way, the refusal reaches the caller as the same `commandNotApproved`, its `message` being the
other plugin's words, and the caller may pass it on the same way.

*Always* is offered only for `java [-Xmx/-Xms/-Xss…] -jar #/cache/<file>.jar <args>` (the `#`
form of a path, see "`run_command` input"): the jar must be a file in the plugin's `/cache`
whose manifest has no `Class-Path`, and every option before `-jar`
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
as `HostError` cannot be read back from one. The launcher's decoder still happens to read a struct
written as an array, so a plugin that writes arrays may half-work today; do not rely on it, it
may stop being accepted in any version. A reader ignores a key it does not know and gives a
missing key its default when the DTO has one, so a field can be added without breaking the other
side.

A value that may be missing (an id, a version, a link, a description, a hash) has one spelling
for "none": it is optional in the schema, and none is `nil` (`null` in JSON). Never an empty
string, `0` or another stand-in. The launcher writes none as `nil` with the key present; a plugin
may write `nil` or leave the key out, both read as none. A field that is not optional always has
a value: an empty string there is that empty string, not "none".

Only four plain enums the launcher sends have a fallback: `ModLoaderDto`, `ContentTypeDto`,
`CommandRefusalDto` and `InstanceInstallStageDto`. A value a plugin built for an older API does
not know reads as `unknown` there, so the call still decodes; a plugin must not write the
`unknown` of these four, the launcher refuses it. (`unknown` of `ContentVersionStatusDto`, the
`status` of a content version, is not a fallback but an ordinary value a provider may write and
the launcher accepts; that enum has no fallback.) `HostError` and the `error` of its `http` kind
(`HttpErrorDto`) have one too: a kind a plugin does not know reads as `unknown`, whatever its `data` (see "When a host
function fails"). Every other enum is strict
(`HashAlgorithmDto` and the other enums with data, `{"type", "data"}`, have none): a new
value or variant of it breaks a plugin built for an older API, and `api.version` is matched as a
caret requirement, so on 0.x it needs a new minor version.

A host function returns `{"status": "ok", "data": <value>}` or `{"status": "err", "data":
<HostError>}`; `data` is always there (`nil` for `()`). The arguments and answers of a plugin's exports are the DTOs as maps.

Every host function takes one structured argument and answers a `{"status", "data"}` result. An
argument the host cannot read as its DTO (not msgpack, no argument at all, a missing field, a
renamed key, a value of an enum the launcher does not know) is answered with
`invalidArgument`, naming the host function; the plugin's call goes on. The
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
`Wire<T>` for Extism in place of `Msgpack<T>` (and re-exports this crate as its `api`).

## When a host function fails

The `data` of an `err` answer is a `HostError`: `{"type": <kind>, "data": {"message": …}}`. The
kind tells what the plugin can do about the failure; `message` is the launcher's text of it, for
the log or the user, not something to match on. The kinds:

| Kind | Meaning | Expect it when |
| --- | --- | --- |
| `invalidArgument` | What the plugin sent is refused; sending it again fails again. | The argument of a host function cannot be read as its DTO (see "Wire format"); an id, a path, a game or loader version, a page or a content id is not valid; a value is unknown to the launcher or not supported there. |
| `notFound` | What the plugin named does not exist. | An instance, a plugin, an importer, an updater, a provider or a content file is not there. |
| `accessDenied` | The plugin may not do this. | A path leaves the instance or the plugin's folders, a program is not one the plugin may run, `instance_create` names a provider of another plugin. The user's refusal of a command is `commandNotApproved` instead. |
| `commandNotApproved` | The user did not allow the command; `data` also has its `reason`. | See "`run_command` needs the user's consent". |
| `tooLarge` | Something is over a size or count limit of the launcher: a file the plugin writes or reads, or what an install brings in (the files of a plan, its overrides, the index of a pack). An over-long answer from the network is `http` / `responseTooLarge` instead. | `write_instance_file` or `read_instance_file` with a file over the instance-file cap (see "Instance files"). |
| `conflict` | The state does not allow it now; it may succeed later. | The instance is running or still installing, a plugin is loading, unloading or busy, the content is there already. |
| `cancelled` | The work was called off before it finished. | The user called off an install. |
| `timedOut` | The work did not finish in its time. | A command of `run_command` ran past 15 minutes and was killed; Java did not report its properties in time. |
| `pluginCallFailed` | A call into another plugin failed. | The other plugin crashed, hit a limit or reported an error. |
| `http` | A network request failed; `data` also has its `error`, which tells how (below). | `http_get`, `http_post_json` or `download_to_cache` failed, or a request the launcher made for the call (a Java download, a content provider). |
| `internal` | The launcher failed on its own; nothing the plugin can change. | Storage, the file system, and every failure the launcher has no kind for yet. |
| `unknown` | A kind this version of the API does not know: a newer launcher sent it. | Never written by the launcher of a plugin's own API version; read it like `internal`. |

The `error` of `http` is `{"type": <kind>, "data": …}` (`data` absent for `hashMismatch`):

| `error` kind | `data` | Expect it when |
| --- | --- | --- |
| `hostNotAllowed` | `host` | The host is not in the plugin's `allowedHosts`: the URL's, or a redirect's. The user can add it in the plugin's settings. Passed back in a failure envelope from an importer or updater call, or a content provider call during an install or an update, it makes the launcher offer the user to allow that host (see "Reporting a typed failure"). |
| `addressBlocked` | `host`, `port` | The host resolves to an address a plugin may not reach (loopback, private, link-local, …), and the user has not let the plugin reach that host on that port (the port the request was for) as a local address. Passed back in a failure envelope the way `hostNotAllowed` is, it makes the launcher offer the user to allow that address (see "Local addresses the user allows"). A host that resolves to no address at all is `network`. |
| `invalidRequest` | `reason` | The request is refused before anything is sent: a malformed URL, a scheme other than `http`/`https`, no host, a forbidden or invalid header, a body that is not JSON or is over 4 MiB, a malformed expected hash of a download. Sending it again fails again. |
| `redirect` | `reason` | A redirect is not followed: `https` → `http`, no usable `Location`, more than 5, or any 3xx where none is followed by `http_post_json`. (A redirect to a scheme other than `http`/`https` arrives as `invalidRequest`.) |
| `responseTooLarge` | `limit` | The answer is over the limit, in bytes: 16 MiB for `http_get` and `http_post_json` (or a third of the plugin's `load.memoryLimit` when that is less, see "Networking from a plugin"), 1 GiB for a download. |
| `status` | `status` | A final answer that is not 2xx and not handed back as an answer: a download's (a final `304` is one too), a provider's API. `http_get` and `http_post_json` hand any non-3xx status back as an answer instead. |
| `hashMismatch` | none | A downloaded file is not the one its hash names. |
| `invalidResponse` | `reason` | An answer the launcher reads itself (a content provider's, say) is not the JSON, TOML or text it expects. |
| `network` | `reason` | No usable answer came: the host could not be resolved (or resolved to no address) or reached, the connection or TLS failed, the request timed out or broke off, a download ran out of time. It may succeed later. |
| `unknown` | none | A kind this version of the API does not know: a newer launcher sent it. Read it as a failed request with no more detail. |

`reason` is text for the log or the user, like `message`. A failure the launcher has on its own
while it makes a request (writing a download to the cache, say) is `internal`, not `http`.

Of the failures `run_command` raises itself, a command that ran past its time is `timedOut`; one
that could not start, or whose output the launcher could not read, is `internal`. A plugin
writes none of these kinds itself, except to pass one back in a failure envelope.

## Reporting a typed failure (any language)

A plugin call that fails reaches the host as text. To let the host tell *why* — a refused command
or host is the case today — the plugin sets that text to a failure envelope:

```
aether-plugin-failure:{"message":"...","host":{...}}
```

The marker, directly followed by one line of JSON that follows
[`schemas/plugin-failure.schema.json`](src/v0/schemas/plugin-failure.schema.json), is the
whole string given to `error_set`; return a non-zero code after it. The host finds the marker
anywhere in the string, so a prefix a framework adds is fine, and it ignores what follows the
JSON. A string without a valid envelope is an ordinary failure with that text as its cause.

`host` is the `HostError` a host function returned to the plugin, passed back untouched.

**With Rust**, use `aether-core-plugin-pdk`; it re-exports this crate as
`aether_core_plugin_pdk::api`, so a plugin needs only that one dependency. Call a host function
through `aether_core_plugin_pdk::host` (`pdk::host::http_get(request)?`) inside a function that
returns `aether_core_plugin_pdk::Result`: the answer comes back decoded, a host's refusal as
`pdk::Error::Host`. (A plugin that declares its own `#[host_fn]` import returns `HostReply<T>`
from it and reads it with `unsafe { f(Wire(arg)) }?.into_result()?`.) `#[plugin_fn]` itself must
return `FnResult`, so call that function from it with `?`, which turns the error into the
envelope. The envelope's `message` is the host's message, or the plugin's words before it when
the plugin adds them with `.context("downloading the pack")` (`ResultExt`); `host` is the host
error unchanged either way. An envelope that went through `anyhow` and back (a helper returning
`FnResult` called with `?`) is read back into the same error, not wrapped in a second one. The
crate documentation has an example.

**In another language**, do the steps by hand:

1. Decode the msgpack response of the host function (see "Wire format"): `{"status": "ok",
   "data": <value>}` or `{"status": "err", "data": <HostError>}`.
2. On `err`, build `{"message": <the plugin's words>, "host": <that HostError>}` (the
   `message` is any text: what the plugin was doing, the host's `message`, or both), prefix it
   with the marker and call `error_set` with it.
3. Return a non-zero code.

What the host makes of `host` is described where the error is documented: for `run_command`, see
"`run_command` needs the user's consent" above. An `http` error of kind `hostNotAllowed` passed
back from a call of an importer or an updater, or of a content provider while content is installed
or an instance is updated, makes the launcher tell the user which host the plugin was refused and
offer to allow it. An `addressBlocked` passed back the same way makes it tell the user which
host and port resolved to a local address and offer to let the plugin reach it, behind a
warning (see "Local addresses the user allows"). It does nothing more where the launcher only logs the failure
(`handle_event`) or shows no toast (searching and browsing content), and a plugin that calls another one through the
host gets the other one's refused host or address as `pluginCallFailed`, not `hostNotAllowed` or
`addressBlocked`: it is not a host of its own to ask for.

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

## Instance files

`/instances` is mounted read-only: a plugin reads an instance through WASI and its own code writes into it
only through `write_instance_file` (a command it runs there may write too).

* `instance_get_dir` answers the sandbox path of the instance, `/instances/<id>`, and
  `instance_plugin_get_dir` that of the plugin's own folder in it,
  `/instances/<id>/.metadata/plugins/<plugin id>`: paths to open through WASI, not host paths.
* `write_instance_file` and `read_instance_file` take the instance id and a path relative to the
  instance, `/`-separated plain names (`config/pack.toml`); a sandbox path such as
  `/instances/<id>/…` or `/mnt/…` is refused. A write replaces the whole file.
* `.metadata` is the launcher's: a write anywhere in it (in any case) is refused as
  `invalidArgument`, except into the plugin's own `.metadata/plugins/<plugin id>/`, spelled
  exactly so. Reading it is allowed.
* One file read or written in one call is at most 64 MiB, or a third of the plugin's
  `load.memoryLimit` when that is less (the bytes are in the plugin's memory three times while
  they pass; without a `memoryLimit` the plugin gets 256 MiB, so 64 MiB). A bigger file fails
  the call as `tooLarge`.
* The `iconPath` of `instance_create` is a file in the plugin's `/cache`
  (`/cache/icon.png`); any other path fails the call as `accessDenied`. An icon the launcher
  cannot import (not an image, or over 4 MiB) does not fail the call: the instance is created
  without one, with a warning.
* The `packInfo.providerId` of `instance_create` names a provider of the calling plugin: its
  `pluginId` is the plugin's own id, its `capabilityId` any of the plugin's capabilities (the
  one that updates the pack). The provider decides which plugin updates the instance and is told
  of its launches, so another plugin's id, `aether` included, fails the call as `accessDenied`.

## Networking from a plugin

A plugin has two ways to make an HTTP request. They are **not** equally safe.

### `http_get` — the host function (preferred)

```rust
// plugin side
// in a function that returns `aether_core_plugin_pdk::Result`
let response: HttpResponseDto = pdk::host::http_get(HttpRequestDto {
    url: "https://example.com/pack.toml".to_string(),
    headers: HashMap::new(),
})?;
```

The host performs the request itself and applies, in this order:

1. **Scheme check** — only `http` and `https`.
2. **Allowlist check in the host** — the URL's host must match the plugin's `allowedHosts`
   (manifest entries plus whatever the user added in the plugin's settings), or be a local
   address the user allowed on that port (see "Local addresses the user allows"). Matching
   uses glob semantics.
3. **Header check** — connection-control and virtual-host headers (`host`, `content-length`,
   `connection`, `transfer-encoding`, …) are rejected.
4. **Address check** — the host name is resolved and **every** returned address must be public.
   Loopback, private, link-local (including the cloud metadata address `169.254.169.254`),
   CGNAT (`100.64.0.0/10`), multicast, documentation and reserved ranges are refused, in both
   IPv4 and IPv6 (including the `::ffff:a.b.c.d` form). If a name resolves to a mix of public
   and non-public addresses, the whole request is refused. Two exceptions:
   * `198.18.0.0/15` (reserved for benchmarking) passes: VPN and proxy clients in TUN mode
     with fake-ip DNS (Clash, sing-box, …) answer every public name with an address from it
     and route the connection by the name, so refusing it refused public hosts. Only the local
     VPN client ever answers there, and it connects to the real destination of the name;
   * a host and port the user allowed as a local address may resolve to non-public addresses
     too, but never to an unspecified (`0.0.0.0`, `::`), broadcast or multicast one.
5. **Pinned connection** — the request is pinned to the addresses that were just checked, so a
   second DNS answer cannot swap in a private address between the check and the connect
   (DNS rebinding).
6. **Checked redirects** — `301`, `302`, `303`, `307` and `308` are followed, **at most 5**
   times; a sixth redirect fails the call ("Too many redirects: more than 5"). Every hop goes
   through steps 1, 2, 4 and 5 again **before** anything is sent to the new host: the target must be
   `http`/`https`, its host must be in `allowedHosts`, all its addresses must be public (or
   the hop's host and port allowed as a local address: a hop to a local address that is not
   allowed is refused, whatever the first host was), and the connection is pinned to them. On
   top of that:
   * `https` → `http` is refused ("Redirect from `…` to `…` would downgrade https to http");
   * a relative `Location` is resolved against the URL that answered; a missing or
     unparseable one fails the call;
   * your headers are sent only while the chain stays on the **origin** (scheme, host and
     port) you asked for. Once a hop leaves it, they are dropped for the rest of the chain —
     even if a later hop comes back — so an API key never reaches a CDN or a third party;
   * the redirect's own body is never read; `status`, `headers` and `body` describe the final
     response. Any other 3xx (`300`, `304`, …) is an error, not a response.

   A redirect the host does not follow fails the call as `http` with the `error` `redirect`; a
   hop to a host outside `allowedHosts` as `hostNotAllowed` with that host, one to a non-public
   address as `addressBlocked` with that host and port (see "When a host function fails" for
   every kind).

   Downloads from GitHub releases need this: `/releases/latest/download/<file>` answers `302`
   to `/releases/download/<tag>/<file>`, which answers `302` to a signed, expiring URL on
   `release-assets.githubusercontent.com`. Put **both** `github.com` and
   `release-assets.githubusercontent.com` in `allowedHosts`; without the second, the call fails
   at that hop as `http` / `hostNotAllowed` with that host.
7. **Limits** — 30 s total for the whole chain of hops, 10 s to connect, 16 MiB of response
   body, or a third of the plugin's `load.memoryLimit` when that is less (the body is in the
   plugin's memory three times while it passes; without a `memoryLimit` the plugin gets
   256 MiB, so 16 MiB). An oversized body fails the call as `http` / `responseTooLarge`; it is
   never silently truncated.

**Rule for plugin authors: use `https://`, and do not expect headers to cross origins.** A key
sent to an `http://` URL is exposed on the very first request, and your headers are never
carried to another origin (scheme, host or port) — not even `http://a` → `https://a`.

**Behind a proxy.** If the user has a proxy configured — the proxy environment variables
(`HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY`, `NO_PROXY`) or, failing those, the OS settings, the
same source the launcher's own requests use — the host functions go through it. The proxy
then resolves host names and connects itself, so steps 4 and 5 cannot apply to a host *name*:
it is passed to the proxy unresolved. Everything else stays: the scheme check, `allowedHosts`
on every hop, the redirect limit and rules, the header drop, and the address check of a host
given as an IP literal (a local address the user allowed passes it). The proxy's own address is
not filtered — the user set it up, not the plugin. The launcher logs once that plugin requests
go through the proxy.

**Local addresses the user allows.** A pack or a jar on a server in the user's own network, on
a NAS or on this computer is refused by step 4. The user, and only the user, can let one plugin
reach one such host on one port: when an importer, updater or content provider call fails with
`addressBlocked` passed back (see "Reporting a typed failure"), the error toast offers "Allow
this local address", and a confirmation warns that the address is in the user's own network and
that the plugin will be able to send requests to it. A URL on `localhost` or a local address
literal typed into an importer's field is asked for the same way before the import starts. A
manifest cannot ask for one, and nothing a plugin sends can give one. A grant:

* names exactly one host as the request names it — an IP literal (`192.168.1.10`, `[fd12::1]`),
  `localhost` or a DNS name — and the port the refused request was for; never a pattern;
* admits that host on that port past `allowedHosts` too, and lets its addresses be non-public;
  every other check stays, and the connection is still pinned to the addresses just checked;
* is checked on every hop: a redirect to a local address that is not allowed is refused, one
  to an allowed one is followed;
* is listed under "Local addresses" in the plugin's settings, where the user can take it back.
  It is kept with the plugin's settings (`localHosts`, `host:port` each; one written there by
  hand as a bare `host` admits every port of it, and one in any other form is dropped, not
  granted).

The response body is a `Vec<u8>` sent as a msgpack `bin` (so a plugin decodes it with a copy, not
an instruction per byte — a multi-megabyte body would otherwise use up the plugin's fuel); the same
holds for the bytes of `write_instance_file`/`read_instance_file` and for the streams of
`run_command`. Interpreting the body (JSON, TOML, …) is the plugin's job. For anything large use
`download_to_cache`, which keeps the body out of the plugin's memory.
Repeated response headers collapse to the last value.

### `http_post_json` — JSON POST

`http_post_json(HttpPostJsonRequestDto { url, headers, body }) -> HostResult<HttpResponseDto>`
goes through the same pipeline as `http_get` (shared code, so the policy cannot drift): scheme,
`allowedHosts`, public-address check, pinned resolution, 30 s / 16 MiB response limits (the
same `memoryLimit` rule).
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
// in a function that returns `aether_core_plugin_pdk::Result`
let file: CachedFileDto = pdk::host::download_to_cache(DownloadToCacheParamsDto {
    url: "https://cdn.modrinth.com/data/AANobbMI/versions/x/sodium.jar".to_string(),
    hash: FileHashDto::sha1("…40 hex characters…"),
})?;
// file.path == "/cache/downloads/sha1/<hex digest>"
```

* **Network policy** — the same code as `http_get`: scheme, `allowedHosts` in the host,
  public addresses only, pinned resolution, **at most 5 redirects, each hop checked** like the
  first request (see `http_get`, step 6). Any other non-2xx final status is an error, `http` with
  the `error` `status`. No custom headers.
* **Hash** — required: `sha1`, `sha256` or `sha512` (no md5). It must be a well-formed hex
  digest of that algorithm (case does not matter); a malformed one fails the call as `http` with
  the `error` `invalidRequest`. The body is hashed while it streams; on a mismatch the call fails
  with `hashMismatch` and **nothing** is left in the cache, not even a partial file.
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
