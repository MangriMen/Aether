# Aether Architecture Guidelines

This document defines the code structure, dependency direction, and module boundaries for the project. Adherence to these rules is mandatory for both human developers and AI agents.

---

## 1. Architectural Style: Vertical Slices + Hexagonal (Ports & Adapters)

The backend code base is divided into independent functional slices at the root level. Inside each slice, Hexagonal Architecture isolates business or technical logic from external frameworks, libraries, and delivery mechanisms.

### Root Directory Allocation:

- **`src/core/`** — **Shared Kernel**: Contains basic domain primitives, global Result/Error types, and zero-dependency language extensions.
- **`src/features/`** — **Business Slices**: Domain-driven features specific to launcher logic (e.g., `auth`, `instance`, `minecraft`).
- **`src/shared/`** — **Technical Slices**: Standalone infrastructure-driven modules (e.g., `request_client`, `cache`, `idempotency`).

### Feature & Shared Slice Layout:

Any slice inside `src/features/` or `src/shared/` must follow this structure if it contains technical implementations:

- `mod.rs` — **Slice Facade**: The entry point managing public contracts.
- **`domain/`** — **Core Logic**: Pure language constructs only. Contains entities, value objects, and domain invariants. Zero technical concerns (no async traits, no network/storage logic).
- **`app/`** — **Application Layer**: Inbound Ports (Use Cases), Outbound Ports (Traits for Storage, Network, OS APIs), Data Transfer Objects (DTOs), and internal application services (coordination logic).
- **`infra/`** — **Infrastructure Layer**: Technical details and outbound adapters. Low-level implementations binding third-party crates (SQLx, reqwest, Tauri) to ports defined in `app/`.

### Use cases and ports:

- A use case is `pub struct <Verb><Noun>UseCase` holding its dependencies as `Arc<dyn Port>`, with a constructor `new(…)` and one method, `execute(…)`.
- When Tauri calls it, it implements an inbound port `<Verb><Noun>UseCasePort`; when another feature calls it, a port named for the role (`*Service`).
- Outbound ports are named for the role, without a `Port` suffix: `*Storage`, `*Provider`, `*Service`.
- A use case file is named after the use case (`create_offline_account.rs`), without a `_uc` suffix.

---

## 2. Dependency Rule

Dependencies must strictly point **inward**:

$$\text{infra} \longrightarrow \text{app} \longrightarrow \text{domain}$$

- **`domain`**: Fully isolated. Zero internal project dependencies. Contains pure logic and business rules.
- **`app`**: Depends only on `domain`. Implements use cases and defines communication contracts via Ports (abstractions).
- **`infra`**: Depends on `app` and `domain`. Implements Adapters binding technical providers to application ports.

---

## 3. Module Granularity: "File until it gets crowded"

To prevent directory bloat and redundant transit module files, the project enforces a strict structural density threshold:

1. **Low Density (1–2 Entities)**: Must live as a single file (`module_name.rs`) directly under the layer root. Creating a subdirectory with a separate `mod.rs` for 1–2 files is strictly prohibited.
   - _Exception_: Rich Domain Models or adapters with heavy internal logic (~150+ lines of validation/invariants) must remain isolated in separate files within a directory to preserve readability. Granularity is measured by logical density, not raw file counts.
2. **High Density (3+ Entities)**: When a specific sub-layer grows to 3 or more tightly coupled files (e.g., multiple use cases or multi-file adapter implementations), it must be refactored into a directory containing a local `mod.rs` to manage internal files.

---

## 4. Re-export (pub use) & Boundary Control

Improper re-exports break architectural boundaries and ruin automated linting.

### Rules for Slice Roots (`src/features/[name]/mod.rs` and `src/shared/[name]/mod.rs`):

1. **Strict Layer Encapsulation**: Making `domain` and `app` layers public (`pub mod domain;`, `pub mod app;`) is strictly prohibited. Keep them private.
2. **Composition Root Exception for Infra**:
   - If the Dependency Injection container wires the module within the same crate: use `pub(crate) mod infra;`.
   - If dependencies are wired by an external crate (e.g., Tauri frontend app / multi-crate architecture): `pub mod infra;` is allowed, but types must be accessed strictly via the `infra_facade` to keep paths predictable.
3. **No Layer Wildcards**: Broad layer re-exports (`pub use domain::*;` or `pub use app::*;`) at the root facade are strictly prohibited. Every exposed component must be listed explicitly.
4. **Pragmatic Model Exposure**: Pure data-container domain models (Anemic Models / Value Objects without private invariants, heavy logic, or security risks) MAY be explicitly re-exported from the slice root via explicit `pub use` for direct use by other Rust features.
5. **Facade Principle**: Beyond safe domain models, the slice root acts as a strict facade. It must expose _only_ what the outside world requires to interact with the slice: Inbound Ports (Use Cases), input/output DTOs, and Application Errors.
6. **No Infra Leaks**: Infrastructure implementation types must **NOT** be directly re-exported from the slice root. Every consumer import of an infrastructure type must explicitly contain the `::infra::` or `::infra_facade::` token.

### Rules for the Infra Layer (`mod.rs` inside `infra/`):

1. **Submodules are internal**: Group adapters into submodules (e.g., `sqlite/`, `file_system/`) as needed, but keep them private (`mod sqlite;`).
2. **Explicit Re-export**: Use explicit re-exports so the Composition Root accesses adapters via `[slice_name]::infra::SqliteStorage`, bypassing deep nested submodule paths like `[slice_name]::infra::sqlite::SqliteStorage`.
3. **Rationale**: This preserves the mandatory `::infra::` or `::infra_facade::` token in consumer import paths for automated architecture linters, while completely hiding internal submodule nesting from consumers.

---

## 5. Global Utilities and Shared Kernel (`src/core/`)

1. **No Global Utils**: Flat, unrestricted `utils/` directories or files acting as "catch-all garbage bins" are completely banned.
2. **Shared Kernel Allocation**: Low-level language extensions, clean formatting helpers, and standard macro extensions with zero external dependencies must live under `src/core/domain/extensions/`.
3. **Encapsulated Technical Helpers**: Technical helpers requiring specialized crates (e.g., crypto hashing, async execution, environment lookups) must be treated as independent technical slices inside `src/shared/` or enclosed privately inside a specific feature's `infra/` layer.

---

## 6. Instructions for AI Agents

1. **Do not bloat the tree**: Prioritize code encapsulation. Add new DTOs, models, or adapters to existing files unless the high-density threshold or complexity exception in Rule 3 is fully met.
2. **Respect Encapsulation**: Never reference `infra` types inside `domain` or `app` (except when writing the actual adapter implementations inside `infra/`). Cross-feature communication must happen exclusively by invoking another feature's public Use Cases or accessing explicitly exposed safe domain models.
3. **UI Boundary DTOs**: DTO conversion for presentation layers (e.g., Tauri Commands delivering data to the frontend) must live at the boundary interface (the Tauri/API layer itself), separating presentation serializability from internal Rust core communication.
4. **Code Comments**: Follow the rules in [section 8](#8-code-comments).
5. **Backend errors**: Follow [section 12](#12-the-tauri-boundary), in particular how an error crosses the boundary.
6. **New rules**: Sections 9–12 apply to new and edited code. Existing code is brought in line slice by slice, when a file is touched anyway.

## 7. Testing Strategy & Guidelines

This section defines how the codebase must be covered by tests. These rules apply equally to human developers and AI agents to ensure architectural boundaries are never violated during testing.

### 7.1. Test Placement ("Keep it Close")

- **Domain & Application Layers**: Use **inline test modules** (`mod tests`) at the bottom of the implementation file. This preserves encapsulation, allowing you to test `pub(crate)` types and functions without bloating the file tree.
  - _Exception_: If the implementation file exceeds ~400 lines, you may move tests to a peer file named `<module>_test.rs` (e.g., `logic.rs` and `logic_test.rs`) within the same directory. A module that is a directory keeps them in `tests.rs`.
- **Use case tests with fixtures** live in `features/<feature>/tests/<use_case>_test.rs`, included from the feature's `mod.rs` with `#[cfg(test)] mod tests;`.
- **Infrastructure Layer**: For complex adapters (e.g., network clients, heavy file system interaction), prefer a separate peer test file next to the adapter to keep production code readable.

### 7.2. Layer-Specific Testing Rules

| Layer      | Target                                              | Test Nature                            | Allowed Tools / Dependencies                                          |
| :--------- | :-------------------------------------------------- | :------------------------------------- | :-------------------------------------------------------------------- |
| **Domain** | Business rules, invariants, validation.             | Pure, synchronous, 100% deterministic. | **No async, no mocks, no external I/O.** Pure Rust only.              |
| **App**    | Use Cases, coordination logic, DTO flows.           | Asynchronous, isolated behavior.       | `tokio::test`, `mockall` (to mock Outbound Ports).                    |
| **Infra**  | Adapters, third-party integrations (SQLx, reqwest). | Component/Integration testing.         | `wiremock` (network), `tempfile` (OS/FS). **No absolute host paths.** |

### 7.3. Strict Testing Constraints

1. **No Outward Leaks**: Never import `infra` adapters or actual technical implementations when testing `domain` or `app` layers. Test the layers in complete isolation using ports (traits) and mocks.
2. **Preserve Encapsulation**: Do not change visibility modifiers (e.g., changing `pub(crate)` to `pub`) or expose internal fields _just to satisfy a test_. Use internal module visibility via `super::*` in your inline tests.
3. **Naming Convention**: Test names must explicitly state the expected outcome and condition using the following pattern:
   `should_[expect_behavior]_when_[condition]`
   _(Example: `should_fail_validation_when_username_is_empty`)_.

## 8. Code Comments

These rules apply equally to human developers and AI agents.

1. **Explain why, not how.** A comment says what the code cannot: the reason for a decision, an invariant, a trap, what a check protects against. Never narrate what the code does or list the steps that the function already shows.
2. **Security boundaries are the exception to minimalism.** On a sandbox limit, allowlist, path or URL check, say what it guards against and what would break if it were loosened.
3. **Keep them short.** Usually one to three lines. A doc comment states the contract (what the caller may rely on), not the implementation. Add `///` to a public item only when its name and types do not already say it. Module-level `//!` is for the non-obvious flow or invariants of the module as a whole.
4. **No task, ticket, finding or question numbers.** `T-1.2`, `F-49`, `Q17` and the like go stale and point nowhere once the planning documents are gone. If the reason matters, write the reason; the link to the task belongs in the commit message or the pull request.
5. **Do not leave history in code.** Not "was changed from X", not "temporary until Y" without a concrete condition. Describe the code as it is now.
6. **English only.**

## 9. Domain Types and Errors

Sections 9–12 apply to new and edited code; each rule names one example to copy.

### 9.1. Validated newtypes

- The field of a validated newtype is private. The check lives in one constructor, and every other way in delegates to it.
- Names, paths and hashes are built by `parse(&str) -> Result<Self, Invalid<Type>>`, numbers by `new(n) -> Result<…>`, identifiers by `TryFrom<String>` (plus `TryFrom<&str>` or `FromStr`) with `#[serde(try_from = "String", into = "String")]`.
- A literal in code is built by a panicking `from_static`. A lenient read from storage goes through `from_stored_or_<fallback>`, which logs the bad value.
- The error is an `Invalid<Type>` of its own, not the error of the whole feature.
- Do not implement `Deref<Target = str>`: it leaks every `str` method into the public API of a type whose point is its invariant. Hand out `as_str()`, `AsRef<str>`, `Display` and `From<Type> for String`.
- Example: `PluginPath` (`features/plugins/domain/models/plugin_path.rs`); `from_static` in `shared/capability/domain/ids.rs`; `new` in `settings/domain/models/bounded_numbers.rs`.

### 9.2. Paths from external data

A path segment that comes from outside (Mojang or Modrinth metadata, a maven coordinate, a version id, a plugin manifest, an API answer) becomes part of a path only as a value of a type that checks segments (`InstanceRelativePath`, `ContentPath`, `PluginPath`) or through the shared "plain file name" helper. A bare `dir.join(external_string)` is not allowed. Example: `LocationInfo::log_config_path` in `settings/domain/models/location_info.rs`.

### 9.3. Errors

- Errors are `thiserror` types. `anyhow` is allowed only where a third-party crate hands it out (Extism host functions, `reqwest-middleware`) and in the composition root of the desktop app, never in `domain`.
- A variant has no `Error` suffix: `Storage(…)`, not `StorageError(…)`. An error about a value is `Invalid<Value>`; a reason for a refusal is `<X>Rejection`.
- The cause goes through `#[source]`, not through `to_string()`.

---

## 10. Storage

### 10.1. Storage formats are separate from the domain

- A domain model does not derive `Serialize`/`Deserialize` for the sake of storage. A file on disk is described by an infra type `<Name>V<n>` with `From`/`TryFrom` both ways; a SQLite row by `Sql<Name>`, whose columns are parsed by `parse_stored_*` functions.
- The suffix `Dto` is only for the Tauri or plugin boundary. The exception is an identifier with `serde(try_from)`.
- Example: `ModLoaderV1` in `instance/infra/fs/instance.rs`; `SqlPackInfo` in `instance/infra/sqlite/sql_pack_info.rs`.

### 10.2. SQL

- Production SQL is written with `sqlx::query!` / `query_as!`, which are checked against `.sqlx`. The runtime form `sqlx::query(…).bind(…)` is allowed in tests, for sqlx's own tables and for text that is built dynamically, with a comment saying why.
- SQL lives in the slice's `infra/sqlite/` and nowhere else.
- Example: `instance/infra/sqlite/sqlite_instance_storage.rs`.

---

## 11. Limits and Untrusted Input

### 11.1. Limits and timeouts

- Every limit on size or count, and every timeout, that protects against external input (network, archives, plugins) is declared in `packages/aether-core/src/shared/limits.rs` with a comment on what it protects against. An adapter only imports it.
- Two constants with the same name and different values are not allowed.
- Not covered: the bounds of a value type's own validation (`MAX_USERNAME_LENGTH` stays in `Username`: `domain` must not depend on `shared`) and the limits of the plugin archive extractor (`ZipPluginExtractorConstants`).

### 11.2. HTTP clients for untrusted URLs

A URL that comes from a plugin, from provider metadata or from Markdown is fetched only by a client whose redirect policy is `Policy::none()` or a `Policy::custom` that checks every hop by the same rule as the first URL (https, public host, allowlist). A shared client with the default policy is for fixed, trusted hosts only. Examples: `VettedHops` in the plugin `http_*` host functions; `icon_redirect_policy` in `instance/infra/fs_content_icon_cache.rs`.

---

## 12. The Tauri Boundary

### 12.1. Commands

- Every feature is a Tauri plugin with `init()` and a `#[must_use] get_specta_commands()`, plus `get_specta_events()` when it has events. Commands are listed in a `<x>_commands!` macro (`apps/desktop/src/shared/commands.rs`), not registered by hand.
- A command is a private `async fn` with `#[tauri::command] #[specta::specta]`, named `verb[_object]` without the feature prefix. It returns `FrontendResult<T>` and converts a core error with `.map_err(aether_core::Error::from)?`.
- A command that changes state takes `request_id: RequestId` and starts with `idempotency.lock_cmd(request_id)?`.
- A synchronous command is not allowed: it runs on the main thread.
- Example: `apps/desktop/src/features/instance/infra/tauri/api/commands.rs`.

### 12.2. Events

- A backend event is `<Name>EventDto` deriving `Type, Event`, with `pub seq: u32` as the first field, taken from the one shared counter. It is registered through `get_specta_events()`.
- When an event has several kinds, the kind is a field `#[serde(flatten)] kind: <Name>EventKindDto`.
- The frontend listens only through `useEventListener`, which drops a `seq` it has seen.
- Example: `events/infra/tauri/dtos/cache_event.rs`, `plugins/infra/tauri/dtos/plugin_event.rs`.

### 12.3. DTOs

- Boundary types are named `<Name>Dto` and live in `infra/tauri/dtos/`. A conversion is a `From`/`TryFrom` in the same file; there are no `to_dto` functions.
- Structs have `#[serde(rename_all = "camelCase")]`, enums `"snake_case"`. Any other case is only for a value that must match an external format, with a comment saying so.

### 12.4. Errors

A DTO carries the error code, typed arguments (id, path, URL without query, version, status, limit), translatable `reason` codes and a `cause` category. Raw error text may cross the boundary only as `cause: { type: "unknown", details }`, which the frontend shows only in a collapsible details block. Never put `to_string()` into any other field. The cause of every failed command is logged. The frontend side is in `apps/frontend/ARCHITECTURE.md`, rule 9.
