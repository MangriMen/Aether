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
4. **Code Comments**: Keep code comments to an absolute minimum. Use them only for complex algorithmic behavior or compiler workarounds. Code comments must be written strictly in English.

## 7. Testing Strategy & Guidelines

This section defines how the codebase must be covered by tests. These rules apply equally to human developers and AI agents to ensure architectural boundaries are never violated during testing.

### 7.1. Test Placement ("Keep it Close")

- **Domain & Application Layers**: Use **inline test modules** (`mod tests`) at the bottom of the implementation file. This preserves encapsulation, allowing you to test `pub(crate)` types and functions without bloating the file tree.
  - _Exception_: If the implementation file exceeds ~400 lines, you may move tests to a peer file (e.g., `logic.rs` and `logic_tests.rs`) within the same directory.
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
